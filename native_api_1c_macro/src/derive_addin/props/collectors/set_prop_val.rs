use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::props::{generate::param_ty_to_ffi_set, PropDesc};

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
        let mut set_prop_val_body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            if !prop_desc.writable {
                continue;
            }

            let prop_ident = &prop_desc.ident;
            let prop_set_tkn = param_ty_to_ffi_set(&prop_desc.ty, quote! { #prop_ident });
            set_prop_val_body.extend(quote! {
                if num == #prop_index {
                    match val {
                        #prop_set_tkn
                        _ => return false,
                    }
                    return true;
                };
            });
        }

        let _definition = quote! {
            fn set_prop_val(&mut self, num: usize, val: &native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue) -> bool {
                #set_prop_val_body
                false
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for SetPropValCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
