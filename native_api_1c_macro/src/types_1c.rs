use darling::FromMeta;

use crate::constants::{BLOB_TYPE, BOOL_TYPE, DATE_TYPE, F64_TYPE, I32_TYPE, STRING_TYPE};

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
        let expr_string = match expr {
            syn::Expr::Lit(str_lit) => match str_lit.lit {
                syn::Lit::Str(ref str) => str.value(),
                _ => return Err(darling::Error::custom("expected string literal or path")),
            },
            syn::Expr::Path(path) => path.path.segments.first().unwrap().ident.to_string(),
            _ => return Err(darling::Error::custom("expected string literal or path")),
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
