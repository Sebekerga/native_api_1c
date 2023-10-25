use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{spanned::Spanned, ToTokens, quote};
use syn::{
    punctuated::Punctuated, Expr, Token, DataStruct, Attribute, Type, ReturnType, Ident,
};

use crate::{types_1c::ParamType, utils::macros::{tkn_err, tkn_err_inner}, constants::{ALL_RETURN_TYPES, BOOL_TYPE, I32_TYPE, F64_TYPE, UNTYPED_TYPE, STRING_TYPE, ALL_ARG_TYPES, NAME_ATTR, NAME_RU_ATTR, ARG_ATTR, RETURNS_ATTR, DEFAULT_ATTR, RESULT_ATTR, BLOB_TYPE, DATE_TYPE, OUT_PARAMETER_FLAG, IN_PARAMETER_FLAG}};

pub struct FuncDesc {
    pub ident: Ident,
    pub name: String,
    pub name_ru: String,
    pub params: Vec<ArgumentDesc>,
    pub return_value: (Option<ParamType>, bool),
}

pub struct ArgumentDesc {
    pub ty: ParamType,
    pub default: Option<Expr>,
    pub out_param: bool,
}

pub fn parse_functions(struct_data: &DataStruct) -> Result<Vec<FuncDesc>, TokenStream> {
    
    let mut functions_descriptions = vec![];    
    
    // iterate over methods
    for field in &struct_data.fields {
        let Some(attr) = field.attrs.get(0) else { 
            continue; 
        };
        if !attr.path().is_ident("add_in_func") {
            continue;
        };
        let Some(prop_ident) = field.ident.clone() else {
            return tkn_err!("AddIn props must have a name", field.ident.__span());
        };

        // parsing main `add_in_func` attribute 
        let func_desc = parse_add_in_func_attr(attr)?;
        let mut func_desc = FuncDesc {
            ident: prop_ident,
            name: func_desc.0,
            name_ru: func_desc.1,
            params: vec![],
            return_value: (None, false),
        };

        // checking if first argument is `self`
        let syn::Type::BareFn(bare_fn) = &field.ty else {
            return tkn_err!("AddIn functions must have bare `fn` type", field.ident.__span());
        };
        if let Some(first_input) = bare_fn.inputs.first(){
            if let Type::Reference(reference) =  &first_input.ty {
                if let Type::Path(path) = &*reference.elem {
                    if let Some(ident) = path.path.get_ident() {
                        if ident == "Self" {
                            func_desc.params.push(ArgumentDesc { ty: ParamType::SelfType, default: None, out_param: reference.mutability.is_some() })
                        }
                    }
                }
            };
        };

        // parsing `arg` attribute 
        let arg_iter = field.attrs.iter().filter(|attr| attr.path().is_ident(ARG_ATTR));
        for attr in arg_iter {
            func_desc.params.push(parse_arg_attr(attr)?);
        }

        // parsing `returns` attribute     
        let returns_iter = field.attrs.iter().filter(|attr| attr.path().is_ident(RETURNS_ATTR));
        if returns_iter.clone().count() > 1 {
            return tkn_err!("AddIn functions can have only 1 `returns` attribute", field.ident.__span());
        };
        if let Some(attr) = returns_iter.clone().next() {
            let (arg_ty, result) = parse_returns_attr(attr)?;
            func_desc.return_value = (arg_ty, result);
        };

        // check function definition
        let Type::BareFn(field_ty) = &field.ty else {
            return tkn_err!("AddIn functions must have bare `fn` type", field.ident.__span());
        };
        if matches!(field_ty.output, ReturnType::Default) && (func_desc.return_value.0.is_some() || func_desc.return_value.1) {
            return tkn_err!("AddIn functions must have a return type if `returns` attribute is specified", field.ident.__span());
        };  
        if func_desc.params.len() != field_ty.inputs.len() {
            return tkn_err!("AddIn functions must have the same number of arguments as `arg` attributes, except for first `&Self` or `&mut Self`", field.ident.__span());
        };    

        functions_descriptions.push(func_desc);
    }

    Ok(functions_descriptions)
}

