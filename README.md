# lobster-rust <!-- omit in toc -->

[![License](https://img.shields.io/badge/License-BSD-3.svg)](https://github.com/NewTec-GmbH/lobster-rust/blob/main/LICENSE)
[![Repo Status](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Rust tool to process Rust projects for BMWs [lobster](https://github.com/bmw-software-engineering/lobster).

This tool processes Rust source code with lobster annotations / tracing tags into the lobster common interchange format.

lobster-rust is currently in development and does not meet the full planned feature set and maturity.

Currently supported features are:
* Extraction of tracing tags in Rust source code.
* Output in the lobster common interchange format.

Currently NOT supported is:
* Special parsing of test code.

To get an overview of what is currently worked on, check out the issues tab.

* [Used Libraries](#used-libraries)
* [Documentation](#documentation)
* [Issues, Ideas And Bugs](#issues-ideas-and-bugs)
* [License](#license)
* [Contribution](#contribution)

## Used Libraries

Used 3rd party libraries which are not part of the standard Rust package:

| Crate | Description | License |
| :---: | ----------- | ------- |
| [ra_ap_syntax](https://crates.io/crates/ra_ap_syntax) | Syntax Tree crate of the rust-analyzer | MIT or Apache-2.0 License |
| [ra_ap_edition](https://crates.io/crates/ra_ap_edition) | Rust Edition Enum of the rust-analyzer | MIT or Apache-2.0 License |
| [json](https://crates.io/crates/json) | Json implementation for Rust | MIT or Apache-2.0 License |
| [regex](https://crates.io/crates/regex) | Regular expressions for Rust | MIT or Apache-2.0 License |
| [clap](https://crates.io/crates/clap) | Command line argument parser for Rust | MIT or Apache-2.0 License |

## Installation

To build lobster-rust yourself, you will need to have Rust and Cargo [installed](https://doc.rust-lang.org/cargo/getting-started/installation.html). Clone the repository and build the project using cargo via ```cargo build --release```.
This will build a lobster-rust executable in the ```./target/release/``` directory.

For prebuild releases, see the [Release](https://github.com/NewTec-GmbH/lobster-rust/releases) page of the repository.

## Usage

lobster-rust is a CLI tool and is configured via command line arguments.

```cmd
> lobster-rust.exe --help
Usage: lobster-rust.exe [OPTIONS] [DIR] [OUT]

Arguments:
  [DIR]  Directory of main.rs (or lib.rs) [default: ./src/]
  [OUT]  Output directory for the .lobster file [default: rust.lobster]

Options:
  -l, --lib                    Parse lib.rs as project root instead of main.rs
      --activity               Generate activity traces (tests) instead of an implementation trace. UNSUPPORTED
      --only-tagged-functions  Only trace functions with tags. UNSUPPORTED
  -h, --help                   Print help
  -V, --version                Print version
```

Because of sensible defaults, a simple cargo project should require no flags at all. lobster-rust expects a main.rs (or lib.rs with the --lib flag) in ```./src/```. Any submodules included are resolved by lobster-rust itself.

A rust.lobster output file in the common interchange format (json based) is created in the cwd.

## Documentation

The code is fully covered with doc comments, allowing the creation of extensive documentation via ```cargo doc```.
Detailed documentation about the tool can be found in the [doc/](https://github.com/NewTec-GmbH/lobster-rust/tree/main/doc) folder.

The full source code shall be compliant to [the Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/).

## Issues, Ideas And Bugs

If you have further ideas or you found some bugs, great! Create an [issue](https://github.com/NewTec-GmbH/lobster-rust/issues) or if you are able and willing to fix it by yourself, fork the repository and create a pull request.

## License

The lobster-rust source code is published under [BSD-3-Clause](https://github.com/NewTec-GmbH/lobster-rust/blob/main/LICENSE).

Consider the different licenses of the used third party libraries too!

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
