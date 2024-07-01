use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::{props::PropDesc, utils::expr_from_os_value};

use super::{empty_prop_collector_error, PropCollector};

pub struct SetPropValCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for SetPropValCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_prop_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a PropDesc)> for SetPropValCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a PropDesc)>>(iter: T) -> Self {
        let mut body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            if !prop_desc.writable {
                continue;
            }

            let prop_ident = &prop_desc.ident;
            let prop_getter = expr_from_os_value(&quote! { val }, &prop_desc.ty);

            body.extend(quote! {
                if num == #prop_index {
                    self.#prop_ident = #prop_getter.into();
                    return Ok(());
                };
            });
        }

        let definition = quote! {
            fn set_prop_val(
                &mut self,
                num: usize,
                val: native_api_1c::native_api_1c_core::interface::ParamValue,
            ) -> native_api_1c::native_api_1c_core::interface::AddInWrapperResult<()> {
                #body
                return Err(())
            }
        };

        Self {
            generated: Ok(definition),
        }
    }
}

impl PropCollector<'_> for SetPropValCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
