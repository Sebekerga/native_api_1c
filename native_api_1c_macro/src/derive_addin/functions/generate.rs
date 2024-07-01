use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Ident;

use crate::derive_addin::utils::{expr_from_os_value, expr_to_os_value};

use super::{FuncArgumentDesc, FuncDesc, FuncParamType};

pub fn func_call_tkn(func: &FuncDesc, set_to: Option<&Ident>) -> TokenStream {
    let func_ident = func.ident.clone();

    let mut pre_call = quote! {};
    let mut func_args = quote! {};
    let mut post_call = quote! {};

    for (param_index, param_desc) in func.get_1c_params().iter().enumerate() {
        let param_ident = Ident::new(&format!("param_{}", param_index + 1), Span::call_site());

        let (pre_call_param, post_call_param) =
            gen_param_prep(param_desc, param_index, &param_ident);

        if func_args.is_empty() {
            func_args.extend(quote! {#param_ident})
        } else {
            func_args.extend(quote! {, #param_ident});
        }

        pre_call.extend(pre_call_param);
        post_call.extend(post_call_param);
    }

    if func.has_self_param() {
        if func_args.is_empty() {
            func_args = quote! {self};
        } else {
            func_args = quote! {self, #func_args};
        }
    }

    let mut func_call = quote! {
        let call_result = (self.#func_ident)(#func_args);
    };

    if func.return_value.result {
        func_call.extend(quote! {
            if call_result.is_err() {
                return Err(());
            }
            let call_result = call_result.unwrap();
        });
    };

    if let Some(set_to) = set_to {
        let return_ty = func.return_value.ty.clone().unwrap();
        let result_wrap = expr_to_os_value(&quote! { call_result }, &return_ty, true);
        func_call.extend(quote! {
            let #set_to
        });
        func_call.extend(quote! { = });
        func_call.extend(quote! {#result_wrap;});
    }

    quote! {
        #pre_call
        #func_call
        #post_call
    }
}

fn gen_param_prep(
    param: &FuncArgumentDesc,
    param_index: usize,
    param_ident: &Ident,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let FuncParamType::PlatformType(param_ty) = &param.ty else {
        panic!("SelfType is not allowed here");
    };

    let param_unwrap = expr_from_os_value(&quote! { params[#param_index]}, param_ty);
    let mut pre_call = quote! {;
        let #param_ident = #param_unwrap;
        let mut #param_ident = #param_ident.clone().into();
    };
    if param.out_param {
        pre_call.extend(quote! {
            let #param_ident = &mut #param_ident;
        });
    }

    let post_call = if !param.out_param {
        quote! {}
    } else {
        let param_wrap = expr_to_os_value(&param_ident.to_token_stream(), param_ty, false);
        let mut q = quote! {
            params[#param_index]
        };
        q.extend(quote! { = });
        q.extend(quote! { #param_wrap; });
        q
    };

    (pre_call, post_call)
}
