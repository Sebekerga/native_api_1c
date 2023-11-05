use proc_macro2::TokenStream;

use super::FuncDesc;
use crate::utils::macros::tkn_err_inner;

pub mod call_as_func;
pub mod call_as_proc;
pub mod find_method;
pub mod get_method_name;
pub mod get_n_methods;
pub mod get_n_params;
pub mod get_param_def_value;
pub mod has_ret_val;

pub use call_as_func::CallAsFuncCollector;
pub use call_as_proc::CallAsProcCollector;
pub use find_method::FindMethodCollector;
pub use get_method_name::GetMethodNameCollector;
pub use get_n_methods::GetNMethodsCollector;
pub use get_n_params::GetNParamsCollector;
pub use get_param_def_value::GetParamDefValueCollector;
pub use has_ret_val::HasReturnValueCollector;

pub trait FunctionCollector<'a>: FromIterator<(usize, &'a FuncDesc)> + Default {
    fn release(self) -> Result<TokenStream, darling::Error>;
}

pub fn empty_func_collector_error() -> darling::Error {
    tkn_err_inner!("No functions found", &proc_macro2::Span::call_site())
}
