mod parse;

use darling::FromMeta;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};

use parse::ExternAddInsDesc;
use syn::{LitByte, LitChar, LitStr};

static ASCII_LOWER: [char; 50] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', // numbers
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', // ASCII uppercase
    'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', // ASCII uppercase
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', // ASCII lowercase
    'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', // ASCII lowercase
];

pub fn extern_functions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let extern_add_ins = syn::parse_macro_input!(input as ExternAddInsDesc);

    if extern_add_ins.components.len() > ASCII_LOWER.len() {
        return quote! {
            compile_error!("Too many components, max is {}", ASCII_LOWER.len());
        }
        .into();
    }

    let mut get_class_object_body = TokenStream::new();
    for (i, add_in_desc) in extern_add_ins.components.iter().enumerate() {
        let alias = ASCII_LOWER[i];
        let alias_literal = LitByte::new(alias as u8, Span::call_site());
        let alias_literal = alias_literal.to_token_stream();
        let init_tkn = &add_in_desc.init_tkn;

        get_class_object_body.extend(quote! {
            #alias_literal => {
                let add_in = #init_tkn;
                native_api_1c::native_api_1c_core::ffi::create_component(component, add_in)
            },
        })
    }
    let get_class_object_body = quote! {
        match *name as u8 {
            #get_class_object_body
            _ => 0,
        }
    };

    let names = ASCII_LOWER[..extern_add_ins.components.len()]
        .iter()
        .map(char::to_string)
        .collect::<Vec<_>>()
        .join("|");
    let names_lit = LitStr::new(&names, Span::call_site());
    let names_lit = names_lit.to_token_stream();
    let get_class_names_body = quote! { utf16_lit::utf16_null!(#names_lit).as_ptr() };

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

    result.into()
}
