use std::vec;

use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Ident;

use crate::derive_addin::utils::{expr_from_os_value, expr_to_os_value};

use super::{FuncArgumentDesc, FuncDesc, FuncParamType};

pub fn func_call_tkn(func: &FuncDesc, set_to: Option<&Ident>) -> TokenStream {
    let func_ident = func.ident.clone();

    let mut tkn_func_args = vec![];
    if func.has_self_param() {
        tkn_func_args.push(quote! {self});
    }

    let mut tkn_param_pre_calls = vec![];
    let mut tkn_param_post_calls = vec![];

    for (param_index, param_desc) in func.get_1c_params().iter().enumerate() {
        let param_ident = Ident::new(&format!("param_{}", param_index + 1), Span::call_site());

        tkn_param_pre_calls.push(param_pre_call_tkn(param_desc, param_index, &param_ident));
        tkn_param_post_calls.push(param_post_call_tkn(param_desc, param_index, &param_ident));
        tkn_func_args.push(param_ident.to_token_stream());
    }

    let tkn_func_call_result_handling = if func.return_value.result {
        quote! {
            if call_result.is_err() {
                return Err(());
            }
            let call_result = call_result.unwrap();
        }
    } else {
        quote! {}
    };

    let tkn_func_call_set_return = if let Some(set_to) = set_to {
        let return_ty = func.return_value.ty.clone().unwrap();
        let tkn_result_unwrap = expr_to_os_value(&quote! { call_result }, &return_ty, true);
        quote! {
            let #set_to = #tkn_result_unwrap;
        }
    } else {
        quote! {}
    };

    quote! {
        // prepare parameters for native function call
        #(#tkn_param_pre_calls)*

        let call_result = (self.#func_ident)(#(#tkn_func_args),*);
        // handle Result if needed
        #tkn_func_call_result_handling
        // set return value if needed
        #tkn_func_call_set_return

        // handle post call, like out parameters
        #(#tkn_param_post_calls)*
    }
}

fn param_pre_call_tkn(
    param: &FuncArgumentDesc,
    param_index: usize,
    param_ident: &Ident,
) -> proc_macro2::TokenStream {
    let FuncParamType::PlatformType(param_ty) = &param.ty else {
        panic!("SelfType is not allowed here");
    };

    let tkn_param_unwrap = expr_from_os_value(&quote! { params[#param_index]}, param_ty);
    let tkn_decouple_param = if param.out_param {
        quote! {
            let mut #param_ident = #param_ident.clone().into();
            let #param_ident = &mut #param_ident;
        }
    } else {
        quote! {
            let #param_ident = #param_ident.clone().into();
        }
    };

    quote! {
        // extract from given params by index, storing as ref
        let #param_ident = #tkn_param_unwrap;
        // store as owned value with needed conversions from ParamValue to target Rust type
        // if out param, make it a mutable reference
        #tkn_decouple_param
    }
}

fn param_post_call_tkn(
    param: &FuncArgumentDesc,
    param_index: usize,
    param_ident: &Ident,
) -> proc_macro2::TokenStream {
    let FuncParamType::PlatformType(param_ty) = &param.ty else {
        panic!("SelfType is not allowed here");
    };

    if !param.out_param {
        return quote! {};
    }

    let tkn_param_wrap = expr_to_os_value(&param_ident.to_token_stream(), param_ty, false);

    quote! {
        params[#param_index] = #tkn_param_wrap;
    }
}
