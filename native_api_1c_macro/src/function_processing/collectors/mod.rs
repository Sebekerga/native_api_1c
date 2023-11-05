use proc_macro2::TokenStream;

use super::FuncDesc;
use crate::utils::macros::tkn_err_inner;

pub mod find_method;

pub trait FunctionCollector<'a>: FromIterator<(usize, &'a FuncDesc)> + Default {
    fn release(&self) -> Result<TokenStream, darling::Error>;
}

pub fn empty_func_collector_error() -> darling::Error {
    tkn_err_inner!("No functions found", &proc_macro2::Span::call_site())
}