fn parse_add_in_func_attr(attr: &Attribute) -> Result<(String, String), TokenStream> {
    let name_values: Punctuated<Expr, Token![,]> = attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err::<TokenStream, _>(|e| tkn_err_inner!(e.to_string(), attr.bracket_token.span.__span()))?;

    let args = name_values
        .iter()
        .flat_map(|exp| {
            let Expr::Assign(assign) = exp else { 
                return Some((exp.to_token_stream().to_string(), None, exp.__span())); 
            };
            let Expr::Lit(lit) = &*assign.right else { return None };
            let syn::Lit::Str(str_lit) = &lit.lit else { return None };
            Some((assign.left.to_token_stream().to_string(), Some(str_lit.value()), exp.__span()))
        });
    let Some(prop_name) = args
        .clone()
        .find(|(name, _, _)| name == NAME_ATTR) else {
            return tkn_err!("AddIn prop must have a `name` argument", attr.bracket_token.span.__span());
        };
    let Some(prop_name) = prop_name.1 else {
        return tkn_err!("AddIn prop argument `name` must be a string literal assignment: name = \"MyPropName\"", prop_name.2);
    };

    let Some(prop_name_ru) = args
        .clone()
        .find(|(name, _, _)| name == NAME_RU_ATTR) else {
            return tkn_err!("AddIn prop must have a `name_ru` argument", attr.bracket_token.span.__span());
        };
    let Some(prop_name_ru) = prop_name_ru.1 else {
        return tkn_err!("AddIn prop argument `name_ru` must be a string literal assignment: name_ru = \"МоеСвойство\"", prop_name_ru.2);
    };

    Ok((prop_name, prop_name_ru))
}

fn parse_arg_attr(attr: &Attribute) -> Result<ArgumentDesc, TokenStream> {
    let exprs: Punctuated<Expr, Token![,]> = attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err::<TokenStream, _>(|e| tkn_err_inner!(e.to_string(), attr.bracket_token.span.__span()))?; 

    let arg_ty = exprs.iter().find(|expr| {
        if matches!(expr, Expr::Assign(_)) { 
            return false;  
        };
        let expr = expr.to_token_stream().to_string();
        ALL_ARG_TYPES.contains(&expr.as_str())
    }).map(|expr| expr.to_token_stream().to_string());
    let Some(arg_ty_str) = arg_ty else {
        return tkn_err!("AddIn function attribute `arg` must have a type specified: `#[arg(TYPE, ...)]`, where type: Bool, Int, Float or Str", attr.bracket_token.span.__span());
    };

    let default = exprs.iter().find_map(|expr| {
        let Expr::Assign(assign) = expr else { 
            return None; 
        };
        let left = assign.left.to_token_stream().to_string();
        if &left != DEFAULT_ATTR {
            return None;
        };        
        
        Some((&*assign.right).to_owned())
    });

    if arg_ty_str == BLOB_TYPE && default.is_some() {
        return tkn_err!("AddIn function attribute `arg` of type `Blob` cannot have default value", attr.bracket_token.span.__span());
    };

    if arg_ty_str == DATE_TYPE && default.is_some() {
        return tkn_err!("AddIn function attribute `arg` of type `Date` cannot have default value", attr.bracket_token.span.__span());
    };

    let out_param = exprs.iter().any(|expr| {
        if let Expr::Assign(_assign) = expr { 
            return false; 
        };
        let expr = expr.to_token_stream().to_string();
        expr.as_str() == OUT_PARAMETER_FLAG
    });

    let in_param = exprs.iter().any(|expr| {
        if let Expr::Assign(_assign) = expr { 
            return false; 
        };
        let expr = expr.to_token_stream().to_string();
        expr.as_str() == IN_PARAMETER_FLAG
    });

    if out_param && in_param {
        return tkn_err!("AddIn function attribute `arg` cannot have both `in` and `out` flags", attr.bracket_token.span.__span());
    };
    
    Ok(ArgumentDesc {
        ty: match arg_ty_str.as_str() {
            BOOL_TYPE => ParamType::Bool,
            I32_TYPE => ParamType::I32,
            F64_TYPE => ParamType::F64,
            STRING_TYPE => ParamType::String,
            DATE_TYPE => ParamType::Date,
            BLOB_TYPE => ParamType::Blob,
            _ => unreachable!(),
        },
        default,
        out_param,
    })
}

