use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{
    common_generators::{param_ty_to_ffi_return, SettableTypes},
    function_processing::FuncDesc,
    utils::macros::tkn_err_inner,
};

use super::{empty_func_collector_error, FunctionCollector};

pub struct GetParamDefValueCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for GetParamDefValueCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for GetParamDefValueCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut get_param_def_value_body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let mut this_get_param_def_value_body = quote! {};
            for (i, arg_desc) in func_desc.params.iter().enumerate() {
                match &arg_desc.default {
                    Some(expr) => {
                        let prop_settable: SettableTypes =
                            match (&arg_desc.ty).try_into().map_err(|_| {
                                tkn_err_inner!(
                                    "Incorrectly attempted to convert type to settable",
                                    &func_desc.ident.span()
                                )
                            }) {
                                Ok(st) => st,
                                Err(err) => {
                                    return Self {
                                        generated: Err(err),
                                    }
                                }
                            };
                        let value_setter = param_ty_to_ffi_return(
                            &prop_settable,
                            quote! { value },
                            expr.into_token_stream(),
                        );
                        this_get_param_def_value_body = quote! {
                            #this_get_param_def_value_body
                            if param_num == #i  {
                                #value_setter;
                                return true;
                            }
                        }
                    }
                    None => {}
                }
            }
            get_param_def_value_body = quote! {
                #get_param_def_value_body
                if method_num == #func_index {
                    #this_get_param_def_value_body
                    return false;
                };
            };
        }

        let find_method_definition = quote! {
            fn get_param_def_value(
                &self,
                method_num: usize,
                param_num: usize,
                value: native_api_1c::native_api_1c_core::ffi::provided_types::ReturnValue,
            ) -> bool {
                #get_param_def_value_body
                false
            }
        };

        Self {
            generated: Ok(find_method_definition),
        }
    }
}

impl FunctionCollector<'_> for GetParamDefValueCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
