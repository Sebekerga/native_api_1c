//! Crate for working with 1C:Enterprise Native API. It contains
//! low level FFI, moved from original C++ implementation, and high level
//! interface for working with Native API from RUST. It attempts to be as
//! close to original C++ implementation as possible, but some changes are
//! made to make it more idiomatic in RUST.
//!
//! While it is possible to use this crate to implement your Native API
//! Component, it is intended to be used with native_api_1c crate.

/// Module for implementations of Native API FFI
pub mod ffi;
/// Module for high level interface of Native API
pub mod interface;
