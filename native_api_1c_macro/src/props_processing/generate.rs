use proc_macro2::TokenStream;
use quote::quote;

use super::PropType;

pub fn param_ty_to_ffi_set(
    param_type: &PropType,
    param_path: proc_macro2::TokenStream,
) -> TokenStream {
    match param_type {
        PropType::Bool => {
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Bool(inner_value) => self.#param_path = inner_value.clone(), }
        }
        PropType::I32 => {
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(inner_value) => self.#param_path = inner_value.clone(), }
        }
        PropType::F64 => {
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(inner_value) => self.#param_path = inner_value.clone(), }
        }
        PropType::String => {
            quote! {
                native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(inner_value) => self.#param_path
                    = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(inner_value).into(),
            }
        }
        PropType::Date => {
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(inner_value) => self.#param_path = inner_value.clone(), }
        }
        PropType::Blob => {
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Blob(inner_value) => self.#param_path = inner_value.clone(), }
        }
    }
}
