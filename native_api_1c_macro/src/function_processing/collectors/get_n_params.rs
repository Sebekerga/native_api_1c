use proc_macro2::TokenStream;
use quote::quote;

use crate::function_processing::{FuncDesc, ParamType};

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
        let mut get_n_params_body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let number_of_params = func_desc
                .params
                .iter()
                .filter(|p| !matches!(p.ty, ParamType::SelfType))
                .count();
            get_n_params_body.extend(quote! {
                #get_n_params_body
                if num == #func_index { return #number_of_params };
            });
        }

        let find_method_definition = quote! {
            fn get_n_params(&self, num: usize) -> usize {
                #get_n_params_body
                0
            }
        };

        Self {
            generated: Ok(find_method_definition),
        }
    }
}

impl FunctionCollector<'_> for GetNParamsCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
