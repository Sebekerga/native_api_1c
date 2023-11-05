use proc_macro2::Span;
use proc_macro2::{LexError, TokenStream};
use syn::Ident;

use self::macros::tkn_err_inner;

pub mod macros {
    macro_rules! tkn_err_inner {
        ($str:expr, $span:expr) => {{
            let err_inner: proc_macro2::TokenStream =
                syn::Error::new($span, $str).to_compile_error().into();
            err_inner
        }};
    }

    macro_rules! tkn_err {
        ($str:expr, $span:expr) => {
            Err(crate::utils::macros::tkn_err_inner!($str, $span))
        };
    }

    pub(crate) use tkn_err;
    pub(crate) use tkn_err_inner;
}

const IDENT_OPTION_ERR: &str = "Unable to get ident from option";

pub fn ident_option_to_token_err(ident: Option<&Ident>) -> Result<&Ident, TokenStream> {
    ident.ok_or(tkn_err_inner!(IDENT_OPTION_ERR, Span::call_site()))
}

pub fn ident_option_to_darling_err(ident: Option<&Ident>) -> Result<&Ident, darling::Error> {
    ident.ok_or_else(|| darling::Error::custom(IDENT_OPTION_ERR))
}

pub fn str_literal_token(
    str_literal: &str,
    err_ident: &Ident,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    let token: Result<TokenStream, TokenStream> =
        format!(r#""{}""#, str_literal)
            .parse()
            .map_err(|e: LexError| {
                let token2: TokenStream =
                    Err(syn::Error::new(err_ident.span(), format!("LexErr: {}", e))
                        .to_compile_error())
                    .unwrap();
                token2
            });
    token
}