fn parse_returns_attr(attr: &Attribute) -> Result<(Option<ParamType>, bool), TokenStream> {
    let exprs: Punctuated<Expr, Token![,]> = attr
        .parse_args_with(Punctuated::parse_terminated)
        .map_err::<TokenStream, _>(|e| tkn_err_inner!(e.to_string(), attr.bracket_token.span.__span()))?; 

    let arg_ty = exprs.iter().find(|expr| {
        if matches!(expr, Expr::Assign(_)) { 
            return false;  
        };
        let expr = expr.to_token_stream().to_string();
        ALL_RETURN_TYPES.contains(&expr.as_str())
    });
    let Some(arg_ty_str) = arg_ty else {
        return tkn_err!("AddIn function attribute `returns` must have a type specified: use `#[returns(None, ...)]` if doesn't return anything", attr.bracket_token.span.__span());
    };
    let arg_ty = match arg_ty_str.to_token_stream().to_string().as_str() {
        BOOL_TYPE => Some(ParamType::Bool),
        I32_TYPE => Some(ParamType::I32),
        F64_TYPE => Some(ParamType::F64),
        STRING_TYPE => Some(ParamType::String),
        DATE_TYPE => Some(ParamType::Date),
        BLOB_TYPE => Some(ParamType::Blob),
        UNTYPED_TYPE => None,
        _ => unreachable!(),
    };

    let result = exprs.iter().any(|expr| {
        if let Expr::Assign(_assign) = expr { 
            return false; 
        };
        let expr = expr.to_token_stream().to_string();
        expr.as_str() == RESULT_ATTR
    });

    Ok((arg_ty, result))        
}

