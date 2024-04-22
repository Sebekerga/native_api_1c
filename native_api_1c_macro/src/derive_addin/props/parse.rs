use darling::{FromField, FromMeta};
use quote::ToTokens;
use syn::{Attribute, DataStruct};

use crate::derive_addin::utils::ident_option_to_darling_err;

use super::{PropDesc, PropType};

impl FromField for PropDesc {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let field_ident = ident_option_to_darling_err(field.ident.as_ref())?;

        let add_in_prop_attr: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("add_in_prop"))
            .collect();
        if add_in_prop_attr.is_empty() {
            return Err(
                darling::Error::custom("Field must have `add_in_prop` attribute")
                    .with_span(&field_ident.clone()),
            );
        } else if add_in_prop_attr.len() > 1 {
            return Err(
                darling::Error::custom("Field can have only 1 `add_in_prop` attribute")
                    .with_span(&field_ident.clone()),
            );
        };
        let add_in_prop_attr = add_in_prop_attr[0];

        let prop_meta = PropMeta::from_meta(&add_in_prop_attr.meta)?;

        let name_literal = match prop_meta.name {
            PropName::StringLiteral(name) => name.to_token_stream(),
            PropName::Ident(ident) => ident.to_token_stream(),
        };
        let name_ru_literal = match prop_meta.name_ru {
            PropName::StringLiteral(name_ru) => name_ru.to_token_stream(),
            PropName::Ident(ident) => ident.to_token_stream(),
        };

        Ok(PropDesc {
            ident: field_ident.clone(),

            name_literal,
            name_ru_literal,

            readable: prop_meta.readable.is_some(),
            writable: prop_meta.writable.is_some(),
            ty: prop_meta.ty,
        })
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

#[derive(FromMeta, Debug)]
pub struct PropMeta {
    pub ty: PropType,
    pub name: PropName,
    pub name_ru: PropName,
    pub readable: Option<()>,
    pub writable: Option<()>,
}

pub fn parse_props(struct_data: &DataStruct) -> Result<Vec<PropDesc>, darling::Error> {
    let mut props = vec![];

    for field in &struct_data.fields {
        let has_add_in_prop_attr = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("add_in_prop"));
        if !has_add_in_prop_attr {
            continue;
        };

        let prop_desc = PropDesc::from_field(field)?;
        props.push(prop_desc);
    }

    Ok(props)
}
