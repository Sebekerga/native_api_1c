use crate::types_1c::ParamType;
use syn::Ident;

pub mod generate;
pub mod parse;

pub struct PropDesc {
    pub ident: Ident,
    pub name: String,
    pub name_ru: String,
    pub readable: bool,
    pub writable: bool,
    pub ty: ParamType,
}
