use darling::FromMeta;
use proc_macro2::Ident;
use syn::Expr;

use crate::{
    common_generators::SettableTypes,
    constants::{BLOB_TYPE, BOOL_TYPE, DATE_TYPE, F64_TYPE, I32_TYPE, STRING_TYPE, UNTYPED_TYPE},
};

pub mod generate;
pub mod parse;

pub struct FuncDesc {
    pub ident: Ident,
    pub name: String,
    pub name_ru: String,
    pub params: Vec<FuncArgumentDesc>,
    pub return_value: ReturnTypeDesc,
}

pub struct FuncArgumentDesc {
    pub ty: ParamType,
    pub default: Option<Expr>,
    pub out_param: bool,
}

pub struct ReturnTypeDesc {
    pub ty: ReturnType,
    pub result: bool,
}
const META_TYPE_ERR: &str = "expected string literal or path";

#[derive(Clone, Debug)]
pub enum ParamType {
    Bool,
    I32,
    F64,
    String,
    Date,
    Blob,
    SelfType,
}

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
        let joined_allowed_types = crate::constants::ALL_ARG_TYPES.join(", ");
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

impl TryFrom<&ParamType> for SettableTypes {
    type Error = ();

    fn try_from(value: &ParamType) -> Result<Self, Self::Error> {
        match value {
            ParamType::Bool => Ok(SettableTypes::Bool),
            ParamType::I32 => Ok(SettableTypes::I32),
            ParamType::F64 => Ok(SettableTypes::F64),
            ParamType::String => Ok(SettableTypes::String),
            ParamType::Date => Ok(SettableTypes::Date),
            ParamType::Blob => Ok(SettableTypes::Blob),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ReturnType {
    Bool,
    I32,
    F64,
    String,
    Date,
    Blob,
    None,
}

impl FromMeta for ReturnType {
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
        let joined_allowed_types = crate::constants::ALL_RETURN_TYPES.join(", ");
        Self::try_from(value).map_err(|_| {
            darling::Error::custom(format!(
                "unknown type `{value}`. Must be one of: {joined_allowed_types}",
            ))
        })
    }
}

impl TryFrom<&str> for ReturnType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            BOOL_TYPE => Ok(ReturnType::Bool),
            I32_TYPE => Ok(ReturnType::I32),
            F64_TYPE => Ok(ReturnType::F64),
            STRING_TYPE => Ok(ReturnType::String),
            DATE_TYPE => Ok(ReturnType::Date),
            BLOB_TYPE => Ok(ReturnType::Blob),
            UNTYPED_TYPE => Ok(ReturnType::None),
            _ => Err(()),
        }
    }
}

impl TryFrom<&ReturnType> for SettableTypes {
    type Error = ();

    fn try_from(value: &ReturnType) -> Result<Self, Self::Error> {
        match value {
            ReturnType::Bool => Ok(SettableTypes::Bool),
            ReturnType::I32 => Ok(SettableTypes::I32),
            ReturnType::F64 => Ok(SettableTypes::F64),
            ReturnType::String => Ok(SettableTypes::String),
            ReturnType::Date => Ok(SettableTypes::Date),
            ReturnType::Blob => Ok(SettableTypes::Blob),
            _ => Err(()),
        }
    }
}
