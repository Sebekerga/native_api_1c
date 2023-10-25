use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunctionMut", name_ru = "МояФункцияМут")]
    #[arg(Blob, as_in)]
    #[arg(Blob, as_out)]
    #[returns(Bool)]
    pub my_function_mut: fn(&mut Self, Vec<u8>, &mut Vec<u8>) -> bool,

    #[add_in_func(name = "MyFunctionRef", name_ru = "МояФункцияРеф")]
    #[arg(Blob, as_in)]
    #[arg(Blob, as_out)]
    #[returns(Bool)]
    pub my_function_ref: fn(&Self, Vec<u8>, &mut Vec<u8>) -> bool,

    #[add_in_func(name = "MyFunctionNoRef", name_ru = "МояФункцияБезРеф")]
    #[arg(Blob, as_in)]
    #[arg(Blob, as_out)]
    #[returns(Bool)]
    pub my_function_no_ref: fn(Vec<u8>, &mut Vec<u8>) -> bool,
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

    fn my_function_mut_inner(&mut self, in_arg: Vec<u8>, out_arg: &mut Vec<u8>) -> bool {
        out_arg.extend_from_slice(&in_arg);
        true
    }

    fn my_function_ref_inner(&self, in_arg: Vec<u8>, out_arg: &mut Vec<u8>) -> bool {
        out_arg.extend_from_slice(&in_arg);
        true
    }

    fn my_function_no_ref_inner(in_arg: Vec<u8>, out_arg: &mut Vec<u8>) -> bool {
        out_arg.extend_from_slice(&in_arg);
        true
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
