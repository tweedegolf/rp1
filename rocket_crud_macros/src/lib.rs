extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn mono_test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let task_fn = syn::parse_macro_input!(item as syn::ItemFn);

    let args = task_fn.sig.inputs.clone();

    let name = task_fn.sig.ident.clone();
    let name_str = name.to_string();
    let body = task_fn.block.clone();

    let visibility = &task_fn.vis;

    let result = quote! {
        #[test]
        #visibility fn #name(#args) -> () {
            compiles_to_ir(#name_str, #body);

        }
    };
    result.into()
}

#[proc_macro_derive(CrudCreate, attributes(auto_error))]
pub fn derive_crud_create(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    todo!()
}
