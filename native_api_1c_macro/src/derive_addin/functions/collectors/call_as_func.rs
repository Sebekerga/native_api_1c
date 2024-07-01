use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::derive_addin::functions::{generate::func_call_tkn, FuncDesc};

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
        let mut body = TokenStream::new();
        for (func_index, func_desc) in iter {
            if func_desc.return_value.ty.is_none() {
                // Skip functions without return value
                continue;
            }

            let return_val_ident = Ident::new("val", proc_macro2::Span::call_site());
            let call_func = func_call_tkn(func_desc, Some(&return_val_ident));
            body.extend(quote! {
                if method_num == #func_index {
                    #call_func
                    return Ok(val);
                };
            });
        }

        let definition = quote! {
            fn call_as_func(
                &mut self,
                method_num: usize,
                params: &mut native_api_1c::native_api_1c_core::interface::ParamValues,
            ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<
                native_api_1c::native_api_1c_core::interface::ParamValue
            > {
                #body
                Err(())
            }
        };

        Self {
            generated: Ok(definition),
        }
    }
}

impl FunctionCollector<'_> for CallAsFuncCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
