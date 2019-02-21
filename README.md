# Soma: Your one-stop CTF problem management tool

[![Build Status](https://dev.azure.com/plus-postech/soma/_apis/build/status/PLUS-POSTECH.soma?branchName=master)](https://dev.azure.com/plus-postech/soma/_build/latest?definitionId=1?branchName=master)

- [What is Soma?](#what-is-soma-)
  * [For problem solvers](#for-problem-solvers)
  * [For problem setters](#for-problem-setters)
  * [Dependency](#dependency)
- [Current Status](#current-status)
  * [Roadmap](#roadmap)
- [How to Use](#how-to-use)
  * [Installation](#installation)
  * [Command overview](#command-overview)
  * [Adding repositories](#adding-repositories)
  * [Building problem images](#building-problem-images)
  * [Running problems](#running-problems)
  * [Fetching problem attachments](#fetching-problem-attachments)
  * [Stopping problems](#stopping-problems)
  * [Removing problem images](#removing-problem-images)
  * [Removing repositories](#removing-repositories)
  * [Notes on repository and problem names](#notes-on-repository-and-problem-names)
    + [Problem query](#problem-query)
    + [Name rules](#name-rules)
- [How to Add Soma Support to Your Repository](#how-to-add-soma-support-to-your-repository)
  * [`soma.toml` syntax](#-somatoml--syntax)
    + [The root section](#the-root-section)
      - [The `name` field](#the--name--field)
      - [The `work_dir` field (optional)](#the--work-dir--field--optional-)
    + [The `[binary]` section](#the---binary---section)
      - [The `os` field](#the--os--field)
      - [The `cmd` field](#the--cmd--field)
      - [File entries](#file-entries)
        * [The `path` field](#the--path--field)
        * [The `target_path` field (optional)](#the--target-path--field--optional-)
        * [The `public` field (optional)](#the--public--field--optional-)
    + [Other subconfigurations](#other-subconfigurations)
  * [`soma-list.toml` syntax](#-soma-listtoml--syntax)
    + [The `problems` field](#the--problems--field)
- [Development](#development)
  * [Prerequisites](#prerequisites)
  * [Testing, Building, and Running](#testing--building--and-running)
- [License](#license)
  * [Contribution](#contribution)


## What is Soma?

Soma is a cross-platform CTF problem management tool.

Soma helps to manage and distribute CTF problems after contests.


### For problem solvers

Downloading and running a problem is as easy as running three commands.

```bash
$ soma add https://github.com/PLUS-POSTECH/simple-bof.git
$ soma build simple-bof
$ soma run simple-bof 31337
```

CTF problems often contain public files. You can also fetch them easily with soma.

```bash
# this will copy public files to the current working directory
$ soma fetch simple-bof
```


### For problem setters

Add `soma.toml` that describes your problem in your project's root directory. The config file below shows an example of it.

```toml
name = "simple-bof"

[binary]
os = "ubuntu:16.04"
cmd = "./simple-bof"

[[binary.executable]]
path = "build/simple-bof"
public = true

[[binary.readonly]]
path = "flag"
```

That's all! Soma gets enough information to run your binary from these 12 lines of configuration.

Soma will use reasonable default value for the other things not specified such as
default working directory, file permissions, fork daemon, and standard stream buffering.
Of course they can be manually configured if needed :)

If your project contains more than one problems, you can put `soma-list.toml` that lists each problem directory.

```toml
problems = [
    # these subdirectories contain soma.toml files
    "problem1",
    "sub-dir/problem2"
]
```

### Dependency

Soma requires [Docker][docker] to be installed on the system.

## Current Status

Soma is in **pre-alpha** stage. Currently, Soma does not have any stable release, and everything is subject to change.

The initial 0.1.0 release will contain the features listed in the [issue #4] and [issue #66].
Issues related to 0.1.0 release are marked with [`0.1.0` milestone].

Soma team is hoping to ship the alpha version in the first quarter of 2019.


### Roadmap

- Better documentation of features. (priority: high)
- Support multiple containers for a single problem. (priority: low)
- Support cloud deployment such as AWS, GCP, Azure as well as local deployment. (priority: low)


## How to Use

### Installation

As we don't provide a pre-compiled binary yet, you should install [Rust][rust-lang] toolchain and build your own binary from the source. In detail, clone this git repository and run `cargo install`.

```bash
$ git clone https://github.com/PLUS-POSTECH/soma.git
$ cd soma
$ cargo install
```

We are expecting to release `0.1.0-alpha` soon to [crates.io].

### Command overview


|          | Add / Create | Remove |
| -------- | ------------ | ------ |
| Repository | [add](#adding-repositories) | [remove](#removing-repositories) |
| Image | [build](#building-problem-images) | [clean](#removing-problem-images) |
| Container | [run](#running-problems) | [stop](#stopping-problems) |


### Adding repositories

Soma repository has `soma.toml` or `soma-list.toml` in its top level directory and can contain one or more problems. To use Soma, start by adding problem repositories. We will use [`soma-bata-list`][soma-bata-list] as an example through out this document.

You can add a repository with `add` subcommand:

```bash
$ soma add https://github.com/PLUS-POSTECH/soma-bata-list.git
```

`add` subcommand takes a repository source as an argument, which is `https://github.com/PLUS-POSTECH/soma-bata-list.git` in the command above. Soma supports the following repository sources:

- Git repository (only HTTPS without authentication is supported for now)

  e.g., `https://github.com/PLUS-POSTECH/soma-bata-list.git`

- Local file system directory

  e.g., `/home/linux-user/soma-repo`

By default, Soma will parse the name of the repository from the repository source string. For example, `https://github.com/PLUS-POSTECH/soma-bata-list.git` will return `soma-bata-list` and `/home/linux-user/soma-repo` will return `soma-repo` as a default repository name. If you want to use another name, you can use `--name [NAME]` flag. This flag will make Soma register the repository under the given name.

Also, note that there are a few requirements for repository names. See "[Name rules](#name-rules)" section for more details.


### Building problem images

Soma uses [Docker][docker] to manage problem images and containers. An image is a snapshot of a problem environment, and a container is an instance of an image. In order to run a problem, you should build the problem image first.

Soma will automatically generate Dockerfile from the problem manifest when you execute the build command. You can build a problem image by executing the following command:

```bash
$ soma build r0pbaby
# or
$ soma build soma-bata-list.r0pbaby
```

As you can see, there are two different ways to select a problem. See "[Problem query](#problem-query)" section for more details.


### Running problems

When the problem image is ready, you can run the problem container! The problem container can be started with the following command:

```bash
$ soma run r0pbaby 13337
# or
$ soma run soma-bata-list.r0pbaby 13337
```

Here, `13337` indicates the port number which binds to the problem container. This port number will expose the problem to the host network. In this example, `r0pbaby` is accessible through `your.host.address:13337`. Try `nc localhost 13337` on your machine to start solving the problem.

`run` command requires a port number for now, but we are planning to support automatic port binding in the future (see [#64][issue #64]).


### Fetching problem attachments

CTF problems often provide a few attachments (usually problem binaries). There is a dedicated subcommand to fetch these files to your current working directory:

```bash
$ soma fetch r0pbaby
# or
$ soma fetch soma-bata-list.r0pbaby
```


### Stopping problems

After finish solving a problem, stop and remove the problem container by invoking:

```bash
$ soma stop r0pbaby
# or
$ soma stop soma-bata-list.r0pbaby
```


### Removing problem images

You may want to remove existing problem images for several reasons (e.g., free disk space, remove repository). You can clean up existing images by:

```bash
$ soma clean r0pbaby
# or
$ soma clean soma-bata-list.r0pbaby
```

Before trying to remove the image, check if any associated container exists. Soma will reject the command if there is a running container from the image.


### Removing repositories

If you want to remove a registered repository, use the following command:

```bash
$ soma remove soma-bata-list
```

There should be no problem image or container associated to the repository when you use this command. Use `clean` and `stop` command to remove them if necessary. Auto pruning for your convenience will be implemented in the future (see [#115][issue #115]).


### Notes on repository and problem names

#### Problem query

There are two ways to identify a problem. You can specify only the name of a problem (e.g., `r0pbaby`) or the full name of a problem containing what repository it is in (e.g., `soma-bata-list.r0pbaby`).

If the problem name is unique (i.e., the problem name is used only in a single repository), you can use the former method. Otherwise, you must use the latter or Soma will give you an error message that the problem name is found among multiple repositories.


#### Name rules

All repository and problem names should follow the docker name component rules with no `.` (i.e., `^[a-z0-9]+((?:_|__|[-]*)[a-z0-9]+)*$`, see [Docker regexp definitions][docker-regexp] for more details). This measure is to prevent Soma from behaving bad when malicious input is provided. We chose docker name component rules as repository and problem names are substrings of Docker image name. And 'no `.`' rule is added because Soma utilizes `.` as a name separator.


## How to Add Soma Support to Your Repository

In order to support Soma, a repository should have `soma.toml` or `soma-list.toml` in its top level directory. `soma-list.toml` lists each subdirectory that contains a `soma.toml` file, and `soma.toml` is the name of Soma problem manifest file.

If your repository contains only one problem in its top level directory, `soma.toml` can be used directly without `soma-list.toml`.


### `soma.toml` syntax

Each problem in a repository needs a manifest file, `soma.toml`, in its directory. The problem manifest file contains necessary information for Soma to manage problems. We will discuss the syntax of `soma.toml` section by section with this example.

```toml
name = "simple-bof"
work_dir = "/home/simple-bof"

[binary]
os = "ubuntu:16.04"
cmd = "./simple-bof"

[[binary.executable]]
path = "build/simple-bof"
public = true

[[binary.readonly]]
path = "flag"
target_path = "/you_cannot_guess_this_very_secret_flag_name"
```

#### The root section

Manifest root contains two metadata for the problem.

```toml
name = "simple-bof"
work_dir = "/home/simple-bof"
```

##### The `name` field

The `name` field of the root section defines the name of the problem. Soma will recognize this field (not the directory name) as the problem name.

##### The `work_dir` field (optional)

The `work_dir` field of the root section contains the path of the working directory inside the problem image. Default value for this field is the home directory of the user whose name is same with the name of the problem (for the example above, `"/home/simple-bof"`).

#### The `[binary]` section

The `[binary]` section contains information required to use binary subconfiguration. Binary subconfiguration supports a scenario which runs an executable and pipes standard input and output through a TCP connection with a fork daemon; this is one of the most common setups in CTF competitions.

```toml
[binary]
os = "ubuntu:16.04"
cmd = "./simple-bof"

[[binary.executable]]
path = "build/simple-bof"
public = true

[[binary.readonly]]
path = "flag"
target_path = "/you_cannot_guess_this_very_secret_flag_name"
```

##### The `os` field

The `os` field indicates what OS flavor is used by the problem. This field is currently redirected into the parent image name of `Dockerfile`. However, it will only allow pre-selected choices in the future.

##### The `cmd` field

The `cmd` field defines how to run the problem binary. The specified binary will be executed through [socat](https://linux.die.net/man/1/socat) daemon.

##### File entries

`[[binary.executable]]`, `[[binary.readonly]]` sections contain file entries of the subconfiguration.

`[[binary.readwrite]]`, `[[binary.with-permissions]]`, and `[[binary.fetchonly]]` are reserved for a future implementation (see [#50][issue #50] and [#84][issue #84]).

###### The `path` field

The `path` field contains a relative path to the file from the problem directory. Supporting external sources such as URL is planned in the future (see [#114][issue #114]).

###### The `target_path` field (optional)

The `target_path` field specifies where the file should be copied inside the problem image. This field defaults to `work_dir` in the root section.

###### The `public` field (optional)

File entries with `public` field set to `true` will be copied to the current working directory when users invoke `fetch` subcommand. This field has a default value of `false`.

#### Other subconfigurations

Other subconfigurations for common CTF setups such as `apache-php7`, `python-uwsgi`, or `mysql` are planned to be supported in a future release (see [#50][issue #50]). Subconfiguration syntax is designed to support multi-configuration problem in the future, which will be handled similarly to [Docker compose][docker-compose].


### `soma-list.toml` syntax

#### The `problems` field

`soma-list.toml` contains a single field `problems` which is an array of relative paths to each problem directory.

```toml
problems = [
    # these subdirectories contain soma.toml files
    "simple-bof",
    "hard/complicated-bof"
]
```


## Development

### Prerequisites

* Install [Rust][rust-lang] stable toolchain.
* Install `openssl` (Required by `openssl-sys` crate).
* Install `clippy` and `rustfmt`.
    * `rustup component add clippy rustfmt`
* Copy files in `hooks` directory to `.git/hooks`.


### Testing, Building, and Running

Soma is written with Rust and utilizes Cargo as a building and testing system.

You can test, build, and run with the following command.

```bash
$ cargo test
$ cargo build
$ cargo run
```


## License

Licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.


[issue #4]: https://github.com/PLUS-POSTECH/soma/issues/4
[issue #50]: https://github.com/PLUS-POSTECH/soma/issues/50
[issue #64]: https://github.com/PLUS-POSTECH/soma/issues/64
[issue #66]: https://github.com/PLUS-POSTECH/soma/issues/66
[issue #84]: https://github.com/PLUS-POSTECH/soma/issues/84
[issue #114]: https://github.com/PLUS-POSTECH/soma/issues/114
[issue #115]: https://github.com/PLUS-POSTECH/soma/issues/115
[`0.1.0` milestone]: https://github.com/PLUS-POSTECH/soma/milestone/1

[crates.io]: https://crates.io/
[docker]: https://www.docker.com/
[docker-compose]: https://docs.docker.com/compose/
[docker-regexp]: https://github.com/docker/distribution/blob/master/reference/regexp.go
[rust-lang]: https://www.rust-lang.org/
[soma-bata-list]: https://github.com/PLUS-POSTECH/soma-bata-list
