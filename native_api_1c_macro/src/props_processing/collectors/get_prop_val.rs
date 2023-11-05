use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    common_generators::{param_ty_to_ffi_return, SettableTypes},
    props_processing::PropDesc,
    utils::macros::tkn_err_inner,
};

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
        let mut get_prop_val_body = TokenStream::new();

        for (prop_index, prop_desc) in iter {
            if !prop_desc.readable {
                continue;
            }

            let prop_settable: SettableTypes = match (&prop_desc.ty).try_into().map_err(|_| {
                tkn_err_inner!(
                    "Incorrectly attempted to convert type to settable",
                    &prop_desc.ident.span()
                )
            }) {
                Ok(settable) => settable,
                Err(err) => {
                    return Self {
                        generated: Err(err),
                    }
                }
            };
            let prop_ident = &prop_desc.ident;
            let ffi_set_tkn =
                param_ty_to_ffi_return(&prop_settable, quote! { val }, quote! {self.#prop_ident});
            get_prop_val_body.extend(quote! {
                if num == #prop_index {
                    #ffi_set_tkn;
                    return true;
                };
            });
        }

        let _definition = quote! {
            fn get_prop_val(&self, num: usize, val: native_api_1c::native_api_1c_core::ffi::provided_types::ReturnValue) -> bool {
                #get_prop_val_body
                false
            }
        };

        Self {
            generated: Ok(_definition),
        }
    }
}

impl PropCollector<'_> for GetPropValCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
