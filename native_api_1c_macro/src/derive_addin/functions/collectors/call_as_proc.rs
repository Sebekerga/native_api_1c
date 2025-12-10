use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::functions::{generate::func_call_tkn, FuncDesc};

use super::{empty_func_collector_error, FunctionCollector};

pub struct CallAsProcCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for CallAsProcCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for CallAsProcCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut tkn_func_calls_with_selectors = vec![];
        for (func_index, func_desc) in iter {
            let tkn_func_call = func_call_tkn(func_desc, None);
            tkn_func_calls_with_selectors.push(quote! {
                if method_num == #func_index {
                    #tkn_func_call
                    return Ok(());
                };
            });
        }

        let definition = quote! {
            fn call_as_proc(
                &mut self,
                method_num: usize,
                params: &mut native_api_1c::native_api_1c_core::interface::ParamValues,
            ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<()> {
                #(#tkn_func_calls_with_selectors)*

                // platform call was invalid
                Err(())
            }
        };

        Self {
            generated: Ok(definition),
        }
    }
}

impl FunctionCollector<'_> for CallAsProcCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
