use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::functions::FuncDesc;

use super::{empty_func_collector_error, FunctionCollector};

pub struct GetNMethodsCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetNMethodsCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for GetNMethodsCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let number_of_func = iter.into_iter().count();

        let find_method_definition = quote! {
            fn get_n_methods(&self) -> usize {
                #number_of_func
            }
        };

        Self {
            generated: Ok(find_method_definition),
        }
    }
}

impl FunctionCollector<'_> for GetNMethodsCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
