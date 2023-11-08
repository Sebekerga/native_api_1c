use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::props::PropDesc;

use super::{empty_prop_collector_error, PropCollector};

pub struct GetNPropsCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetNPropsCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for GetNPropsCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut _body = TokenStream::new();

        let number_of_props = iter.into_iter().count();

        let _definition = quote! {
            fn get_n_props(&self) -> usize {
                #number_of_props
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for GetNPropsCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
