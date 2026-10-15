use std::sync::Arc;

use native_api_1c::native_api_1c_core::{
    ffi::connection::Connection,
    interface::ParamValue,
};
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunction", name_ru = "МояФункция")]
    #[arg(ty = Any, default = ParamValue::Empty)]
    #[returns(ty = Any)]
    pub my_function: fn(&Self, ParamValue) -> ParamValue,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            my_function: Self::my_function_inner,
        }
    }

    fn my_function_inner(&self, arg: ParamValue) -> ParamValue {
        arg
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
