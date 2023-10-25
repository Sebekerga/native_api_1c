use proc_macro::TokenStream;
use quote::{spanned::Spanned, ToTokens};
use syn::{
    punctuated::Punctuated, Expr, Token, DataStruct, Ident,
};
use crate::{utils::{macros::{tkn_err, tkn_err_inner}, convert_ty_to_param_type}, constants::{NAME_ATTR, NAME_RU_ATTR, READABLE_ATTR, WRITABLE_ATTR}, types_1c::ParamType};

pub struct PropDesc {
    pub ident: Ident,
    pub name: String,
    pub name_ru: String,
    pub readable: bool,
    pub writable: bool,
    pub ty: ParamType,
}

pub fn parse_props(struct_data: &DataStruct) -> Result<Vec<PropDesc>, TokenStream> {

    let mut props = vec![];   

    // iterate over props
    for prop in &struct_data.fields {
        let Some(attr) = prop.attrs.get(0) else { 
            continue; 
        };
        if !attr.path().is_ident("add_in_prop") {
            continue;
        };
        if prop.attrs.len() > 1 {
            return tkn_err!("AddIn fields can have 1 attribute at most", prop.__span());
        }
        let Some(prop_ident) = prop.ident.clone() else {
            return tkn_err!("AddIn props must have a name", prop.__span());
        };

        let name_values: Punctuated<Expr, Token![,]> = attr
            .parse_args_with(Punctuated::parse_terminated)
            .map_err::<TokenStream, _>(|e| tkn_err_inner!(e.to_string(), attr.__span()))?;

        let args = name_values
            .iter()
            .flat_map(|exp| {
                let Expr::Assign(assign) = exp else { 
                    return Some((exp.to_token_stream().to_string(), None, exp.__span())); 
                };
                let Expr::Lit(lit) = &*assign.right else { return None };
                let syn::Lit::Str(str_lit) = &lit.lit else { return None };
                Some((assign.left.to_token_stream().to_string(), Some(str_lit.value()), exp.__span()))
            });
        let Some(prop_name) = args
            .clone()
            .find(|(name, _, _)| name == NAME_ATTR) else {
                return tkn_err!("AddIn prop must have a `name` argument: name = \"MyPropName\"", attr.__span());
            };
        let Some(prop_name) = prop_name.1 else {
            return tkn_err!("AddIn prop argument `name` must be a string literal assignment: name = \"MyPropName\"", prop_name.2);
        };

        let Some(prop_name_ru) = args
            .clone()
            .find(|(name, _, _)| name == NAME_RU_ATTR) else {
                return tkn_err!("AddIn prop must have a `name_ru` argument: name_ru = \"МоеСвойство\"", attr.__span());
            };
        let Some(prop_name_ru) = prop_name_ru.1 else {
            return tkn_err!("AddIn prop argument `name_ru` must be a string literal assignment: name_ru = \"МоеСвойство\"", prop_name_ru.2);
        };

        let readable = args
            .clone()
            .find(|(name, _, _)| name == READABLE_ATTR)
            .is_some();

        let writable = args
            .clone()
            .find(|(name, _, _)| name == WRITABLE_ATTR)
            .is_some();

        if !readable && !writable {
            return tkn_err!("AddIn prop must be either readable, writable or both", attr.__span());
        }

        props.push(PropDesc {
            ident: prop_ident,
            name: prop_name,
            name_ru: prop_name_ru,
            readable,
            writable,
            ty: convert_ty_to_param_type(&prop.ty, prop.__span())?,
        });
    };

    Ok(props)
}
