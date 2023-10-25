This is a fork of [medigor/example-native-api-rs](https://github.com/medigor/example-native-api-rs) that is made to be a core crate for 1C:Enterprise 8 Native API development. As of this moment, crate is tested on Linux and Windows. It should work on MacOS as well, but it is not tested. 

It implements FFI for Native API components and provides a set of types and `AddInWrapper` trait that can be used to implement 1C:Enterprise 8 Native API components in Rust. While it can be used as a standalone crate, it is intended to be used as a dependency for [native_api_1c](https://github.com/sebekerga/native_api_1c) crate.

>_For FFI implementation, see [original repository](https://github.com/medigor/example-native-api-rs) or [this issue discussion](https://github.com/Sebekerga/native_api_1c/issues/2)_

Aside from some features (especially on Connection interface) not yet implemented, this crate should cover most important of the Native API functionality.