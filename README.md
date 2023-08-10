# Описание
Библиотека для простой реализации внешней компоненты для 1С на чистом Rust, основано на примере, созданным [пользователем **medigor**](https://github.com/medigor/example-native-api-rs)

Библиотека делится на два подмодуля:
- `native_api_1c_core` описывает все необходимое для реализации ВК
- `native_api_1c_macro` предоставляет инструмент для значительного упрощения описания компоненты, беря на себя реализацию свойства `native_api_1c_core::interface::AddInWrapper`

Пример реализации простой компоненты:

```rust
// lib.rs
use std::sync::Arc;

use native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>, // Arc для возможности многопоточности

    #[add_in_prop(name = "MyProp", name_ru = "МоеСвойство", readable, writable)]
    pub some_prop: i32,
    #[add_in_prop(name = "ProtectedProp", name_ru = "ЗащищенноеСвойство", readable)]
    pub protected_prop: i32,
    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    pub my_function: fn(&Self, i32) -> i32,

    private_field: i32,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            some_prop: 0,
            protected_prop: 50,
            my_function: Self::my_function,
            private_field: 100,
        }
    }

    fn my_function(&self, arg: i32) -> i32 {
        self.protected_prop + self.some_prop + arg + self.private_field
    }
}
```