pub fn func_call_tkn(
    func: &FuncDesc,
    return_value: bool,
) -> Result<proc_macro2::TokenStream, TokenStream> {
    let func_ident = &func.ident;
    let mut param_extract = quote! {};
    let mut pre_call = quote! {};
    let mut func_call = quote! {};
    let mut post_call = quote! {};

    func.params.iter().enumerate().for_each(|(counter,param)| {
        let param_ident = match param.ty {
            ParamType::SelfType => Ident::new("param_self", Span::call_site()),
            _ => {
                let ident = Ident::new(&format!("param_{}", counter), Span::call_site());
                let param_ident_raw = Ident::new(&format!("{}_raw", ident), Span::call_site());
                param_extract = quote! {
                    #param_extract ref mut #param_ident_raw,
                };
                ident
            },    
        };

        let (param_pre_call, param_post_call) = gen_param_prep(param, &param_ident);
        pre_call = quote! {
            #pre_call
            #param_pre_call
        };
        post_call = quote! {
            #post_call
            #param_post_call
        };

        match param.ty {
            ParamType::SelfType => 
            func_call = quote! {
                #func_call
                self,
            },
            _ => if param.out_param {
                func_call = quote! {
                    #func_call
                    #param_ident,
                };
            }
            else {
                func_call = quote! {
                    #func_call
                    #param_ident.clone().into(),
                };
            }
        }
    });    

    pre_call = quote! {
        let [#param_extract] = params else {
            return false;
        };
        #pre_call
    };

    let mut func_call = if func.return_value.1 {
        quote! {
            #pre_call
            let call_result = (self.#func_ident)(#func_call);
            let Ok(call_result) = call_result else { return false; };
            #post_call
        }
    }
    else {
        quote! {
            #pre_call
            let call_result = (self.#func_ident)(#func_call);
            #post_call
        }
    };

    if return_value {
        let value_setter = match &func.return_value.clone().0.unwrap() {
            ParamType::Bool => quote! { val.set_bool(call_result.into()); },
            ParamType::I32 => quote! { val.set_i32(call_result.into()); },
            ParamType::F64 => quote! { val.set_f64(call_result.into()); },
            ParamType::String => {
                quote! { val.set_str(&native_api_1c::native_api_1c_core::ffi::string_utils::os_string_nil(String::from(&call_result).as_str())); }
            }
            ParamType::Date => {
                quote! { val.set_date(call_result.into()); }
            }
            ParamType::Blob => {
                quote! { val.set_blob(&call_result); }
            }
            ParamType::SelfType => unreachable!("SelfType is never used in return params"),
        };
        func_call = quote! {
            #func_call
            #value_setter
        };
    }

    Ok(func_call)
}

fn gen_param_prep(param: &ArgumentDesc, param_ident: &Ident) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let param_ident_ref = Ident::new(&format!("{}_ref", param_ident), Span::call_site());
    let param_ident_raw = Ident::new(&format!("{}_raw", param_ident), Span::call_site());
    
    let mut pre_call = quote! {};
    let mut post_call = quote! {};

    match param.ty {
        ParamType::Bool => {
            pre_call = quote! {
                #pre_call
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Bool(#param_ident) 
                = #param_ident_raw else { 
                    return false; 
                };
            };
        }
        ParamType::I32 => {
            pre_call = quote! {
                #pre_call
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(#param_ident) 
                = #param_ident_raw else { 
                    return false; 
                };
            };
        }
        ParamType::F64 => {
            if param.out_param {
                pre_call = quote! {
                    #pre_call
                    let mut #param_ident_ref = match #param_ident_raw {
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(val) => *val,
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(val) => *val as f64,
                        _ => return false,
                    };
                    let #param_ident = &mut #param_ident_ref;
                };
                post_call = quote!{
                    #post_call
                    *#param_ident_raw = native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(*#param_ident);
                };
            } else {
                pre_call = quote! {
                    #pre_call
                    let #param_ident = match #param_ident_raw {
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::F64(val) => *val,
                        native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::I32(val) => *val as f64,
                        _ => return false,
                    };
                };
            }
        }
        ParamType::String => {
            if param.out_param {
                let param_ident_str = Ident::new(&format!("{}_str", param_ident), Span::call_site());
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(#param_ident_ref) 
                    = #param_ident_raw else { 
                        return false; 
                    };
                    let mut #param_ident_str = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(&#param_ident_ref);
                    let #param_ident = &mut #param_ident_str;
                };
                post_call = quote!{
                    #post_call
                    *#param_ident_ref = native_api_1c::native_api_1c_core::ffi::string_utils::os_string(&#param_ident);
                };
            }
            else {
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Str(#param_ident_ref) 
                    = #param_ident_raw else { 
                        return false; 
                    };
                    let #param_ident = native_api_1c::native_api_1c_core::ffi::string_utils::from_os_string(&#param_ident_ref);
                }
            }
        }
        ParamType::Date => {
            if param.out_param {
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(#param_ident_ref) 
                    = #param_ident_raw else { 
                        return false; 
                    };
                    let mut #param_ident: chrono::DateTime<chrono::FixedOffset> = #param_ident_ref.clone().into();
                };
                post_call = quote!{
                    #post_call
                    *#param_ident_ref = #param_ident.into();
                };
            } else {
                pre_call = quote! {
                    #pre_call
                    let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Date(#param_ident) 
                    = #param_ident_raw else { 
                        return false; 
                    };
                };
            }
        }
        ParamType::Blob => {
            pre_call = quote! {
                #pre_call
                let native_api_1c::native_api_1c_core::ffi::provided_types::ParamValue::Blob(#param_ident) 
                = #param_ident_raw else { 
                    return false; 
                };
            };
        },
        ParamType::SelfType => {
        },
    } 
    (pre_call, post_call)
}
