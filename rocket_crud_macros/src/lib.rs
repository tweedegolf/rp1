extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input};


#[proc_macro_derive(CrudInsertable, attributes(primary_key, generated, table_name))]
pub fn derive_crud_insertable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let orig_ident = input.ident;

    let table_name = input
        .attrs
        .iter()
        .find(|a| a.path.is_ident("table_name"));
    let non_generated_fields: Vec<_> = input
        .fields
        .iter()
        .filter(|f| {
            !f.attrs
                .iter()
                .any(|a| a.path.is_ident("generated") || a.path.is_ident("primary_key"))
        })
        .collect();
    let ident = quote::format_ident!("New{}", &orig_ident);

    let tokens = quote::quote! {
        #[derive(::diesel::Insertable)]
        #table_name
        struct #ident {
            #(#non_generated_fields),*
        }

        impl ::rocket_crud::CrudInsertableMarker for #orig_ident {}
    };
    tokens.into()
}

#[derive(Debug)]
struct DatabasePath {
    eq_token: syn::Token![=],
    value: syn::Path,
}

impl syn::parse::Parse for DatabasePath {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(DatabasePath {
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[proc_macro_derive(CrudCreate, attributes(crud_db))]
pub fn derive_crud_create(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let orig_ident = input.ident;
    let new_ident = quote::format_ident!("New{}", &orig_ident);
    let db_ident = input.attrs.iter().find(|a| a.path.is_ident("crud_db")).map(|db| {
        let tokens: TokenStream = db.tokens.clone().into();
        let input: DatabasePath = syn::parse(tokens).expect("No valid database path");
        // let input = parse_macro_input!(tokens as DatabasePath);
        input.value
    }).expect("Database connection name is required");

    let tokens = quote! {
        #[::rocket::post("/")]
        async fn create_fn(db: #db_ident, insertable: #new_ident) -> ::rocket::serde::json::Json<#orig_ident> {
            todo!()
        }
    };
    println!("{}", tokens);
    tokens.into()
}
