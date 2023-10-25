use std::sync::Arc;

use native_api_1c::native_api_1c_core::ffi::connection::Connection;
use native_api_1c_macro::AddIn;

#[derive(AddIn)]
pub struct MyAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_func(name = "MyFunctionMut", name_ru = "МояФункцияМут")]
    #[arg(Float, as_in)]
    #[arg(Float, as_out)]
    #[returns(Bool)]
    pub my_function_mut: fn(&mut Self, f64, &mut f64) -> bool,

    #[add_in_func(name = "MyFunctionRef", name_ru = "МояФункцияРеф")]
    #[arg(Float, as_in)]
    #[arg(Float, as_out)]
    #[returns(Bool)]
    pub my_function_ref: fn(&Self, f64, &mut f64) -> bool,

    #[add_in_func(name = "MyFunctionNoRef", name_ru = "МояФункцияБезРеф")]
    #[arg(Float, as_in)]
    #[arg(Float, as_out)]
    #[returns(Bool)]
    pub my_function_no_ref: fn(f64, &mut f64) -> bool,
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

    fn my_function_mut_inner(&mut self, in_arg: f64, out_arg: &mut f64) -> bool {
        *out_arg = in_arg * 2.0;
        true
    }

    fn my_function_ref_inner(&self, in_arg: f64, out_arg: &mut f64) -> bool {
        *out_arg = in_arg * 2.0;
        true
    }

    fn my_function_no_ref_inner(in_arg: f64, out_arg: &mut f64) -> bool {
        *out_arg = in_arg * 2.0;
        true
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
