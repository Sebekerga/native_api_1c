use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::functions::FuncDesc;

use super::{empty_func_collector_error, FunctionCollector};

pub struct GetMethodNameCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetMethodNameCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for GetMethodNameCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut get_func_name_body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let name_literal = func_desc.name_literal.clone();
            let name_ru_literal = func_desc.name_ru_literal.clone();

            get_func_name_body.extend(quote! {
                #get_func_name_body
                if num == #func_index && alias == 0 {
                    return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        #name_literal).into()
                    )
                };
                if num == #func_index {
                    return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(
                        #name_ru_literal).into()
                    )
                };
            });
        }

        let get_func_name_definition = quote! {
            fn get_method_name(&self, num: usize, alias: usize) -> Option<Vec<u16>> {
                #get_func_name_body
                None
            }
        };

        Self {
            generated: Ok(get_func_name_definition),
        }
    }
}

impl FunctionCollector<'_> for GetMethodNameCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
