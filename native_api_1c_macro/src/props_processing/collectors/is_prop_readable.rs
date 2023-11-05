use proc_macro2::TokenStream;
use quote::quote;

use crate::props_processing::PropDesc;

use super::{empty_prop_collector_error, PropCollector};

pub struct IsPropReadableCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for IsPropReadableCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for IsPropReadableCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut is_prop_readable_body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            let readable = prop_desc.readable;

            is_prop_readable_body.extend(quote! {
                if num == #prop_index { return #readable };
            });
        }

        let _definition = quote! {
            fn is_prop_readable(&self, num: usize) -> bool {
                #is_prop_readable_body
                false
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for IsPropReadableCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
