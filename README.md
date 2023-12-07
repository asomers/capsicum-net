# capsicum-net

Rust bindings to FreeBSD's [cap_net(3)] library.  `cap_net` allows access to
several network APIs that are forbidden in capability mode by delegating them to
an unsandboxed process, the Casper daemon.

![Build Status](https://api.cirrus-ci.com/github/asomers/capsicum-net.svg)](https://cirrus-ci.com/github/asomers/capsicum-net)
[![Crates.io](https://img.shields.io/crates/v/capsicum-net.svg)](https://crates.io/crates/capsicum-net)

[Documentation](https://docs.rs/crate/capsicum-net)

[cap_net(3)]: https://man.freebsd.org/cgi/man.cgi?query=cap_net

# Usage

See the examples in the API docs.  The general idea is to create the `Casper`
and `CapNetAgent` objects when your program first starts up.  Then, use
functions like `CapNetAgent::bind` instead of `std::net::UdpSocket::bind`.
There are three APIs available:

* Low-level methods which operate directly on the `CapNetAgent` object.  These
  work well with the [nix](https://docs.rs/nix/0.27.1/nix/) crate.
* Extension traits that work on the standard socket types.
* Extension traits that work with tokio types.  These require the crate to be
  built with the `tokio` feature.

# Platforms

This crate only works on FreeBSD.  At least, until somebody ports `cap_net` to a
different operating system.

# Minimum Supported Rust Version (MSRV)

`capsicum-net` does not guarantee any specific MSRV.  Rather, it guarantees
compatibility with the oldest rustc shipped in the FreeBSD package collection.

* https://www.freshports.org/lang/rust/

# License

`capsicum-net` is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See LICENSE-APACHE, and LICENSE-MIT for details.

# Sponsorship

`capsicum-net` is sponsored by Axcient, inc.
