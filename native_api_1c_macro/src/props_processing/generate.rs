use quote::quote;

use super::PropType;

pub fn param_ty_to_ffi_set(
    param_type: &PropType,
    param_path: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, proc_macro2::TokenStream> {
    match param_type {
        PropType::Bool => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Bool(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        PropType::I32 => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        PropType::F64 => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        PropType::String => Ok(quote! {
            native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(inner_value) => self.#param_path
                = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(inner_value).into(),
        }),
        PropType::Date => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(inner_value) => self.#param_path = inner_value.clone(), },
        ),
        PropType::Blob => Ok(
            quote! { native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Blob(inner_value) => self.#param_path = inner_value.clone(), },
        ),
    }
}
