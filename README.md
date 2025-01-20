# lobster-rust <!-- omit in toc -->

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://choosealicense.com/licenses/mit/)
[![License](https://img.shields.io/badge/license-Apache2.0-blue.svg)](https://choosealicense.com/licenses/apache-2.0/)
[![Repo Status](https://www.repostatus.org/badges/latest/active.svg)](https://www.repostatus.org/#active)

Rust tool to process rust projects for BMWs [lobster](https://github.com/bmw-software-engineering/lobster).

This tool processes rust source code with lobster annotations into the lobster intermediate format.

**!!!** lobster-rust is currently in development and does not meet the feature set and maturity of the lobster tools for languages like python or C++. To get an overview of the current state and feature set, check out the issues tab **!!!**

* [Used Libraries](#used-libraries)
* [Issues, Ideas And Bugs](#issues-ideas-and-bugs)
* [License](#license)
* [Contribution](#contribution)

## Used Libraries

Used 3rd party libraries which are not part of the standard Python package:

* [ra_ap_syntax](https://crates.io/crates/ra_ap_syntax) - Syntax Tree crate of the rust-analyzer - MIT or Apache-2.0 License
* [ra_ap_edition](https://crates.io/crates/ra_ap_edition) - Rust Edition Enum of the rust-analyzer - MIT or Apache-2.0 License
* [json](https://crates.io/crates/json) - json implementation for Rust - MIT or Apache-2.0 License
* [regex](https://crates.io/crates/regex) - regular expressions for Rust - MIT or Apache-2.0 License

## Issues, Ideas And Bugs

If you have further ideas or you found some bugs, great! Create a issue or if you are able and willing to fix it by yourself, fork the repository and create a pull request.

## License

The lobster-rust source code is published under [BSD-3-Clause](https://github.com/NewTec-GmbH/lobster-rust/blob/main/LICENSE).
Consider the different licenses of the used third party libraries too!

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, shall be licensed as above, without any additional terms or conditions.
