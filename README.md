>Гайд по использованию на русском языке можно посмотреть 
>[здесь](https://infostart.ru/1c/articles/1920565/) и задать вопросы по использованию, но не 
> оставляйте там комментарии об ошибках, т.к. там сложно это обсуждать. Лучше создайте issue в этом 
> репозитории.

# Disclaimer

This is my personal project, so there are some things coming from this fact:
- While I'm trying to implement everything in an idiomatic and 'pretty' way, sometimes I just want 
to see progress, so some cargo clippy warnings are ignored at times, but I always try to fix them 
later
- There'll be weeks or even months of inactivity, because I'm occupied with other things
- I'll try to help anyone, who opens issue or discussion, but I can't guarantee that I'll be able 
to do it in a timely manner

## Contributing
I'd be glad to see any contributions, but please, follow these rules:
- If you want to add a feature, please, open an issue first, so we can discuss it. I don't want you 
to waste your time on something that I won't be accepting for one reason or another
- If you want to fix a bug, better do the same, but if it's a small bug, you can just open a PR
- If you want to help, but don't know what to do, you can look at issues with `help wanted` label, 
or just ask [in this Telegram chat](https://t.me/+2YFbh4up3y8wZmIy)

# About

Library for simple 1C:Enterprise platform Native API Component development, originates from findings
of this [medigor/example-native-api-rs](https://github.com/medigor/example-native-api-rs)

Crate is tested on Linux and Windows. It should work on MacOS as well, but it is not tested.

## A word on testing

In order to test the actual FFI calls, you need to have 1C:Enterprise file base, which makes it hard
to test reliably with Rust tests, *and also makes it not free :)*. One alternative is to use 
[OneScript](https://github.com/EvilBeaver/OneScript), which can run 1C:Enterprise scripts, including
AddIn functions. However, it is not a perfect solution, because it is not 1C:Enterprise itself, and
**their implementations of Native API interfaces is not the same**
(See [this issue](https://github.com/EvilBeaver/OneScript/issues/1359) or try building and running
[this example](https://github.com/Sebekerga/native_api_1c_go))

# Structure
Library is divided into two submodules:
- `native_api_1c_core` describes all necessary for implementing 1C:Enterprise Native API
- `native_api_1c_macro` provides a tool for significant simplification of component implementation, 
taking care of `native_api_1c_core::interface::AddInWrapper` property implementation

# Usage

## Attributes `#[add_in_prop(...)]`
- `name` - property name in 1C
- `name_ru` - property name in 1C in Russian
- `readable` - property is readable from 1C
- `writable` - property is writable from 1C

Available property types: `i32`, `f64`, `bool`, `String`, `chrono::NaiveDateTime`, `Vec<u8>`,
`ParamValue` (for `Any`)

## Functions or procedures `#[add_in_func(...)]`
- `name` - property name in 1C
- `name_ru` - property name in 1C in Russian
### Input arguments, `#[arg(ty = ...)]`, for each type of argument must be set, on of:
| Type definition | Rust type               | 1C type                 |
|-----------------|-------------------------|-------------------------|
| `Int`           | `i32`                   | `Number` (Int)          |
| `Float`         | `f64`                   | `Number` (Float or Int) |
| `Bool`          | `bool`                  | `Boolean`               |
| `Str`           | `String`                | `String`                |
| `Date`          | `chrono::NaiveDateTime` | `Date`                  |
| `Blob`          | `Vec<u8>`               | `BinaryData`            |
| `Any`           | `ParamValue`            | any of the above        |

### Return values, `#[returns(ty = ...)]`, type must be set, one of:
| Type definition | Rust type               | 1C type      |
|-----------------|-------------------------|--------------|
| `Int`           | `i32`                   | `Number`     |
| `Float`         | `f64`                   | `Number`     |
| `Bool`          | `bool`                  | `Boolean`    |
| `Str`           | `String`                | `String`     |
| `Date`          | `chrono::NaiveDateTime` | `Date`       |
| `Blob`          | `Vec<u8>`               | `BinaryData` |
| `Any`           | `ParamValue`            | any of above |
| `None`          | `()`                    | `Undefined`  |

Additionally, `Result<T, ()>` can be used, where `T` is one of the above. In this case, `result`
must be set in `#[returns(...)]` attribute: `#[returns(Int, result)]` for `Result<i32, ()>`

### Dynamic values, `ty = Any`

`Any` is not a type of its own. `ParamValue` is an enum holding whichever value the platform sent,
so an `Any` argument, return value or property arrives and leaves with its 1C type intact. Match on
it in Rust to tell the types apart:

```rust
use native_api_1c::native_api_1c_core::interface::ParamValue;

#[add_in_func(name = "DescribeValue", name_ru = "ОписатьЗначение")]
#[arg(ty = Any)]
#[returns(ty = Any)]
pub describe_value: fn(&mut Self, ParamValue) -> ParamValue,

// ...

fn describe_value_inner(&mut self, value: ParamValue) -> ParamValue {
    match &value {
        ParamValue::Empty => { /* Undefined */ }
        ParamValue::Bool(v) => { /* Boolean */ }
        ParamValue::I32(v) => { /* Number, integer */ }
        ParamValue::F64(v) => { /* Number, fractional */ }
        ParamValue::Date(v) => { /* Date, as `Tm` */ }
        ParamValue::String(v) => { /* String, as UTF-16 */ }
        ParamValue::Blob(v) => { /* BinaryData */ }
    };
    value
}
```

A default value for an `Any` argument is any `ParamValue` expression, for instance
`#[arg(ty = Any, default = ParamValue::Empty)]`.

Native API does not support `VTYPE_INTERFACE` and `VTYPE_VARIANT`, so 1C objects (references, value
tables, structures) cannot be handed to a component at all. `Any` covers `Number`, `String`,
`Boolean`, `Date`, `BinaryData` and `Undefined` only - serialize objects on the 1C side if you need
to pass them. Note that `BinaryData` is not supported by the web client, and numbers sent by the web
client may arrive with any numeric `VTYPE`; those are normalized while converting to `ParamValue`.

### Errors and panics

Every callback the platform makes into the component runs behind a panic guard. A Rust panic
is caught at the FFI boundary, reported to 1C through `AddError` and turned into the failure
value the platform expects, instead of unwinding into the C++ frames of 1C and aborting the
process.

After the first panic the component is poisoned: its state is unknown once the stack has
unwound, so every later call fails without touching the add-in. Reconnect the component to get
a usable object again.

`extern_functions!` exports `SetPlatformCapabilities`, and that is what makes the reporting
above visible at all. Without the negotiation the platform withholds message output, so
`Connection::add_error` silently does nothing.

To report an error from your own code, use the connection kept in the `#[add_in_con]` field:

```rust
use native_api_1c::native_api_1c_core::ffi::connection::MessageCode;

fn my_function_inner(&self) -> Result<i32, ()> {
    let Some(connection) = *self.connection else {
        return Err(());
    };

    if !port_is_free() {
        connection.add_error(MessageCode::Fail, "MyAddIn", "Порт занят");
        return Err(());
    }

    Ok(42)
}
```

`MessageCode` mirrors the `ADDIN_E_*` codes of the 1C documentation: `Info`, `Attention`,
`Important`, `VeryImportant` and `Fail` put a line with the matching icon into the message
window, while `DialogAttention`, `DialogInfo` and `DialogFail` show a modal dialog. Returning
`Err(())` on top of that makes the platform raise a runtime error, catchable with
`Попытка ... Исключение`; note that `ОписаниеОшибки()` there holds the platform's own text,
while your `add_error` message goes to the message window.

> `catch_unwind` cannot help if the crate is built with `panic = "abort"` in
> `[profile.release]` - the process aborts before the guard sees anything. Keep the default
> `unwind` strategy for a crate that becomes an external component.

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
native_api_1c = "0.10.5"
```

```rust
// src/lib.rs
use std::sync::Arc;

use native_api_1c::{
    native_api_1c_core::ffi::connection::Connection,
    native_api_1c_macro::{extern_functions, AddIn},
};

#[derive(AddIn)]
pub struct SampleAddIn {
    /// connection with 1C, used for calling events
    /// Arc is used to allow multiple threads to access the connection
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    /// Property, readable and writable from 1C
    #[add_in_prop(ty = Int, name = "MyProp", name_ru = "МоеСвойство", readable, writable)]
    pub some_prop: i32,

    /// Property, readable from 1C but not writable
    #[add_in_prop(ty = Int, name = "ProtectedProp", name_ru = "ЗащищенноеСвойство", readable)]
    pub protected_prop: i32,

    /// Function, taking one or two arguments and returning a result
    /// In 1C it can be called as:
    /// ```bsl
    ///  CompObj.MyFunction(10, 15); // 2nd arg = 15
    ///  CompObj.MyFunction(10);     // 2nd arg = 12 (default value)
    /// ```
    /// If function returns an error, but does not panic, then 1C will throw an exception
    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(ty = Int)]
    #[arg(ty = Int, default = 12)] // default value for the second argument
    #[returns(ty = Int, result)]
    pub my_function: fn(&Self, i32, i64) -> Result<i32, ()>,

    /// Function, taking no arguments and returning nothing
    #[add_in_func(name = "MyProcedure", name_ru = "МояПроцедура")]
    pub my_procedure: fn(&mut Self),

    /// Private field, not visible from 1C
    private_field: i32,
}

impl Default for SampleAddIn {
    fn default() -> Self {
        Self {
            connection: Arc::new(None),
            some_prop: 0,
            protected_prop: 50,
            my_function: Self::my_function_inner,
            my_procedure: Self::my_procedure_inner,
            private_field: 100,
        }
    }
}

impl SampleAddIn {
    fn my_function_inner(&self, arg: i32, arg_maybe_default: i64) -> Result<i32, ()> {
        Ok(self.protected_prop
            + self.some_prop
            + arg
            + self.private_field
            + arg_maybe_default as i32)
    }

    fn my_procedure_inner(&mut self) {
        self.protected_prop += 10;
    }
}

extern_functions! {
    SampleAddIn::default(),
}
```

### Adding more objects

Method `extern_functions!` can take multiple objects, like this:
```rust
extern_functions! {
    SampleAddIn::default(),
    AnotherAddIn::default(),
    YetAnotherAddIn::default(),
}
```

These object must have trait `AddIn` implemented. This can be done either with `#[derive(AddIn)]`
or manually. Latter is useful when you need some unusual behaviors that cannot be derived.