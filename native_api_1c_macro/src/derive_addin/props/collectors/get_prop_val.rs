use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::{props::PropDesc, utils::expr_to_os_value};

use super::{empty_prop_collector_error, PropCollector};

pub struct GetPropValCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetPropValCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for GetPropValCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            if !prop_desc.readable {
                // Skip non-readable properties
                continue;
            }

            let prop_ident = &prop_desc.ident;
            let prop_setter = expr_to_os_value(&quote! {self.#prop_ident}, &prop_desc.ty, false);
            body.extend(quote! {
                if num == #prop_index {
                    return Ok(#prop_setter);
                };
            });
        }

        let definition = quote! {
            fn get_prop_val(&self, num: usize) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<
                native_api_1c::native_api_1c_core::interface::ParamValue
            > {
                #body
                return Err(())
            }
        };

        Self {
            generated: Ok(definition),
        }
    }
}

impl PropCollector<'_> for GetPropValCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
