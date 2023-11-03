use darling::{FromField, FromMeta};
use proc_macro::TokenStream;
use syn::{Attribute, DataStruct};

use crate::utils::{
    convert_ty_to_param_type, ident_option_to_darling_err, ident_option_to_token_err,
    macros::tkn_err_inner,
};

use super::PropDesc;

impl FromField for PropDesc {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let field_ident = ident_option_to_darling_err(field.ident.as_ref())?;

        let add_in_prop_attr: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("add_in_func"))
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

        let ty = convert_ty_to_param_type(&field.ty, field_ident.span())
            .map_err(|e| darling::Error::custom(e.to_string()).with_span(&field_ident.clone()))?;

        Ok(PropDesc {
            ident: field_ident.clone(),
            name: prop_meta.name,
            name_ru: prop_meta.name_ru,
            readable: prop_meta.readable,
            writable: prop_meta.writable,
            ty,
        })
    }
}

#[derive(FromMeta, Debug)]
pub struct PropMeta {
    pub name: String,
    pub name_ru: String,
    pub readable: bool,
    pub writable: bool,
}

pub fn parse_props(struct_data: &DataStruct) -> Result<Vec<PropDesc>, TokenStream> {
    let mut props = vec![];

    for field in &struct_data.fields {
        let has_add_in_prop_attr = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("add_in_prop"));
        if !has_add_in_prop_attr {
            continue;
        };

        let field_ident = ident_option_to_token_err(field.ident.as_ref())?;
        props.push(PropDesc::from_field(field).map_err(|e| {
            let new_e: TokenStream = tkn_err_inner!(e.to_string(), field_ident.span());
            new_e
        })?);
    }

    Ok(props)
}
