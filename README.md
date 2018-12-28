# Soma

[![Build Status](https://dev.azure.com/plus-postech/soma/_apis/build/status/PLUS-POSTECH.soma?branchName=master)](https://dev.azure.com/plus-postech/soma/_build/latest?definitionId=1?branchName=master)

Your one-stop CTF problem management tool


## Testing, Building, and Running

Owl is written with Rust, and utilizes Cargo as a building and testing system.

You can test, build, run using the following command:

```
cargo test
cargo build
cargo run
```


## Development

* Install Rust stable toolchain.
* Install `openssl` (Required by `openssl-sys` crate).
* Install `rustfmt`.
    * `rustup component add rustfmt`
* Copy files in `hooks` directory to `.git/hooks`.


## License

Licensed under either of
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.


### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
