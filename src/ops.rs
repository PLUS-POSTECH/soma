use std::convert::TryFrom;
use std::fs;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use fs_extra::{dir, file};
use futures::Future;
use handlebars::Handlebars;
use hyper::client::connect::Connect;
use tempfile::tempdir;
use tokio::runtime::current_thread::Runtime;

use crate::docker;
use crate::prelude::*;
use crate::problem::configs::SolidBinaryConfig;
use crate::problem::Problem;
use crate::repository::backend;
use crate::template::{HandleBarsExt, Templates};
use crate::{Environment, NameString, Printer};

pub fn add(
    env: &mut Environment<impl Connect, impl Printer>,
    repo_location: &str,
    repo_name: Option<&NameString>,
) -> SomaResult<()> {
    let (resolved_repo_name, backend) = backend::location_to_backend(repo_location)?;
    let resolved_repo_name = NameString::try_from(resolved_repo_name);
    let repo_name = match repo_name {
        Some(repo_name) => Ok(repo_name),
        None if resolved_repo_name.is_ok() => Ok(resolved_repo_name.as_ref().unwrap()),
        _ => Err(resolved_repo_name.err().unwrap()),
    }?;

    env.repo_manager_mut().add_repo(&repo_name, backend)?;

    let mut repository = env.repo_manager().get_repo(&repo_name)?;
    repository.update(&[])?;

    env.printer()
        .write_line(&format!("Repository added: '{}'", &repo_name));

    Ok(())
}

pub fn fetch(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    cwd: impl AsRef<Path>,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;
    let manifest = problem.load_manifest()?;

    manifest
        .public_files()
        .into_iter()
        .try_for_each(|public_file_path| {
            let file_path = problem.path().join(public_file_path);
            let file_name = file_path.file_name().ok_or(SomaError::FileNameNotFound)?;

            env.printer()
                .write_line(&format!("Fetching '{}'...", file_name.to_string_lossy()));
            fs::copy(&file_path, cwd.as_ref().join(file_name))?;
            Ok(())
        })
}

pub fn build(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;

    runtime.block_on(docker::prune_images_from_prob(&env, &problem))?;
    build_image(&env, &problem, runtime)?;
    env.printer().write_line(&format!(
        "Built image for problem: '{}'",
        problem.fully_qualified_name()
    ));
    Ok(())
}

fn construct_image_root(
    image_root: impl AsRef<Path>,
    problem_dir: impl AsRef<Path>,
    binary_config: &SolidBinaryConfig,
) -> SomaResult<()> {
    let mut dir_copy_options = dir::CopyOptions::new();
    dir_copy_options.copy_inside = true;
    // Latter entry has higher priority
    dir_copy_options.overwrite = true;
    let mut file_copy_options = file::CopyOptions::new();
    file_copy_options.overwrite = true;

    for (local_path, target_path) in binary_config.path_maps() {
        let local_path = problem_dir.as_ref().join(local_path);
        let destination = image_root.as_ref().join(target_path.strip_prefix("/")?);
        // TODO: more descriptive error
        fs::create_dir_all(destination.parent().ok_or(SomaError::InvalidManifest)?)?;
        if local_path.is_dir() {
            if destination.exists() {
                unimplemented!("Handling copy of nested or duplicate directory");
            }
            dir::copy(local_path, destination, &dir_copy_options)?;
        } else if local_path.is_file() {
            file::copy(local_path, destination, &file_copy_options)?;
        } else {
            Err(SomaError::FileUnreachable)?;
        }
    }
    Ok(())
}

fn encode_context(path: impl AsRef<Path>) -> SomaResult<Vec<u8>> {
    let compressor = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar = tar::Builder::new(compressor);
    tar.append_dir_all("", path)?;
    tar.finish()?;
    let compressor = tar.into_inner()?;
    Ok(compressor.finish()?)
}

