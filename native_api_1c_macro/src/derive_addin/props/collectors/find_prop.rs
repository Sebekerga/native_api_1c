use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::props::PropDesc;

use super::{empty_prop_collector_error, PropCollector};

pub struct FindPropCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for FindPropCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for FindPropCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut find_prop_body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            let name_literal = prop_desc.name_literal.clone();
            let name_ru_literal = prop_desc.name_ru_literal.clone();

            find_prop_body.extend(quote! {
                if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_literal) == name { 
                    return Some(#prop_index) 
                };
                if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_ru_literal) == name { 
                    return Some(#prop_index) 
                };
            });
        }

        let _definition = quote! {
            fn find_prop(&self, name: &[u16]) -> Option<usize> {
                #find_prop_body
                None
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for FindPropCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
