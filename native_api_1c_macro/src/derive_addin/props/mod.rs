use proc_macro2::{Ident, TokenStream};

use super::parsers::ParamType;

pub mod collectors;
pub mod generate;
pub mod parse;

#[derive(Debug)]
pub struct PropDesc {
    pub ident: Ident,

    pub name_literal: TokenStream,
    pub name_ru_literal: TokenStream,

    pub readable: bool,
    pub writable: bool,
    pub ty: ParamType,
}
