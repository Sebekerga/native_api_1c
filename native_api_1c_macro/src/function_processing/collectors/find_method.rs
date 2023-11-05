use proc_macro2::TokenStream;
use quote::quote;

use crate::function_processing::FuncDesc;

use super::{empty_func_collector_error, FunctionCollector};

pub struct FindMethodCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for FindMethodCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for FindMethodCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut find_method_body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let name_literal = func_desc.name_literal.clone();
            let name_ru_literal = func_desc.name_ru_literal.clone();

            find_method_body.extend(quote! {
                if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_literal) == name { 
                    return Some(#func_index) 
                };
                if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_ru_literal) == name { 
                    return Some(#func_index) 
                };
            });
        }

        let find_method_definition = quote! {
            fn find_method(&self, name: &[u16]) -> Option<usize> {
                #find_method_body
                None
            }
        };

        Self {
            generated: Ok(find_method_definition),
        }
    }
}

impl FunctionCollector<'_> for FindMethodCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
