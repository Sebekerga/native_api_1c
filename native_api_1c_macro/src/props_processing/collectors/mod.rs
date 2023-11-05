use proc_macro2::TokenStream;

use super::PropDesc;
use crate::utils::macros::tkn_err_inner;

pub mod find_prop;
pub mod get_n_props;
pub mod get_prop_name;
pub mod get_prop_val;
pub mod is_prop_readable;
pub mod is_prop_writable;
pub mod set_prop_val;

pub use find_prop::FindPropCollector;
pub use get_n_props::GetNPropsCollector;
pub use get_prop_name::GetPropNameCollector;
pub use get_prop_val::GetPropValCollector;
pub use is_prop_readable::IsPropReadableCollector;
pub use is_prop_writable::IsPropWritableCollector;
pub use set_prop_val::SetPropValCollector;

pub trait PropCollector<'a>: FromIterator<(usize, &'a PropDesc)> + Default {
    fn release(self) -> Result<TokenStream, darling::Error>;
}

pub fn empty_prop_collector_error() -> darling::Error {
    tkn_err_inner!("No props found", &proc_macro2::Span::call_site())
}
