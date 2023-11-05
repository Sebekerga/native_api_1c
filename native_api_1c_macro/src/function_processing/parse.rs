use darling::{FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Attribute, DataStruct, Expr};

use crate::utils::{ident_option_to_darling_err, str_literal_token};

use super::{FuncArgumentDesc, FuncDesc, ParamType, ReturnType, ReturnTypeDesc};

impl FromField for FuncDesc {
    fn from_field(field: &syn::Field) -> darling::Result<Self> {
        let field_ident = ident_option_to_darling_err(field.ident.as_ref())?;

        let add_in_func_attr: Vec<&Attribute> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("add_in_func"))
            .collect();
        if add_in_func_attr.is_empty() {
            return Err(
                darling::Error::custom("Field must have `add_in_func` attribute")
                    .with_span(field_ident),
            );
        } else if add_in_func_attr.len() > 1 {
            return Err(
                darling::Error::custom("Field can have only 1 `add_in_func` attribute")
                    .with_span(field_ident),
            );
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
            return Err(
                darling::Error::custom("Field can have at most 1 `returns` attribute")
                    .with_span(field_ident),
            );
        };
        let returns_attr = returns_attrs.get(0).copied();

        let func_meta = FuncHeadMeta::from_meta(&add_in_func_attr.meta)?;
        let params_meta = arg_attrs
            .iter()
            .map(|attr| FuncArgumentMeta::from_meta(&attr.meta))
            .collect::<darling::Result<Vec<FuncArgumentMeta>>>()?;
        let return_meta = returns_attr
            .map(|attr| FuncReturnMeta::from_meta(&attr.meta))
            .transpose()?;

        let mut params = params_meta
            .into_iter()
            .map(FuncArgumentDesc::try_from)
            .map(|res| res.map_err(|_| darling::Error::custom("Invalid argument type")))
            .collect::<Result<Vec<FuncArgumentDesc>, darling::Error>>()?;

        let return_value = match return_meta {
            Some(return_meta) => FuncReturnDesc::try_from(return_meta)
                .map_err(|_| darling::Error::custom("Invalid argument type"))?,
            None => FuncReturnDesc {
                ty: ReturnType::None,
                result: false,
            },
        };

        let syn::Type::BareFn(bare_fn) = &field.ty else {
            return Err(darling::Error::custom("AddIn functions must have bare `fn` type")
                .with_span(field_ident));
        };

        if let Some(first_input) = bare_fn.inputs.first() {
            let arg_tkn_stream: TokenStream = first_input.to_token_stream();

            let reference: syn::TypeReference = syn::parse2(arg_tkn_stream.clone())?;

            if arg_tkn_stream
                .into_iter()
                .filter(|t| t.to_string() == "Self")
                .count()
                == 1
            {
                params.insert(
                    0,
                    FuncArgumentDesc {
                        ty: ParamType::SelfType,
                        default: None,
                        out_param: reference.mutability.is_some(),
                    },
                )
            };
        };

        let name_literal = str_literal_token(&func_meta.name, field_ident)?;
        let name_ru_literal = str_literal_token(&func_meta.name_ru, field_ident)?;

        Ok(Self {
            ident: field_ident.to_owned(),

            name: func_meta.name,
            name_ru: func_meta.name_ru,

            name_literal,
            name_ru_literal,

            params,
            return_value: ReturnTypeDesc {
                ty: return_value.ty,
                result: return_value.result,
            },
        })
    }
}

#[derive(FromMeta, Debug)]
struct FuncHeadMeta {
    name: String,
    name_ru: String,
}

#[derive(FromMeta, Debug)]
struct FuncArgumentMeta {
    ty: ParamType,
    default: Option<Expr>,
    #[allow(dead_code)]
    as_in: Option<()>,
    as_out: Option<()>,
}

impl TryFrom<FuncArgumentMeta> for FuncArgumentDesc {
    type Error = ErrorConvertingMeta;

    fn try_from(arg_meta: FuncArgumentMeta) -> Result<Self, Self::Error> {
        if arg_meta.as_in.is_some() && arg_meta.as_out.is_some() {
            return Err(Self::Error::ConflictingParams(
                "as_in".to_string(),
                "as_out".to_string(),
            ));
        }
        Ok(Self {
            ty: arg_meta.ty,
            default: arg_meta.default,
            out_param: arg_meta.as_out.is_some(),
        })
    }
}

pub struct FuncReturnDesc {
    pub ty: ReturnType,
    pub result: bool,
}

#[derive(FromMeta, Debug)]
struct FuncReturnMeta {
    ty: Option<ReturnType>,
    result: Option<()>,
}

impl TryFrom<FuncReturnMeta> for FuncReturnDesc {
    type Error = ErrorConvertingMeta;

    fn try_from(arg_meta: FuncReturnMeta) -> Result<Self, Self::Error> {
        Ok(Self {
            ty: match arg_meta.ty {
                Some(ty) => ty,
                None => ReturnType::None,
            },
            result: arg_meta.result.is_some(),
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

pub fn parse_functions(struct_data: &DataStruct) -> Result<Vec<FuncDesc>, darling::Error> {
    let mut functions_descriptions = vec![];

    // iterate over methods
    for field in &struct_data.fields {
        let has_add_in_func_attr = field
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("add_in_func"));
        if !has_add_in_func_attr {
            continue;
        };

        let func_desc = FuncDesc::from_field(field)?;
        functions_descriptions.push(func_desc);
    }

    Ok(functions_descriptions)
}
