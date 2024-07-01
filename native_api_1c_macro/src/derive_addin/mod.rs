use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use functions::{collectors::*, parse::parse_functions};
use props::{collectors::*, parse::parse_props};
use utils::{macros::tkn_err, str_literal_token};

mod constants;
mod functions;
mod parsers;
mod props;
mod utils;

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match derive_result(&derive_input) {
        Ok(tokens) => tokens.into(),
        Err(tokens) => tokens.into(),
    }
}

fn derive_result(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let impl_block = build_impl_block(input).map_err(|darling_error| {
        let error_tokens = darling_error.write_errors();
        let error_tokens = quote! {
            compile_error!(#error_tokens);
        };
        error_tokens
    })?;

    Ok(quote! {
        #impl_block
    })
}

fn build_impl_block(input: &DeriveInput) -> Result<proc_macro2::TokenStream, darling::Error> {
    let struct_ident = &input.ident;
    let syn::Data::Struct(struct_data) = &input.data else {
        return tkn_err!(
            "AddIn can only be derived for structs",
            &struct_ident.span()
        );
    };
    let add_in_name_literal = str_literal_token(&struct_ident.to_string(), struct_ident)?;

    let props = parse_props(struct_data)?;
    let functions = parse_functions(struct_data)?;

    let pi = props.iter().enumerate();
    let prop_definitions = [
        pi.clone().collect::<FindPropCollector>().release()?,
        pi.clone().collect::<GetNPropsCollector>().release()?,
        pi.clone().collect::<GetPropNameCollector>().release()?,
        pi.clone().collect::<IsPropReadableCollector>().release()?,
        pi.clone().collect::<IsPropWritableCollector>().release()?,
        pi.clone().collect::<GetPropValCollector>().release()?,
        pi.clone().collect::<SetPropValCollector>().release()?,
    ];

    let fi = functions.iter().enumerate();
    let func_definitions = [
        fi.clone().collect::<FindMethodCollector>().release()?,
        fi.clone().collect::<GetMethodNameCollector>().release()?,
        fi.clone().collect::<GetNMethodsCollector>().release()?,
        fi.clone().collect::<GetNParamsCollector>().release()?,
        fi.clone().collect::<HasReturnValueCollector>().release()?,
        fi.clone().collect::<CallAsProcCollector>().release()?,
        fi.clone().collect::<CallAsFuncCollector>().release()?,
        fi.clone()
            .collect::<GetParamDefValueCollector>()
            .release()?,
    ];

    let result = quote! {
        impl native_api_1c::native_api_1c_core::interface::AddInWrapper for #struct_ident {
            fn init(&mut self, interface: &'static native_api_1c::native_api_1c_core::ffi::connection::Connection) -> bool {
                self.connection = std::sync::Arc::new(Some(interface));
                true
            }

            fn get_info(&self) -> u16 {
                2000
            }
            fn done(&mut self) {}
            fn register_extension_as(&mut self) -> &[u16] {
                &utf16_lit::utf16_null!(#add_in_name_literal)
            }

            #(#prop_definitions)*
            #(#func_definitions)*

            fn set_locale(&mut self, loc: &[u16]) {
            }
            fn set_user_interface_language_code(&mut self, lang: &[u16]) {
            }
        }
    };
    Ok(result)
}
