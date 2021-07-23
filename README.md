# Crate dygpi

Provides support for _Dynamic Generic PlugIns_, library based plugins for Rust.

![MIT License](https://img.shields.io/badge/license-mit-118811.svg)
![Minimum Rust Version](https://img.shields.io/badge/Min%20Rust-1.50-green.svg)
[![crates.io](https://img.shields.io/crates/v/dygpi.svg)](https://crates.io/crates/dygpi)
[![docs.rs](https://docs.rs/dygpi/badge.svg)](https://docs.rs/dygpi)
![Build](https://github.com/johnstonskj/rust-dygpi/workflows/Rust/badge.svg)
![Audit](https://github.com/johnstonskj/rust-dygpi/workflows/Security%20audit/badge.svg)
[![GitHub stars](https://img.shields.io/github/stars/johnstonskj/rust-dygpi.svg)](https://github.com/johnstonskj/rust-dygpi/stargazers)

-----

# Example

TBD

-----

## Changes

**Version 0.1.5**

* Changed the PluginManager API to take Path and PathBuf values not strings for library names.
 
**Version 0.1.4**

* Added public function to make dylib file names.
* Reworked Github action and test cases for non-macos platforms.

**Version 0.1.3**

* Moved the PluginRegistrar struct from the manager to plugin module; this seems cleaner from a client perspective.

**Version 0.1.2**

* Added the ability to override the registration function name, this allows for multiple plugin types in a single host.
* Fixed the E2E example, added a simple run script.

**Version 0.1.1**

* Internal change to use search_path crate for library resolution.
* Added Github action config.

**Version 0.1.0**

* Initial commit.
