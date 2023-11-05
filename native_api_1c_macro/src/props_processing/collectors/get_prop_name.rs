use proc_macro2::TokenStream;
use quote::quote;

use crate::props_processing::PropDesc;

use super::{empty_prop_collector_error, PropCollector};

pub struct GetPropNameCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetPropNameCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for GetPropNameCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut get_prop_name_body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            let name_literal = prop_desc.name_literal.clone();
            let name_ru_literal = prop_desc.name_ru_literal.clone();

            get_prop_name_body.extend(quote! {
                if num == #prop_index && alias == 0 { return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_literal).into()) };
                if num == #prop_index { return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_ru_literal).into()) };
            });
        }

        let _definition = quote! {
            fn get_prop_name(&self, num: usize, alias: usize) -> Option<Vec<u16>> {
                #get_prop_name_body
                None
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for GetPropNameCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
