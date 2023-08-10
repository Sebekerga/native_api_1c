use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c::native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_prop(name = "MyProp", name_ru = "МоеСвойство", readable, writable)]
    pub some_prop: i32,
    #[add_in_prop(name = "ProtectedProp", name_ru = "ЗащищенноеСвойство", readable)]
    pub protected_prop: i32,
    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    pub my_function: fn(&Self, i32) -> i32,
    #[add_in_func(name = "SumOfTwo", name_ru = "СуммаДвух")]
    pub sum_of_2: fn(&Self, i32, i32) -> String,

    private_field: i32,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            some_prop: 0,
            protected_prop: 50,
            my_function: Self::my_function,
            sum_of_2: Self::sum_of_2,
            private_field: 100,
        }
    }

    fn my_function(&self, arg: i32) -> i32 {
        self.protected_prop + self.some_prop + arg + self.private_field
    }

    fn sum_of_2(&self, arg1: i32, arg2: i32) -> String {
        format!("{}", arg1 + arg2)
    }
}
