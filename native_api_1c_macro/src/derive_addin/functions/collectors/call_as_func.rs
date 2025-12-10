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
        let mut tkn_func_calls_with_selectors = vec![];
        for (func_index, func_desc) in iter {
            // Skip functions without return value
            if func_desc.return_value.ty.is_none() {
                continue;
            }

            let ident_return_val = Ident::new("val", proc_macro2::Span::call_site());
            let tkn_func_call = func_call_tkn(func_desc, Some(&ident_return_val));
            tkn_func_calls_with_selectors.push(quote! {
                if method_num == #func_index {
                    #tkn_func_call
                    return Ok(#ident_return_val);
                };
            });
        }

        let definition: TokenStream = quote! {
            fn call_as_func(
                &mut self,
                method_num: usize,
                params: &mut native_api_1c::native_api_1c_core::interface::ParamValues,
            ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<
                native_api_1c::native_api_1c_core::interface::ParamValue
            > {
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

impl FunctionCollector<'_> for CallAsFuncCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
