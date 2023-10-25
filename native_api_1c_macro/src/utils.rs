use proc_macro::{LexError, TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, Type};

use crate::types_1c::ParamType;

use self::macros::tkn_err;

pub mod macros {
    macro_rules! tkn_err_inner {
        ($str:expr, $span:expr) => {
            syn::Error::new($span, $str).to_compile_error().into()
        };
    }

    macro_rules! tkn_err {
        ($str:expr, $span:expr) => {
            Err(crate::utils::macros::tkn_err_inner!($str, $span))
        };
    }

    pub(crate) use tkn_err;
    pub(crate) use tkn_err_inner;
}

pub fn convert_ty_to_param_type(ty: &Type, span: Span) -> Result<ParamType, TokenStream> {
    match ty {
        Type::Path(path_type) => {
            let syn::Path {
                leading_colon,
                segments,
            } = &path_type.path;
            if leading_colon.is_some() {
                return tkn_err!("AddIn props type must not have leading colons", span);
            }
            if segments.len() != 1 {
                return tkn_err!("AddIn props type must have exactly one segment", span);
            }
            let syn::PathSegment { ident, arguments } = segments.iter().next().unwrap();
            let syn::PathArguments::None = arguments else {
                return tkn_err!("AddIn props type must not have arguments", span);
            };

            match ident.to_string().as_str() {
                "bool" => Ok(ParamType::Bool),
                "i32" => Ok(ParamType::I32),
                "f64" => Ok(ParamType::F64),
                "String" => Ok(ParamType::String),
                _ => return tkn_err!("AddIn props type must be bool, i32, f64 or String", span),
            }
        }
        _ => return tkn_err!("AddIn props type must be bool, i32, f64, or String", span),
    }
}

pub fn param_ty_to_ffi_return(
    param_type: &ParamType,
    target: proc_macro2::TokenStream,
    source: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    match param_type {
        ParamType::Bool => Ok(quote! { #target.set_bool(#source.into()) }),
        ParamType::I32 => Ok(quote! { #target.set_i32(#source.into()) }),
        ParamType::F64 => Ok(quote! { #target.set_f64(#source.into()) }),
        ParamType::String => Ok(
            quote! { #target.set_str(&native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(String::from(#source.clone()).as_str())) },
        ),
        ParamType::Date => Ok(quote! { #target.set_date(#source.into()) }),
        ParamType::Blob => Ok(quote! { #target.set_blob(&#source) }),
        ParamType::SelfType => unreachable!("SelfType is never used in return params"),
    }
}

pub fn param_ty_to_ffi_set(
    param_type: &ParamType,
    param_path: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    match param_type {
        ParamType::Bool => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Bool(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        ParamType::I32 => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        ParamType::F64 => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        ParamType::String => Ok(quote! {
            native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(inner_value) => self.#param_path
                = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(inner_value).into(),
        }),
        ParamType::Date => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        ParamType::Blob => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Blob(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        ParamType::SelfType => unreachable!("SelfType is never used in return params"),
    }
}

pub fn str_literal_token(
    str_literal: &str,
    err_ident: &Ident,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    let token: Result<TokenStream, TokenStream> =
        format!(r#""{}""#, str_literal)
            .parse()
            .map_err(|e: LexError| {
                let token2: TokenStream = Err(syn::Error::new(
                    err_ident.span(),
                    format!("LexErr: {}", e.to_string()),
                )
                .to_compile_error())
                .unwrap();
                token2
            });
    token.map(|t| t.into())
}
