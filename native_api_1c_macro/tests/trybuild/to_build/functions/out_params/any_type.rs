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

    #[add_in_func(name = "MyFunctionMut", name_ru = "МояФункцияМут")]
    #[arg(ty = Any, as_in)]
    #[arg(ty = Any, as_out)]
    #[returns(ty = Bool)]
    pub my_function_mut: fn(&mut Self, ParamValue, &mut ParamValue) -> bool,

    #[add_in_func(name = "MyFunctionRef", name_ru = "МояФункцияРеф")]
    #[arg(ty = Any, as_in)]
    #[arg(ty = Any, as_out)]
    #[returns(ty = Bool)]
    pub my_function_ref: fn(&Self, ParamValue, &mut ParamValue) -> bool,

    #[add_in_func(name = "MyFunctionNoRef", name_ru = "МояФункцияБезРеф")]
    #[arg(ty = Any, as_in)]
    #[arg(ty = Any, as_out)]
    #[returns(ty = Bool)]
    pub my_function_no_ref: fn(ParamValue, &mut ParamValue) -> bool,
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

    fn my_function_mut_inner(&mut self, in_arg: ParamValue, out_arg: &mut ParamValue) -> bool {
        *out_arg = in_arg;
        true
    }

    fn my_function_ref_inner(&self, in_arg: ParamValue, out_arg: &mut ParamValue) -> bool {
        *out_arg = in_arg;
        true
    }

    fn my_function_no_ref_inner(in_arg: ParamValue, out_arg: &mut ParamValue) -> bool {
        *out_arg = in_arg;
        true
    }
}

fn main() {
    let _add_in = MyAddIn::new();
}
