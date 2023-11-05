use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use function_processing::{collectors::*, parse::parse_functions};
use props_processing::{collectors::*, parse::parse_props};
use utils::{macros::tkn_err, str_literal_token};

mod common_generators;
mod constants;
mod function_processing;
mod props_processing;
mod utils;

#[proc_macro_derive(AddIn, attributes(add_in_prop, add_in_func, add_in_con, arg, returns))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match derive_result(&derive_input) {
        Ok(tokens) => tokens.into(),
        Err(tokens) => tokens.into(),
    }
}

fn derive_result(input: &DeriveInput) -> Result<TokenStream, TokenStream> {
    let extern_functions = build_extern_functions(input)?;
    let impl_block = build_impl_block(input).map_err(|darling_error| {
        let error_tokens = darling_error.write_errors();
        let error_tokens = quote! {
            compile_error!(#error_tokens);
        };
        error_tokens
    })?;

    Ok(quote! {
        #impl_block
        #extern_functions
    })
}

fn build_impl_block(input: &DeriveInput) -> Result<proc_macro2::TokenStream, darling::Error> {
    let struct_ident = &input.ident;
    let syn::Data::Struct(struct_data) = &input.data else {
        return tkn_err!("AddIn can only be derived for structs", &struct_ident.span());
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

fn build_extern_functions(input: &DeriveInput) -> Result<proc_macro2::TokenStream, TokenStream> {
    let struct_ident = &input.ident;
    let get_class_object_body = quote! {
        match *name as u8 {
            b'1' => {
                let add_in_1 = #struct_ident::new();
                native_api_1c::native_api_1c_core::ffi::create_component(component, add_in_1)
            },
            _ => 0,
        }
    };
    let get_class_names_body = quote! { utf16_lit::utf16_null!("1").as_ptr() };

    let result = quote! {
        pub static mut PLATFORM_CAPABILITIES: std::sync::atomic::AtomicI32 =
            std::sync::atomic::AtomicI32::new(-1);

        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn GetAttachType() -> native_api_1c::native_api_1c_core::ffi::AttachType {
            native_api_1c::native_api_1c_core::ffi::AttachType::Any
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn DestroyObject(component: *mut *mut std::ffi::c_void) -> std::ffi::c_long {
            native_api_1c::native_api_1c_core::ffi::destroy_component(component)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn GetClassObject(
            name: *const u16,
            component: *mut *mut std::ffi::c_void,
        ) -> std::ffi::c_long {
            #get_class_object_body
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub extern "C" fn GetClassNames() -> *const u16 {
            #get_class_names_body
        }
    };

    Ok(result)
}
