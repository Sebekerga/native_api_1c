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
        let mut body = TokenStream::new();
        for (func_index, func_desc) in iter {
            let call_func = func_call_tkn(func_desc, None);
            body.extend(quote! {
                if method_num == #func_index {
                    #call_func
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
                #body
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