fn build_image(
    env: &Environment<impl Connect, impl Printer>,
    problem: &Problem,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let image_name = problem.docker_image_name(env.username());

    env.printer().write_line("Preparing build context...");
    let context = tempdir()?;
    let context_path = context.path();

    env.printer().write_line("Loading manifest...");
    let manifest = problem.load_manifest()?.solidify()?;

    env.printer().write_line("Constructing image root...");
    let image_root = context_path.join("image-root");
    let problem_dir = problem.path();
    fs::create_dir(&image_root)?;
    let binary_config = manifest.binary();
    construct_image_root(image_root, problem_dir, binary_config)?;

    env.printer().write_line("Rendering build files...");
    fs::create_dir(context_path.join(".soma"))?;
    Handlebars::new().render_templates(Templates::Binary, &manifest, context_path)?;

    env.printer().write_line("Encoding build context...");
    let build_context = encode_context(context_path)?;

    context.close()?;
    env.printer().write_line("Building image...");
    let labels = docker::docker_labels(&env, &problem);
    runtime.block_on(docker::build(&env, labels, &image_name, build_context))?;
    Ok(())
}

pub fn run(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    port: u32,
    runtime: &mut Runtime,
) -> SomaResult<String> {
    let problem = env.repo_manager().search_prob(prob_query)?;
    let image_name = problem.docker_image_name(env.username());
    let port_str = &port.to_string();

    let containers = runtime.block_on(docker::list_containers(&env))?;
    if docker::container_from_prob_running(&containers, &problem) {
        Err(SomaError::ProblemAlreadyRunning)?
    }

    runtime.block_on(docker::prune_containers_from_prob(&env, &problem))?;

    let labels = docker::docker_labels(env, &problem);
    let container_run =
        docker::create(env, labels, &image_name, port_str).and_then(|container_name| {
            env.printer().write_line("Starting container...");
            docker::start(env, &container_name).map(|_| container_name)
        });

    env.printer().write_line(&format!(
        "Creating container for problem: '{}'",
        problem.fully_qualified_name()
    ));
    let container_name = runtime.block_on(container_run)?;
    env.printer()
        .write_line(&format!("Container started: '{}'", &container_name));

    Ok(container_name)
}

pub fn remove(
    env: &mut Environment<impl Connect, impl Printer>,
    repo_name: &NameString,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let image_list = runtime.block_on(docker::list_images(env))?;
    if docker::image_from_repo_exists(&image_list, repo_name) {
        Err(SomaError::RepositoryInUse)?;
    }

    env.repo_manager_mut().remove_repo(repo_name)?;
    env.printer()
        .write_line(&format!("Repository removed: '{}'", &repo_name));

    Ok(())
}

pub fn clean(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;

    let container_list = runtime.block_on(docker::list_containers(env))?;
    if docker::container_from_prob_exists(&container_list, &problem) {
        Err(SomaError::RepositoryInUse)?;
    }

    runtime.block_on(docker::remove_image(
        env,
        &problem.docker_image_name(env.username()),
    ))?;
    env.printer().write_line(&format!(
        "Problem image cleaned: '{}'",
        problem.fully_qualified_name()
    ));

    Ok(())
}

pub fn stop(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;

    let container_list = runtime.block_on(docker::list_containers(env))?;
    if !docker::container_from_prob_exists(&container_list, &problem) {
        Err(SomaError::ProblemNotRunning)?;
    }

    let container_list = docker::containers_from_prob(container_list, &problem);
    let states_to_stop = &["paused", "restarting", "running"];

    let containers_to_stop = container_list
        .iter()
        .filter(|container| states_to_stop.contains(&container.container().state.as_str()));

    for container in containers_to_stop {
        runtime.block_on(docker::stop(env, &container.container().id))?;
    }

    for container in container_list {
        runtime.block_on(docker::remove_container(env, &container.container().id))?;
    }

    env.printer().write_line(&format!(
        "Problem stopped: '{}'",
        problem.fully_qualified_name()
    ));

    Ok(())
}

pub fn update(
    env: &Environment<impl Connect, impl Printer>,
    repo_name: &NameString,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let mut repository = env.repo_manager().get_repo(repo_name)?;
    repository.update(&runtime.block_on(docker::list_images(env))?)?;
    env.printer()
        .write_line(&format!("Repository updated: '{}'", repo_name));

    Ok(())
}
