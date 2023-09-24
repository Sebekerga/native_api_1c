>Гайд по использованию на русском языке можно посмотреть [здесь](https://infostart.ru/1c/articles/1920565/) и задать вопросы по использованию, но не оставляйте там комментарии об ошибках, т.к. там сложно это обсуждать. Лучше создайте issue в этом репозитории.

Library for simple 1C:Enterprise platform Native API Component development, originates from findings of this [medigor/example-native-api-rs](https://github.com/medigor/example-native-api-rs)

Crate is tested on Linux and Windows. It should work on MacOS as well, but it is not tested.

# Structure
Library is divided into two submodules:
- `native_api_1c_core` describes all necessary for implementing 1C:Enterprise Native API
- `native_api_1c_macro` provides a tool for significant simplification of component implementation, taking care of `native_api_1c_core::interface::AddInWrapper` property implementation

# Usage

## Attributes `#[add_in_prop(...)]`
- `name` - property name in 1C
- `name_ru` - property name in 1C in Russian
- `readable` - property is readable from 1C
- `writable` - property is writable from 1C

Available property types: `i32`, `f64`, `bool`, `String`

## Functions or procedures `#[add_in_func(...)]`
- `name` - property name in 1C
- `name_ru` - property name in 1C in Russian
### Input arguments, `#[arg(...)]`, for each type of argument must be set, on of:
- `Int` - `i32`
- `Float` - `f64`
- `Bool` - `bool`
- `Str` - `String`
- `Date` - `chrono::DateTime<chrono::FixedOffset>`
- `Blob` - `Vec<u8>`
### Return values, `#[returns(...)]`, type must be set, one of:
- `Int` - `i32`
- `Float` - `f64`
- `Bool` - `bool`
- `Str` - `String`
- `Date` - `chrono::DateTime<chrono::FixedOffset>`
- `Blob` - `Vec<u8>`
- `None` - `()`
Additionally, `Result<T, ()>` can be used, where `T` is one of the above. In this case, `result` must be set in `#[returns(...)]` attribute: `#[returns(Int, result)]` for `Result<i32, ()>`

## Example

```toml
# Cargo.toml
[package]
name = "my_addin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
utf16_lit = "2.0"
native_api_1c = "0.10.3"
```

```rust
// src/lib.rs
use std::sync::Arc;

use native_api_1c::{native_api_1c_core::ffi::connection::Connection, native_api_1c_macro::AddIn};

#[derive(AddIn)]
pub struct MyAddIn {
    /// connection with 1C, used for calling events
    /// Arc is used to allow multiple threads to access the connection 
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>, 

    /// Property, readable and writable from 1C
    #[add_in_prop(name = "MyProp", name_ru = "МоеСвойство", readable, writable)]
    pub some_prop: i32,

    /// Property, readable from 1C but not writable
    #[add_in_prop(name = "ProtectedProp", name_ru = "ЗащищенноеСвойство", readable)]
    pub protected_prop: i32,

    /// Function, taking one or two arguments and returning a result
    /// In 1C it can be called as:
    ///  ComponentObject.MyFunction(10, 15); // 2nd argument = 15
    ///  ComponentObject.MyFunction(10);     // 2nd argument = 12 (default value)
    /// If function returns an error, but does not panic, then 1C will throw an exception 
    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(Int)]
    #[arg(Int, default = 12)]
    #[returns(Int, result)]
    pub my_function: fn(&Self, i32, i64) -> Result<i32, ()>,

    /// Function, taking no arguments and returning nothing 
    #[add_in_func(name = "MyProcedure", name_ru = "МояПроцедура")]
    pub my_procedure: fn(&mut Self),

    /// Private field, not visible from 1C
    private_field: i32,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            some_prop: 0,
            protected_prop: 50,
            my_function: Self::my_function,
            my_procedure: Self::my_procedure,
            private_field: 100,
        }
    }

    fn my_function(&self, arg: i32, arg_maybe_default: i64) -> Result<i32, ()> {
        Ok(self.protected_prop
            + self.some_prop
            + arg
            + self.private_field
            + arg_maybe_default as i32)
    }

    fn my_procedure(&mut self) {
        self.protected_prop += 1;
    }
}
```