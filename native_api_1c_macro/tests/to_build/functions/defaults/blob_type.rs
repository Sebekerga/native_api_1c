use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(Blob, default = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9])]
    #[returns(Blob)]
    pub my_function: fn(&Self, Vec<u8>) -> Vec<u8>,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            my_function: Self::my_function_inner,
        }
    }

    fn my_function_inner(&self, arg: Vec<u8>) -> Vec<u8> {
        arg
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
