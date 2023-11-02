use proc_macro2::Ident;
use syn::Expr;

use crate::types_1c::ParamType;

pub mod generate;
pub mod parse;

pub struct FuncDesc {
    pub ident: Ident,
    pub name: String,
    pub name_ru: String,
    pub params: Vec<FuncArgumentDesc>,
    pub return_value: (Option<ParamType>, bool),
}

pub struct FuncArgumentDesc {
    pub ty: ParamType,
    pub default: Option<Expr>,
    pub out_param: bool,
}
