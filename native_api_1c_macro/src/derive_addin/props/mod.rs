use darling::FromMeta;
use proc_macro2::{Ident, TokenStream};

use crate::derive_addin::{
    common_generators::SettableTypes,
    constants::{BLOB_TYPE, BOOL_TYPE, DATE_TYPE, F64_TYPE, I32_TYPE, STRING_TYPE},
};

pub mod collectors;
pub mod generate;
pub mod parse;

pub struct PropDesc {
    pub ident: Ident,

    pub name: String,
    pub name_ru: String,

    pub name_literal: TokenStream,
    pub name_ru_literal: TokenStream,

    pub readable: bool,
    pub writable: bool,
    pub ty: PropType,
}

#[derive(Clone, Debug)]
pub enum PropType {
    Bool,
    I32,
    F64,
    String,
    Date,
    Blob,
}

const META_TYPE_ERR: &str = "expected string literal or path";

impl FromMeta for PropType {
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

impl TryFrom<&str> for PropType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            BOOL_TYPE => Ok(PropType::Bool),
            I32_TYPE => Ok(PropType::I32),
            F64_TYPE => Ok(PropType::F64),
            STRING_TYPE => Ok(PropType::String),
            DATE_TYPE => Ok(PropType::Date),
            BLOB_TYPE => Ok(PropType::Blob),
            _ => Err(()),
        }
    }
}

impl TryFrom<&PropType> for SettableTypes {
    type Error = ();

    fn try_from(value: &PropType) -> Result<Self, Self::Error> {
        match value {
            PropType::Bool => Ok(SettableTypes::Bool),
            PropType::I32 => Ok(SettableTypes::I32),
            PropType::F64 => Ok(SettableTypes::F64),
            PropType::String => Ok(SettableTypes::String),
            PropType::Date => Ok(SettableTypes::Date),
            PropType::Blob => Ok(SettableTypes::Blob),
        }
    }
}
