```rust
use std::sync::Arc;

use native_api_1c::{
    native_api_1c_core::ffi::connection::Connection,
    native_api_1c_macro::{extern_functions, AddIn},
};

#[derive(AddIn)]
pub struct MyAddIn {
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
    ///  ComponentObject.MyFunction(10, 15); // 2nd argument = 15
    ///  ComponentObject.MyFunction(10);     // 2nd argument = 12 (default value)
    /// ```
    /// If function returns an error, but does not panic, then 1C will throw an exception
    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(ty = Int)]
    #[arg(ty = Int, default = 12)]
    #[returns(ty = Int, result)]
    pub my_function: fn(&Self, i32, i64) -> Result<i32, ()>,

    /// Function, taking no arguments and returning nothing
    #[add_in_func(name = "MyProcedure", name_ru = "МояПроцедура")]
    #[returns(ty = Str)]
    pub my_procedure: fn(&mut Self) -> String,

    /// Private field, not visible from 1C
    private_field: i32,
}

impl Default for MyAddIn {
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

impl MyAddIn {
    fn my_function_inner(&self, arg: i32, arg_maybe_default: i64) -> Result<i32, ()> {
        Ok(self.protected_prop
            + self.some_prop
            + arg
            + self.private_field
            + arg_maybe_default as i32)
    }

    fn my_procedure_inner(&mut self) -> String {
        self.protected_prop += 1;
        "Some string from rust".to_string()
    }
}

extern_functions! {
    MyAddIn::default(),
}
```