use darling::{FromField, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Attribute, DataStruct, Expr, Ident};

use crate::{constants::UNTYPED_TYPE, types_1c::ParamType};

pub struct FuncDesc {
    pub ident: Ident,
    pub name: String,
    pub name_ru: String,
    pub params: Vec<FuncArgumentDesc>,
    pub return_value: (Option<ParamType>, bool),
}

impl FromField for FuncDesc {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let add_in_func_attr: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("add_in_func"))
            .collect();
        if add_in_func_attr.is_empty() {
            return Err(darling::Error::custom(
                "Field must have `add_in_func` attribute",
            ));
        } else if add_in_func_attr.len() > 1 {
            return Err(darling::Error::custom(
                "Field can have only 1 `add_in_func` attribute",
            ));
        };
        let add_in_func_attr = add_in_func_attr[0];

        let arg_attrs: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("arg"))
            .collect();

        let returns_attrs: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("returns"))
            .collect();
        if returns_attrs.len() > 1 {
            return Err(darling::Error::custom(
                "Field can have at most 1 `returns` attribute",
            ));
        };
        let returns_attr = returns_attrs.get(0).copied();

        let func_meta = FuncHeadMeta::from_meta(&add_in_func_attr.parse_args()?)?;
        let params_meta = arg_attrs
            .iter()
            .map(|attr| FuncArgumentMeta::from_meta(&attr.parse_args()?))
            .collect::<darling::Result<Vec<FuncArgumentMeta>>>()?;
        let return_meta = returns_attr
            .map(|attr| FuncReturnMeta::from_meta(&attr.parse_args()?))
            .transpose()?;

        let params = params_meta
            .into_iter()
            .map(FuncArgumentDesc::try_from)
            .map(|res| res.map_err(|_| darling::Error::custom("Invalid argument type")))
            .collect::<Result<Vec<FuncArgumentDesc>, darling::Error>>()?;

        let return_value = match return_meta {
            Some(return_meta) => FuncReturnDesc::try_from(return_meta)
                .map_err(|_| darling::Error::custom("Invalid argument type"))?,
            None => FuncReturnDesc {
                ty: None,
                result: false,
            },
        };

        Ok(Self {
            ident: field.ident.clone().unwrap(),
            name: func_meta.name,
            name_ru: func_meta.name_ru,
            params,
            return_value: (return_value.ty, return_value.result),
        })
    }
}

#[derive(FromMeta, Debug)]
pub struct FuncHeadMeta {
    name: String,
    name_ru: String,
}

pub struct FuncArgumentDesc {
    pub ty: ParamType,
    pub default: Option<Expr>,
    pub out_param: bool,
}

#[derive(FromMeta, Debug)]
pub struct FuncArgumentMeta {
    ty: String,
    default: Option<Expr>,
    #[allow(dead_code)]
    as_in: bool,
    as_out: bool,
}

impl TryFrom<FuncArgumentMeta> for FuncArgumentDesc {
    type Error = ErrorConvertingMeta;

    fn try_from(arg_meta: FuncArgumentMeta) -> Result<Self, Self::Error> {
        if arg_meta.as_in && arg_meta.as_out {
            return Err(Self::Error::ConflictingParams(
                "as_in".to_string(),
                "as_out".to_string(),
            ));
        }
        Ok(Self {
            ty: ParamType::try_from(&arg_meta.ty)
                .map_err(|_| Self::Error::InvalidTypeForParam(arg_meta.ty))?,
            default: arg_meta.default,
            out_param: arg_meta.as_out,
        })
    }
}

pub struct FuncReturnDesc {
    pub ty: Option<ParamType>,
    pub result: bool,
}

#[derive(FromMeta, Debug)]
pub struct FuncReturnMeta {
    ty: String,
    result: bool,
}

impl TryFrom<FuncReturnMeta> for FuncReturnDesc {
    type Error = ErrorConvertingMeta;

    fn try_from(arg_meta: FuncReturnMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            ty: match arg_meta.ty.as_str() {
                UNTYPED_TYPE => None,
                _ => Some(
                    ParamType::try_from(&arg_meta.ty)
                        .map_err(|_| Self::Error::InvalidTypeForReturn(arg_meta.ty))?,
                ),
            },
            result: arg_meta.result,
        })
    }
}

pub enum ErrorConvertingMeta {
    InvalidTypeForParam(String),
    InvalidTypeForReturn(String),
    ConflictingParams(String, String),
}

impl From<ErrorConvertingMeta> for darling::Error {
    fn from(err: ErrorConvertingMeta) -> Self {
        match err {
            ErrorConvertingMeta::InvalidTypeForParam(ty) => {
                let joined_allowed_types = crate::constants::ALL_ARG_TYPES.join(", ");
                darling::Error::custom(format!(
                    "Invalid type: `{ty}`. Must be one of: {joined_allowed_types}"
                ))
            }
            ErrorConvertingMeta::InvalidTypeForReturn(ty) => {
                let joined_allowed_types = crate::constants::ALL_RETURN_TYPES.join(", ");
                darling::Error::custom(format!(
                    "Invalid type: `{ty}`. Must be one of: {joined_allowed_types}"
                ))
            }
            ErrorConvertingMeta::ConflictingParams(param1, param2) => {
                darling::Error::custom(format!("Conflicting params: {} and {}", param1, param2))
            }
        }
    }
}

