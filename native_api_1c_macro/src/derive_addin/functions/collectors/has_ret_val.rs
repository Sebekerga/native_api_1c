use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_addin::functions::{FuncDesc, ReturnType};

use super::{empty_func_collector_error, FunctionCollector};

pub struct HasReturnValueCollector {
    generated: Result<TokenStream, darling::Error>,
}

impl Default for HasReturnValueCollector {
    fn default() -> Self {
        Self {
            generated: Err(empty_func_collector_error()),
        }
    }
}

impl<'a> FromIterator<(usize, &'a FuncDesc)> for HasReturnValueCollector {
    fn from_iter<T: IntoIterator<Item = (usize, &'a FuncDesc)>>(iter: T) -> Self {
        let mut has_ret_val_body = TokenStream::new();

        for (func_index, func_desc) in iter {
            let has_ret_val = !matches!(func_desc.return_value.ty, ReturnType::None);
            has_ret_val_body.extend(quote! {
                #has_ret_val_body
                if method_num == #func_index { return #has_ret_val };
            });
        }

        let has_ret_val_definition = quote! {
            fn has_ret_val(&self, method_num: usize) -> bool {
                #has_ret_val_body
                false
            }
        };

        Self {
            generated: Ok(has_ret_val_definition),
        }
    }
}

impl FunctionCollector<'_> for HasReturnValueCollector {
    fn release(self) -> Result<TokenStream, darling::Error> {
        self.generated
    }
}
