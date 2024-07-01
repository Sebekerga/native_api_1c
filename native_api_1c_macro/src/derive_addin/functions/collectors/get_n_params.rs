use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::functions::FuncDesc;

use super::{empty_func_collector_error, FunctionCollector};

pub struct GetNParamsCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetNParamsCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for GetNParamsCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let number_of_params = func_desc.get_1c_params().len();
            body.extend(quote! {
                if num == #func_index { return #number_of_params };
            });
        }

        let definition = quote! {
            fn get_n_params(&self, num: usize) -> usize {
                #body
                0
            }
        };

        Self {
            generated: Ok(definition),
        }
    }
}

impl FunctionCollector<'_> for GetNParamsCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
