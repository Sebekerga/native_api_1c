use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Ident;

use super::{FuncArgumentDesc, FuncDesc, ParamType, ReturnType};

pub fn func_call_tkn(func: &FuncDesc, set_to: Option<&Ident>) -> TokenStream {
    let func_ident = &func.ident;
    let mut param_extract = quote! {};
    let mut pre_call = quote! {};
    let mut func_call = quote! {};
    let mut post_call = quote! {};

    func.params.iter().enumerate().for_each(|(counter, param)| {
        let param_ident = match param.ty {
            ParamType::SelfType => Ident::new("param_self", Span::call_site()),
            _ => {
                let ident = Ident::new(&format!("param_{}", counter), Span::call_site());
                let param_ident_raw = Ident::new(&format!("{}_raw", ident), Span::call_site());
                param_extract.extend(quote! { ref mut #param_ident_raw, });
                ident
            }
        };

        let (param_pre_call, param_post_call) = gen_param_prep(param, &param_ident);
        pre_call.extend(quote! {            
            #param_pre_call
        });
        post_call.extend(quote! {            
            #param_post_call
        });

        match param.ty {
            ParamType::SelfType => {
                func_call.extend(quote! {                    
                    self,
                })
            }
            _ => {
                if param.out_param {
                    func_call.extend(quote! {                        
                        #param_ident,
                    });
                } else {
                    func_call.extend(quote! {                        
                        #param_ident.clone().into(),
                    });
                }
            }
        }
    });

    pre_call = quote! {
        let [#param_extract] = params else {
            return false;
        };
        #pre_call
    };

    let mut func_call = if func.return_value.result {
        quote! {
            #pre_call
            let call_result = (self.#func_ident)(#func_call);
            let Ok(call_result) = call_result else { return false; };
            #post_call
        }
    } else {
        quote! {
            #pre_call
            let call_result = (self.#func_ident)(#func_call);
            #post_call
        }
    };

    if let Some(set_to) = set_to {
        let value_setter = match &func.return_value.ty {
            ReturnType::Bool => quote! { #set_to.set_bool(call_result.into()); },
            ReturnType::I32 => quote! { #set_to.set_i32(call_result.into()); },
            ReturnType::F64 => quote! { #set_to.set_f64(call_result.into()); },
            ReturnType::String => {
                quote! { #set_to.set_str(&native_api_1c::native_api_1c_core::ffi::string_utils::os_string(String::from(&call_result).as_str())); }
            }
            ReturnType::Date => {
                quote! { #set_to.set_date(call_result.into()); }
            }
            ReturnType::Blob => {
                quote! { #set_to.set_blob(&call_result); }
            }
            ReturnType::None => {
                quote! {}
            }
        };

        func_call.extend(quote! {            
            #value_setter;
        });
    }

    func_call
}

fn gen_param_prep(
    param: &FuncArgumentDesc,
    param_ident: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let param_ident_ref = Ident::new(&format!("{}_ref", param_ident), Span::call_site());
    let param_ident_raw = Ident::new(&format!("{}_raw", param_ident), Span::call_site());

    let mut pre_call = quote! {};
    let mut post_call = quote! {};

    match param.ty {
        ParamType::Bool => {
            pre_call.extend(quote! {                
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Bool(#param_ident)
                = #param_ident_raw else {
                    return false;
                };
            });
        }
        ParamType::I32 => {
            pre_call.extend(quote! {                
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(#param_ident)
                = #param_ident_raw else {
                    return false;
                };
            });
        }
        ParamType::F64 => {
            if param.out_param {
                pre_call.extend(quote! {                    
                    let mut #param_ident_ref = match #param_ident_raw {
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(val) => *val,
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(val) => *val as f64,
                        _ => return false,
                    };
                    let #param_ident = &mut #param_ident_ref;
                });
                post_call.extend(quote! {                    
                    *#param_ident_raw = native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(*#param_ident);
                });
            } else {
                pre_call.extend(quote! {                    
                    let #param_ident = match #param_ident_raw {
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(val) => *val,
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(val) => *val as f64,
                        _ => return false,
                    };
                });
            }
        }
        ParamType::String => {
            if param.out_param {
                let param_ident_str =
                    Ident::new(&format!("{}_str", param_ident), Span::call_site());
                pre_call.extend(quote! {                    
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(#param_ident_ref)
                    = #param_ident_raw else {
                        return false;
                    };
                    let mut #param_ident_str = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(&#param_ident_ref);
                    let #param_ident = &mut #param_ident_str;
                });
                post_call.extend(quote! {                    
                    *#param_ident_ref = native_api_1c::native_api_1c_core::ffi::string_utils::os_string(&#param_ident);
                });
            } else {
                pre_call.extend(quote! {                    
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(#param_ident_ref)
                    = #param_ident_raw else {
                        return false;
                    };
                    let #param_ident = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(&#param_ident_ref);
                })
            }
        }
        ParamType::Date => {
            if param.out_param {
                pre_call.extend(quote! {                    
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(#param_ident_ref)
                    = #param_ident_raw else {
                        return false;
                    };
                    let mut #param_ident: chrono::DateTime<chrono::FixedOffset> = #param_ident_ref.clone().into();
                });
                post_call.extend(quote! {                    
                    *#param_ident_ref = #param_ident.into();
                });
            } else {
                pre_call.extend(quote! {                    
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(#param_ident)
                    = #param_ident_raw else {
                        return false;
                    };
                });
            }
        }
        ParamType::Blob => {
            pre_call.extend(quote! {                
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Blob(#param_ident)
                = #param_ident_raw else {
                    return false;
                };
            });
        }
        ParamType::SelfType => {}
    }
    (pre_call, post_call)
}
