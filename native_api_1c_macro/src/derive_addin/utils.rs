use proc_macro2::LexError;
use syn::{spanned::Spanned, Ident};

pub mod macros {
    macro_rules! tkn_err_inner {
        ($str:expr, $span:expr) => {{
            let err_inner: darling::Error = darling::Error::custom($str).with_span($span);
            err_inner
        }};
    }

    macro_rules! tkn_err {
        ($str:expr, $span:expr) => {
            Err(crate::derive_addin::utils::macros::tkn_err_inner!(
                $str, $span
            ))
        };
    }

    pub(crate) use tkn_err;
    pub(crate) use tkn_err_inner;
}

const IDENT_OPTION_ERR: &str = "Unable to get ident from option";

pub fn ident_option_to_darling_err(ident: Option<&Ident>) -> Result<&Ident, darling::Error> {
    ident.ok_or_else(|| darling::Error::custom(IDENT_OPTION_ERR))
}

pub fn str_literal_token<T>(
    str_literal: &str,
    err_ident: &T,
) -> Result<proc_macro2::TokenStream, darling::Error>
where
    T: Spanned,
{
    format!(r#""{}""#, str_literal)
        .parse()
        .map_err(|e: LexError| {
            darling::Error::custom(format!("Unable to parse string literal: {}", e))
                .with_span(err_ident)
        })
}
