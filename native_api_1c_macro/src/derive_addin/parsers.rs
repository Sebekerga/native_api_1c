use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use super::constants::{BLOB_TYPE, BOOL_TYPE, DATE_TYPE, F64_TYPE, I32_TYPE, STRING_TYPE};

#[derive(Clone, Debug, PartialEq)]
pub enum ParamType {
    Bool,
    I32,
    F64,
    String,
    Date,
    Blob,
}

const META_TYPE_ERR: &str = "expected string literal or path";

impl FromMeta for ParamType {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        let meta_type_err = darling::Error::custom(META_TYPE_ERR);
        let expr_string = match expr {
            syn::Expr::Lit(str_lit) => match str_lit.lit {
                syn::Lit::Str(ref str) => str.value(),
                _ => return Err(meta_type_err),
            },
            syn::Expr::Path(path) => path.path.segments.first().unwrap().ident.to_string(),
            _ => return Err(meta_type_err),
        };
        Self::from_string(&expr_string)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        let joined_allowed_types = crate::derive_addin::constants::ALL_ARG_TYPES.join(", ");
        Self::try_from(value).map_err(|_| {
            darling::Error::custom(format!(
                "unknown type `{value}`. Must be one of: {joined_allowed_types}",
            ))
        })
    }
}

impl TryFrom<&str> for ParamType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            BOOL_TYPE => Ok(ParamType::Bool),
            I32_TYPE => Ok(ParamType::I32),
            F64_TYPE => Ok(ParamType::F64),
            STRING_TYPE => Ok(ParamType::String),
            DATE_TYPE => Ok(ParamType::Date),
            BLOB_TYPE => Ok(ParamType::Blob),
            _ => Err(()),
        }
    }
}

impl ToTokens for ParamType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        *tokens = match self {
            ParamType::Bool => {
                quote! { native_api_1c::native_api_1c_core::interface::ParamValue::Bool }
            }
            ParamType::I32 => {
                quote! { native_api_1c::native_api_1c_core::interface::ParamValue::I32 }
            }
            ParamType::F64 => {
                quote! { native_api_1c::native_api_1c_core::interface::ParamValue::F64 }
            }
            ParamType::Date => {
                quote! { native_api_1c::native_api_1c_core::interface::ParamValue::Date }
            }
            ParamType::String => {
                quote! { native_api_1c::native_api_1c_core::interface::ParamValue::String }
            }
            ParamType::Blob => {
                quote! { native_api_1c::native_api_1c_core::interface::ParamValue::Blob }
            }
        }
    }
}

#[derive(Debug)]
pub enum PropName {
    StringLiteral(syn::LitStr),
    Ident(syn::ExprPath),
}

impl FromMeta for PropName {
    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Lit(lit) => match &lit.lit {
                syn::Lit::Str(str_lit) => Ok(PropName::StringLiteral(str_lit.clone())),
                _ => Err(darling::Error::custom("expected string literal").with_span(expr)),
            },
            syn::Expr::Path(path) => Ok(PropName::Ident(path.clone())),
            _ => Err(darling::Error::custom("expected string literal or path").with_span(expr)),
        }
    }
}

impl From<PropName> for proc_macro2::TokenStream {
    fn from(prop_name: PropName) -> proc_macro2::TokenStream {
        match prop_name {
            PropName::StringLiteral(str_lit) => str_lit.to_token_stream(),
            PropName::Ident(ident) => ident.to_token_stream(),
        }
    }
}
