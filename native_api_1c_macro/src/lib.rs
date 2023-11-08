mod derive_addin;
mod extern_functions;

#[proc_macro_derive(AddIn, attributes(add_in_prop, add_in_func, add_in_con, arg, returns))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_addin::derive(input)
}

#[proc_macro]
pub fn extern_functions(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    extern_functions::extern_functions(input)
}
