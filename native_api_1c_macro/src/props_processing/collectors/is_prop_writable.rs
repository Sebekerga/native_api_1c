use proc_macro2::TokenStream;
use quote::quote;

use crate::props_processing::PropDesc;

use super::{empty_prop_collector_error, PropCollector};

pub struct IsPropWritableCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for IsPropWritableCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for IsPropWritableCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut is_prop_writable_body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            let writable = prop_desc.writable;
            is_prop_writable_body.extend(quote! {
                if num == #prop_index { return #writable };
            });
        }

        let _definition = quote! {
            fn is_prop_writable(&self, num: usize) -> bool {
                #is_prop_writable_body
                false
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for IsPropWritableCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
