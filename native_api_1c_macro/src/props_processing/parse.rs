use darling::{FromField, FromMeta};
use syn::{Attribute, DataStruct};

use crate::utils::ident_option_to_darling_err;

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

        Ok(PropDesc {
            ident: field_ident.clone(),
            name: prop_meta.name,
            name_ru: prop_meta.name_ru,
            readable: prop_meta.readable.is_some(),
            writable: prop_meta.writable.is_some(),
            ty: prop_meta.ty,
        })
    }
}

#[derive(FromMeta, Debug)]
pub struct PropMeta {
    pub ty: PropType,
    pub name: String,
    pub name_ru: String,
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
