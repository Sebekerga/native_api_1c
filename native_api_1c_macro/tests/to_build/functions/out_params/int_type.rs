use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunctionMut", name_ru = "МояФункцияМут")]
    #[arg(Int, as_in)]
    #[arg(Int, as_out)]
    #[returns(Bool)]
    pub my_function_mut: fn(&mut Self, i32, &mut i32) -> bool,

    #[add_in_func(name = "MyFunctionRef", name_ru = "МояФункцияРеф")]
    #[arg(Int, as_in)]
    #[arg(Int, as_out)]
    #[returns(Bool)]
    pub my_function_ref: fn(&Self, i32, &mut i32) -> bool,

    #[add_in_func(name = "MyFunctionNoRef", name_ru = "МояФункцияБезРеф")]
    #[arg(Int, as_in)]
    #[arg(Int, as_out)]
    #[returns(Bool)]
    pub my_function_no_ref: fn(i32, &mut i32) -> bool,
}

impl MyAddIn {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(None),
            my_function_mut: Self::my_function_mut_inner,
            my_function_ref: Self::my_function_ref_inner,
            my_function_no_ref: Self::my_function_no_ref_inner,
        }
    }

    fn my_function_mut_inner(&mut self, in_arg: i32, out_arg: &mut i32) -> bool {
        *out_arg = in_arg * 2;
        true
    }

    fn my_function_ref_inner(&self, in_arg: i32, out_arg: &mut i32) -> bool {
        *out_arg = in_arg * 2;
        true
    }

    fn my_function_no_ref_inner(in_arg: i32, out_arg: &mut i32) -> bool {
        *out_arg = in_arg * 2;
        true
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
