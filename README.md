# archive-rs

A Rust crate that aims to provide a generic way of dealing with multiple archive and compression formats by providing a generic abstraction on top of existing Rust crates for specific formats.

This crate makes use of streaming/lending iterators and [GATs (Generic Associated Types)](https://blog.rust-lang.org/2021/08/03/GATs-stabilization-push.html) to provide such a generic interface, and as such this crate depends on Rust nightly, as these features are still work in progress.
However, through the use of these features, this crate can hopefully serve as an example and pinpoint any issues with these features in particular to get them a step closer to stabilization.

# File Formats

The following file formats are currently supported:

* [x] .cab (thanks to [cab](https://crates.io/crates/cab))
* [x] .tar (thanks to [tar](https://crates.io/crates/tar))
* [x] .tar.bz2 (thanks to [tar](https://crates.io/crates/tar) and [bzip2](https://crates.io/crates/bzip2))
* [x] .tar.gz (thanks to [tar](https://crates.io/crates/tar) and [flate2](https://crates.io/crates/flate2))
* [x] .tar.xz (thanks to [tar](https://crates.io/crates/tar) and [rust-lzma](https://crates.io/crates/rust-lzma))
* [x] .zip (thanks to [zip](https://crates.io/crates/zip))
