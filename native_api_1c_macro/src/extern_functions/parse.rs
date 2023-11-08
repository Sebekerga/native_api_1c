use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parse::{Parse, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    token::Comma,
    Attribute,
};

#[derive(Debug)]
pub struct ExternAddInsDesc {
    pub components: Vec<ExternAddInComponentDesc>,
}

impl Parse for ExternAddInsDesc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let parser = Punctuated::<syn::Expr, Comma>::parse_terminated(input).unwrap();

        let components = parser
            .into_iter()
            .map(|expr| {
                let input: proc_macro::TokenStream = expr.to_token_stream().into();
                ExternAddInComponentDesc::parse.parse(input)
            })
            .collect::<syn::Result<Vec<_>>>()?;

        Ok(ExternAddInsDesc { components })
    }
}

#[derive(FromMeta, Debug)]
struct ExternAddInComponentMeta {
    #[darling(rename = "name")]
    name_override: Option<String>,
}

#[derive(Debug)]
pub struct ExternAddInComponentDesc {
    pub name_override: Option<String>,
    pub init_tkn: TokenStream,
}

impl Parse for ExternAddInComponentDesc {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer).unwrap();
        let add_in_component_attrs = attrs
            .iter()
            .filter(|attr| attr.path().is_ident("add_in_component"))
            .collect::<Vec<_>>();
        if add_in_component_attrs.len() > 1 {
            return Err(syn::Error::new(
                add_in_component_attrs[1].span(),
                "at most one `add_in_component` attribute is allowed",
            ));
        }
        let attr = add_in_component_attrs.first();
        let addin_desc = attr.map(|attr| ExternAddInComponentMeta::from_meta(&attr.meta));
        let addin_desc = match addin_desc {
            Some(Ok(desc)) => Some(desc),
            Some(Err(err)) => {
                return Err(syn::Error::new(
                    add_in_component_attrs[1].span(),
                    err.to_string(),
                ))
            }
            None => None,
        };

        let init_tkn = input.call(TokenStream::parse).unwrap();

        Ok(ExternAddInComponentDesc {
            name_override: addin_desc.and_then(|desc| desc.name_override),
            init_tkn,
        })
    }
}
