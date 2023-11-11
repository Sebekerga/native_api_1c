use darling::{FromField, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{spanned::Spanned, Attribute, DataStruct, Meta};

use crate::derive_addin::utils::{ident_option_to_darling_err, str_literal_token};

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
            .map(|res| res.map_err(|err| err.into()))
            .collect::<Result<Vec<FuncArgumentDesc>, darling::Error>>()?;

        let return_value = match return_meta {
            Some(return_meta) => FuncReturnDesc::try_from(return_meta),
            None => Ok(FuncReturnDesc {
                ty: ReturnType::None,
                result: false,
            }),
        };
        let return_value: Result<FuncReturnDesc, darling::Error> =
            return_value.map_err(|err| err.into());
        let return_value = return_value?;

        let syn::Type::BareFn(bare_fn) = &field.ty else {
            return Err(darling::Error::custom("AddIn functions must have bare `fn` type")
                .with_span(field_ident));
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
                            ty: ParamType::SelfType,
                            default: None,
                            out_param: reference.mutability.is_some(),
                        },
                    )
                };
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
    ident: Option<syn::Ident>,
    ty: ParamType,
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

        let allowed_defaults = match arg_meta.ty {
            ParamType::Bool => true,
            ParamType::I32 => true,
            ParamType::F64 => true,
            ParamType::String => true,
            ParamType::Date => false,
            ParamType::Blob => false,
            ParamType::SelfType => false,
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
    UnexpectedMetaType(Span),
    TypeCannotBeDefault(ParamType, Span),
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
