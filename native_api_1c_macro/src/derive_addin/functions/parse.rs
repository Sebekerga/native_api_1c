use darling::{FromField, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, DataStruct, Meta};

use crate::derive_addin::{
    parsers::{ParamType, PropName},
    utils::ident_option_to_darling_err,
};

use super::{FuncArgumentDesc, FuncDesc, FuncParamType, ReturnTypeDesc};

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
        let returns_attr = returns_attrs.first().copied();

        let func_meta = FuncHeadMeta::from_meta(&add_in_func_attr.meta)?;
        let params_meta = arg_attrs
            .iter()
            .map(|attr| FuncArgumentMeta::from_meta(&attr.meta))
            .collect::<darling::Result<Vec<FuncArgumentMeta>>>()?;
        let return_meta = returns_attr
            .map(|attr| FuncReturnMeta::from_meta(&attr.meta))
            .transpose()?;
        let return_value = match return_meta {
            Some(meta) => ReturnTypeDesc::from(meta),
            None => ReturnTypeDesc {
                ty: None,
                result: false,
            },
        };

        let mut params = params_meta
            .into_iter()
            .map(FuncArgumentDesc::try_from)
            .map(|res| res.map_err(|err| err.into()))
            .collect::<Result<Vec<FuncArgumentDesc>, darling::Error>>()?;

        let syn::Type::BareFn(bare_fn) = &field.ty else {
            return Err(
                darling::Error::custom("AddIn functions must have bare `fn` type")
                    .with_span(field_ident),
            );
        };

        if let Some(first_input) = bare_fn.inputs.first() {
            let arg_tkn_stream: TokenStream = first_input.to_token_stream();

            if let Ok(reference) = syn::parse2::<syn::TypeReference>(arg_tkn_stream.clone()) {
                if arg_tkn_stream
                    .into_iter()
                    .filter(|t| t.to_string() == "Self")
                    .count()
                    == 1
                {
                    params.insert(
                        0,
                        FuncArgumentDesc {
                            ty: FuncParamType::SelfType,
                            default: None,
                            out_param: reference.mutability.is_some(),
                        },
                    )
                };
            };
        };

        Ok(Self {
            ident: field_ident.to_owned(),

            name_literal: func_meta.name.into(),
            name_ru_literal: func_meta.name_ru.into(),

            params,
            return_value,
        })
    }
}

#[derive(FromMeta, Debug)]
struct FuncHeadMeta {
    name: PropName,
    name_ru: PropName,
}

#[derive(FromMeta, Debug)]
struct FuncArgumentMeta {
    ident: Option<syn::Ident>,
    ty: FuncParamType,
    default: Option<Meta>,
    #[allow(dead_code)]
    as_in: Option<()>,
    as_out: Option<()>,
}

impl TryFrom<FuncArgumentMeta> for FuncArgumentDesc {
    type Error = ErrorConvertingMeta;

    fn try_from(arg_meta: FuncArgumentMeta) -> Result<Self, Self::Error> {
        if arg_meta.as_in.is_some() && arg_meta.as_out.is_some() {
            return Err(Self::Error::ConflictingParams(
                arg_meta.ident.span(),
                "as_in".to_string(),
                "as_out".to_string(),
            ));
        }

        let allowed_defaults = match arg_meta.ty.clone() {
            FuncParamType::SelfType => false,
            FuncParamType::PlatformType(ty) => match ty {
                ParamType::Bool => true,
                ParamType::I32 => true,
                ParamType::F64 => true,
                ParamType::String => true,
                ParamType::Date => false,
                ParamType::Blob => false,
            },
        };

        if arg_meta.default.is_some() && !allowed_defaults {
            return Err(Self::Error::TypeCannotBeDefault(
                arg_meta.ty,
                arg_meta.default.span(),
            ));
        }

        // if you pass "some_string" as default, it would get parsed by darling as `Ident`
        let default_fixed = arg_meta.default.map(|d| match d {
            Meta::NameValue(nv) => Ok(nv.value.to_token_stream()),
            _ => Err(Self::Error::UnexpectedMetaType(arg_meta.ident.span())),
        });
        let default_fixed = default_fixed.transpose()?;

        Ok(Self {
            ty: arg_meta.ty,
            default: default_fixed,
            out_param: arg_meta.as_out.is_some(),
        })
    }
}

#[derive(FromMeta, Debug)]
struct FuncReturnMeta {
    ty: Option<ParamType>,
    result: Option<()>,
}

impl From<FuncReturnMeta> for ReturnTypeDesc {
    fn from(arg_meta: FuncReturnMeta) -> Self {
        Self {
            ty: arg_meta.ty,
            result: arg_meta.result.is_some(),
        }
    }
}

pub enum ErrorConvertingMeta {
    UnexpectedMetaType(Span),
    TypeCannotBeDefault(FuncParamType, Span),
    InvalidTypeForParam(Span, String),
    InvalidTypeForReturn(Span, String),
    ConflictingParams(Span, String, String),
}

impl From<ErrorConvertingMeta> for darling::Error {
    fn from(err: ErrorConvertingMeta) -> Self {
        match err {
            ErrorConvertingMeta::InvalidTypeForParam(span, ty) => {
                let joined_allowed_types = crate::derive_addin::constants::ALL_ARG_TYPES.join(", ");
                darling::Error::custom(format!(
                    "Invalid type: `{ty}`. Must be one of: {joined_allowed_types}"
                ))
                .with_span(&span)
            }
            ErrorConvertingMeta::InvalidTypeForReturn(span, ty) => {
                let joined_allowed_types =
                    crate::derive_addin::constants::ALL_RETURN_TYPES.join(", ");
                darling::Error::custom(format!(
                    "Invalid type: `{ty}`. Must be one of: {joined_allowed_types}"
                ))
                .with_span(&span)
            }
            ErrorConvertingMeta::ConflictingParams(span, param1, param2) => {
                darling::Error::custom(format!("Conflicting params: {} and {}", param1, param2))
                    .with_span(&span)
            }
            ErrorConvertingMeta::TypeCannotBeDefault(param_type, span) => {
                darling::Error::custom(format!("Type `{param_type}` cannot have default value"))
                    .with_span(&span)
            }
            ErrorConvertingMeta::UnexpectedMetaType(span) => {
                darling::Error::custom("Unexpected meta type").with_span(&span)
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
