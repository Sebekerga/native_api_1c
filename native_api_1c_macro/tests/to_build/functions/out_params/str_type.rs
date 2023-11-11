use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunctionMut", name_ru = "МояФункцияМут")]
    #[arg(ty = Str, as_in)]
    #[arg(ty = Str, as_out)]
    #[returns(ty = Bool)]
    pub my_function_mut: fn(&mut Self, String, &mut String) -> bool,

    #[add_in_func(name = "MyFunctionRef", name_ru = "МояФункцияРеф")]
    #[arg(ty = Str, as_in)]
    #[arg(ty = Str, as_out)]
    #[returns(ty = Bool)]
    pub my_function_ref: fn(&Self, String, &mut String) -> bool,

    #[add_in_func(name = "MyFunctionNoRef", name_ru = "МояФункцияБезРеф")]
    #[arg(ty = Str, as_in)]
    #[arg(ty = Str, as_out)]
    #[returns(ty = Bool)]
    pub my_function_no_ref: fn(String, &mut String) -> bool,
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

    fn my_function_mut_inner(&mut self, in_arg: String, out_arg: &mut String) -> bool {
        *out_arg = format!("in was: {}", in_arg);
        true
    }

    fn my_function_ref_inner(&self, in_arg: String, out_arg: &mut String) -> bool {
        *out_arg = format!("in was: {}", in_arg);
        true
    }

    fn my_function_no_ref_inner(in_arg: String, out_arg: &mut String) -> bool {
        *out_arg = format!("in was: {}", in_arg);
        true
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
