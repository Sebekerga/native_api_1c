use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::function_processing::{generate::func_call_tkn, FuncDesc, ReturnType};

use super::{empty_func_collector_error, FunctionCollector};

pub struct CallAsFuncCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for CallAsFuncCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for CallAsFuncCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut call_as_func_body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let has_ret_val = !matches!(func_desc.return_value.ty, ReturnType::None);
            if !has_ret_val {
                continue;
            }

            let return_val_ident = Ident::new("val", proc_macro2::Span::call_site());
            let call_func = func_call_tkn(func_desc, Some(&return_val_ident));
            call_as_func_body.extend(quote! {
                if method_num == #func_index {
                    #call_func
                    return true;
                };
            });
        }

        let call_as_func_definition = quote! {
            fn call_as_func(
                &mut self,
                method_num: usize,
                params: &mut [native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue],
                val: native_api_1c::native_api_1c_core::ffi::provided_types::ReturnValue,
            ) -> bool {
                #call_as_func_body
                false
            }
        };

        Self {
            generated: Ok(call_as_func_definition),
        }
    }
}

impl FunctionCollector<'_> for CallAsFuncCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
