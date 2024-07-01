use super::super::super::utils::expr_to_os_value;
use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::functions::{FuncDesc, FuncParamType};

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
        let mut body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let mut func_body = quote! {};
            for (arg_index, arg_desc) in func_desc.get_1c_params().iter().enumerate() {
                let Some(expr) = &arg_desc.default else {
                    // Skip parameters without default value
                    continue;
                };

                let FuncParamType::PlatformType(ty) = &arg_desc.ty else {
                    // Skip parameters that is not platform type
                    continue;
                };

                let expr = expr_to_os_value(expr, ty, true);
                func_body.extend(quote! {
                    if param_num == #arg_index  {
                        return Some(#expr);
                    }
                })
            }
            body.extend(quote! {
                if method_num == #func_index {
                    #func_body
                    return None;
                };
            });
        }

        let definition = quote! {
            fn get_param_def_value(
                &self,
                method_num: usize,
                param_num: usize,
            ) -> Option<native_api_1c::native_api_1c_core::interface::ParamValue> {
                #body
                None
            }
        };

        Self {
            generated: Ok(definition),
        }
    }
}

impl FunctionCollector<'_> for GetParamDefValueCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
