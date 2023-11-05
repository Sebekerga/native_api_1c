use common_generators::{param_ty_to_ffi_return, SettableTypes};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_macro_input, DeriveInput};

use function_processing::{generate::func_call_tkn, parse::parse_functions, ParamType, ReturnType};
use props_processing::{generate::param_ty_to_ffi_set, parse::parse_props};
use utils::{
    macros::{tkn_err, tkn_err_inner},
    str_literal_token,
};

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

    let number_of_props = props.len();
    let number_of_func = functions.len();

    let mut find_prop_body = quote! {};
    let mut get_prop_name_body = quote! {};
    let mut is_prop_readable_body = quote! {};
    let mut is_prop_writable_body = quote! {};
    let mut get_prop_val_body = quote! {};
    let mut set_prop_val_body = quote! {};

    for prop in &props {
        let prop_ident = &prop.ident;
        let name_literal = str_literal_token(&prop.name, struct_ident)?;
        let name_ru_literal = str_literal_token(&prop.name_ru, struct_ident)?;
        let readable = prop.readable;
        let writable = prop.writable;
        let prop_index = props.iter().position(|p| p.name == prop.name).unwrap();

        find_prop_body = quote! {
            #find_prop_body
            if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_literal) == name { return Some(#prop_index) };
            if native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_ru_literal) == name { return Some(#prop_index) };
        };
        get_prop_name_body = quote! {
            #get_prop_name_body
            if num == #prop_index && alias == 0 { return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_literal).into()) };
            if num == #prop_index { return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_ru_literal).into()) };
        };
        is_prop_readable_body = quote! {
            #is_prop_readable_body
            if num == #prop_index { return #readable };
        };
        is_prop_writable_body = quote! {
            #is_prop_writable_body
            if num == #prop_index { return #writable };
        };

        if readable {
            let prop_settable: SettableTypes = (&prop.ty).try_into().map_err(|_| {
                tkn_err_inner!(
                    "Incorrectly attempted to convert type to settable",
                    &prop.ident.span()
                )
            })?;

            let ffi_set_tkn =
                param_ty_to_ffi_return(&prop_settable, quote! { val }, quote! {self.#prop_ident});
            get_prop_val_body = quote! {
                #get_prop_val_body
                if num == #prop_index {
                    #ffi_set_tkn;
                    return true;
                };
            };
        };

        if writable {
            let prop_set_tkn = param_ty_to_ffi_set(&prop.ty, quote! { #prop_ident });
            set_prop_val_body = quote! {
                #set_prop_val_body
                if num == #prop_index {
                    match val {
                        #prop_set_tkn
                        _ => return false,
                    }
                    return true;
                };
            };
        }
    }

    let mut find_func_body = quote! {};
    let mut get_func_name_body = quote! {};
    let mut has_ret_val_body = quote! {};
    let mut get_n_params_body = quote! {};
    let mut call_as_proc_body = quote! {};
    let mut call_as_func_body = quote! {};
    let mut get_param_def_value_body = quote! {};

    for func in &functions {
        let name_literal = str_literal_token(&func.name, struct_ident)?;
        let name_ru_literal = str_literal_token(&func.name_ru, struct_ident)?;
        let has_ret_val = !matches!(func.return_value.ty, ReturnType::None);
        let func_index = functions.iter().position(|p| p.name == func.name).unwrap();
        let number_of_params = func
            .params
            .iter()
            .filter(|p| !matches!(p.ty, ParamType::SelfType))
            .count();

        get_func_name_body = quote! {
            #get_func_name_body
            if num == #func_index && alias == 0 { return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_literal).into()) };
            if num == #func_index { return Some(native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(#name_ru_literal).into()) };
        };
        has_ret_val_body = quote! {
            #has_ret_val_body
            if method_num == #func_index { return #has_ret_val };
        };
        get_n_params_body = quote! {
            #get_n_params_body
            if num == #func_index { return #number_of_params };
        };

        let call_proc = func_call_tkn(func, None);
        call_as_proc_body = quote! {
            #call_as_proc_body
            if method_num == #func_index {
                #call_proc
                return true;
            };
        };

        if has_ret_val {
            let return_val_ident = Ident::new("val", proc_macro2::Span::call_site());
            let call_func = func_call_tkn(func, Some(&return_val_ident));
            call_as_func_body = quote! {
                #call_as_func_body
                if method_num == #func_index {
                    #call_func
                    return true;
                };
            };
        }

        let mut this_get_param_def_value_body = quote! {};
        for (i, arg_desc) in func.params.iter().enumerate() {
            match &arg_desc.default {
                Some(expr) => {
                    let prop_settable: SettableTypes = (&arg_desc.ty).try_into().map_err(|_| {
                        tkn_err_inner!(
                            "Incorrectly attempted to convert type to settable",
                            &func.ident.span()
                        )
                    })?;
                    let value_setter = param_ty_to_ffi_return(
                        &prop_settable,
                        quote! { value },
                        expr.into_token_stream(),
                    );
                    this_get_param_def_value_body = quote! {
                        #this_get_param_def_value_body
                        if param_num == #i  {
                            #value_setter;
                            return true;
                        }
                    }
                }
                None => {}
            }
        }
        get_param_def_value_body = quote! {
            #get_param_def_value_body
            if method_num == #func_index {
                #this_get_param_def_value_body
                return false;
            };
        };
    }

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
            fn get_n_props(&self) -> usize {
                #number_of_props
            }
            fn find_prop(&self, name: &[u16]) -> Option<usize> {
                #find_prop_body
                None
            }
            fn get_prop_name(&self, num: usize, alias: usize) -> Option<Vec<u16>> {
                #get_prop_name_body
                None
            }
            fn get_prop_val(&self, num: usize, val: native_api_1c::native_api_1c_core::ffi::provided_types::ReturnValue) -> bool {
                #get_prop_val_body
                false
            }
            fn set_prop_val(&mut self, num: usize, val: &native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue) -> bool {
                #set_prop_val_body
                false
            }
            fn is_prop_readable(&self, num: usize) -> bool {
                #is_prop_readable_body
                false
            }
            fn is_prop_writable(&self, num: usize) -> bool {
                #is_prop_writable_body
                false
            }
            fn get_n_methods(&self) -> usize {
                #number_of_func
            }
            fn find_method(&self, name: &[u16]) -> Option<usize> {
                #find_func_body
                None
            }
            fn get_method_name(&self, num: usize, alias: usize) -> Option<Vec<u16>> {
                #get_func_name_body
                None
            }
            fn get_n_params(&self, num: usize) -> usize {
                #get_n_params_body
                0
            }
            fn get_param_def_value(
                &self,
                method_num: usize,
                param_num: usize,
                value: native_api_1c::native_api_1c_core::ffi::provided_types::ReturnValue,
            ) -> bool {
                #get_param_def_value_body
                false
            }
            fn has_ret_val(&self, method_num: usize) -> bool {
                #has_ret_val_body
                false
            }
            fn call_as_proc(
                &mut self,
                method_num: usize,
                params: &mut [native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue],
            ) -> bool {
                #call_as_proc_body
                false
            }
            fn call_as_func(
                &mut self,
                method_num: usize,
                params: &mut [native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue],
                val: native_api_1c::native_api_1c_core::ffi::provided_types::ReturnValue,
            ) -> bool {
                #call_as_func_body
                false
            }
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
