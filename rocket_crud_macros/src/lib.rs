extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{DeriveInput, Path, parse_macro_input};
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
    let _input = parse_macro_input!(input as DeriveInput);

    todo!()
}

#[proc_macro_derive(CrudInsertable, attributes(primary_key, generated))]
pub fn derive_crud_insertable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);

    let non_generated_fields: Vec<_> = input.fields.iter().filter(|f| !f.attrs.iter().any(
        |a| a.path.is_ident("generated") || a.path.is_ident("primary_key")
    )).collect();
    let ident = quote::format_ident!("New{}", input.ident);

    let tokens = quote::quote! {
        #[derive(Insertable)]
        struct #ident {
            #(#non_generated_fields),*
        }
    };
    println!("{}", tokens);
    tokens.into()
}