pub fn parse_functions(struct_data: &DataStruct) -> Result<Vec<FuncDesc>, TokenStream> {
    let mut functions_descriptions = vec![];

    // iterate over methods
    for field in &struct_data.fields {
        let fields_without_docs: Vec<&syn::Attribute> = field
            .attrs
            .iter()
            .filter(|attr| !attr.path().is_ident("doc"))
            .collect();

        let has_add_in_func_attr = fields_without_docs
            .iter()
            .any(|attr| attr.path().is_ident("add_in_func"));
        if !has_add_in_func_attr {
            continue;
        };

        let func_desc_result = FuncDesc::from_field(field);
        let func_desc = match func_desc_result {
            Ok(func_desc) => func_desc,
            Err(err) => return Err(err.write_errors().into()),
        };
        functions_descriptions.push(func_desc);
    }

    Ok(functions_descriptions)
}

pub fn func_call_tkn(
    func: &FuncDesc,
    return_value: bool,
) -> Result<proc_macro2::TokenStream, TokenStream> {
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
                param_extract = quote! {
                    #param_extract ref mut #param_ident_raw,
                };
                ident
            }
        };

        let (param_pre_call, param_post_call) = gen_param_prep(param, &param_ident);
        pre_call = quote! {
            #pre_call
            #param_pre_call
        };
        post_call = quote! {
            #post_call
            #param_post_call
        };

        match param.ty {
            ParamType::SelfType => {
                func_call = quote! {
                    #func_call
                    self,
                }
            }
            _ => {
                if param.out_param {
                    func_call = quote! {
                        #func_call
                        #param_ident,
                    };
                } else {
                    func_call = quote! {
                        #func_call
                        #param_ident.clone().into(),
                    };
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

    let mut func_call = if func.return_value.1 {
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

    if return_value {
        let value_setter = match &func.return_value.clone().0.unwrap() {
            ParamType::Bool => quote! { val.set_bool(call_result.into()); },
            ParamType::I32 => quote! { val.set_i32(call_result.into()); },
            ParamType::F64 => quote! { val.set_f64(call_result.into()); },
            ParamType::String => {
                quote! { val.set_str(&native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(String::from(&call_result).as_str())); }
            }
            ParamType::Date => {
                quote! { val.set_date(call_result.into()); }
            }
            ParamType::Blob => {
                quote! { val.set_blob(&call_result); }
            }
            ParamType::SelfType => unreachable!("SelfType is never used in return params"),
        };
        func_call = quote! {
            #func_call
            #value_setter
        };
    }

    Ok(func_call)
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
            pre_call = quote! {
                #pre_call
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Bool(#param_ident)
                = #param_ident_raw else {
                    return false;
                };
            };
        }
        ParamType::I32 => {
            pre_call = quote! {
                #pre_call
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(#param_ident)
                = #param_ident_raw else {
                    return false;
                };
            };
        }
        ParamType::F64 => {
            if param.out_param {
                pre_call = quote! {
                    #pre_call
                    let mut #param_ident_ref = match #param_ident_raw {
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(val) => *val,
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(val) => *val as f64,
                        _ => return false,
                    };
                    let #param_ident = &mut #param_ident_ref;
                };
                post_call = quote! {
                    #post_call
                    *#param_ident_raw = native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(*#param_ident);
                };
            } else {
                pre_call = quote! {
                    #pre_call
                    let #param_ident = match #param_ident_raw {
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(val) => *val,
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(val) => *val as f64,
                        _ => return false,
                    };
                };
            }
        }
        ParamType::String => {
            if param.out_param {
                let param_ident_str =
                    Ident::new(&format!("{}_str", param_ident), Span::call_site());
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(#param_ident_ref)
                    = #param_ident_raw else {
                        return false;
                    };
                    let mut #param_ident_str = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(&#param_ident_ref);
                    let #param_ident = &mut #param_ident_str;
                };
                post_call = quote! {
                    #post_call
                    *#param_ident_ref = native_api_1c::native_api_1c_core::ffi::string_utils::os_string(&#param_ident);
                };
            } else {
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(#param_ident_ref)
                    = #param_ident_raw else {
                        return false;
                    };
                    let #param_ident = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(&#param_ident_ref);
                }
            }
        }
        ParamType::Date => {
            if param.out_param {
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(#param_ident_ref)
                    = #param_ident_raw else {
                        return false;
                    };
                    let mut #param_ident: chrono::DateTime<chrono::FixedOffset> = #param_ident_ref.clone().into();
                };
                post_call = quote! {
                    #post_call
                    *#param_ident_ref = #param_ident.into();
                };
            } else {
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(#param_ident)
                    = #param_ident_raw else {
                        return false;
                    };
                };
            }
        }
        ParamType::Blob => {
            pre_call = quote! {
                #pre_call
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Blob(#param_ident)
                = #param_ident_raw else {
                    return false;
                };
            };
        }
        ParamType::SelfType => {}
    }
    (pre_call, post_call)
}
