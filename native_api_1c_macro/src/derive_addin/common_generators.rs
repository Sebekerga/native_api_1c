use quote::quote;

pub enum SettableTypes {
    Bool,
    I32,
    F64,
    String,
    Date,
    Blob,
}

pub fn param_ty_to_ffi_return(
    param_type: &SettableTypes,
    target: proc_macro2::TokenStream,
    source: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    match param_type {
        SettableTypes::Bool => quote! { #target.set_bool(#source.into()) },
        SettableTypes::I32 => quote! { #target.set_i32(#source.into()) },
        SettableTypes::F64 => quote! { #target.set_f64(#source.into()) },
        SettableTypes::String => {
            quote! { #target.set_str(&native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(String::from(#source.clone()).as_str())) }
        }
        SettableTypes::Date => quote! { #target.set_date(#source.into()) },
        SettableTypes::Blob => quote! { #target.set_blob(&#source) },
    }
}
