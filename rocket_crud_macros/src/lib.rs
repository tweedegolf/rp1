extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn crud(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as syn::ItemStruct);
    let ident = input.ident.clone();

    let visibility = input.vis.clone();
    input.vis = syn::Visibility::Public(syn::VisPublic {
        pub_token: syn::Token!(pub)([proc_macro2::Span::call_site()]),
    });

    let crudmod = quote::format_ident!("fkfkffkfk");

    let tokens = quote::quote! {

        mod #crudmod {
            use crate::Db;
            use crate::schema::*;
            use diesel::prelude::*;

            #[derive(::rocket_crud::CrudInsertable)]
            #[derive(::rocket_crud::CrudCreate)]
            #[derive(::rocket_crud::CrudRead)]
            #[derive(::rocket_crud::CrudDelete)]
            #[derive(::rocket_crud::CrudUpdate)]
            #input

            impl #ident {
                pub fn get_routes() -> Vec<::rocket::Route> {
                    rocket::routes![create_fn, read_fn, delete_fn, update_fn]
                }
            }
        }

        #visibility use self::#crudmod::#ident;

    };
    tokens.into()
}

#[proc_macro_derive(CrudInsertable, attributes(primary_key, generated, table_name))]
pub fn derive_crud_insertable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let orig_ident = input.ident;

    let table_name = input.attrs.iter().find(|a| a.path.is_ident("table_name"));
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
        #[derive(::diesel::Queryable)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
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

/// Get the `foo` out of `#[table_name = "foo"]`
fn get_table_name(attrs: &[syn::Attribute]) -> Option<syn::Ident> {
    let table_name_attr = attrs.iter().find(|a| a.path.is_ident("table_name"));
    if let syn::Meta::NameValue(mnv) = table_name_attr.unwrap().parse_meta().unwrap() {
        if let syn::Lit::Str(lit_str) = mnv.lit {
            Some(quote::format_ident!("{}", lit_str.value()))
        } else {
            None
        }
    } else {
        None
    }
}

//    let db_ident = input.attrs.iter().find(|a| a.path.is_ident("crud_db")).map(|db| {
//        let tokens: TokenStream = db.tokens.clone().into();
//        let input: DatabasePath = syn::parse(tokens).expect("No valid database path");
//        // let input = parse_macro_input!(tokens as DatabasePath);
//        input.value
//    }).expect("Database connection name is required");

#[proc_macro_derive(CrudCreate, attributes(crud_db, table_name))]
pub fn derive_crud_create(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let orig_ident = input.ident;
    let new_ident = quote::format_ident!("New{}", &orig_ident);

    let db_ident = quote::format_ident!("Db");

    let table_name = get_table_name(&input.attrs).unwrap();

    let tokens = quote! {
        #[::rocket::post("/", format = "json", data = "<value>")]
        async fn create_fn(db: #db_ident, value: ::rocket::serde::json::Json<#new_ident>) -> ::rocket::serde::json::Json<#orig_ident> {
            let value = value.into_inner();

            let result = db.run(move |conn| {
                diesel::insert_into(crate::schema::#table_name::table)
                    .values(&value)
                    .get_result(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };
    tokens.into()
}

#[proc_macro_derive(CrudRead, attributes(crud_db, table_name))]
pub fn derive_crud_read(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let orig_ident = input.ident;

    let db_ident = quote::format_ident!("Db");

    let table_name = get_table_name(&input.attrs).unwrap();

    let tokens = quote! {
        #[::rocket::get("/<id>")]
        async fn read_fn(db: #db_ident, id: i32) -> ::rocket::serde::json::Json<#orig_ident> {

            let result = db.run(move |conn| {
                crate::schema::#table_name::table
                    .find(id)
                    .first(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };
    tokens.into()
}

#[proc_macro_derive(CrudUpdate, attributes(crud_db, table_name))]
pub fn derive_crud_update(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);
    let orig_ident = input.ident;

    let db_ident = quote::format_ident!("Db");

    let table_name = get_table_name(&input.attrs).unwrap();

    let primary_key_col_name = quote::format_ident!("id");

    let non_generated_fields: Vec<_> = input
        .fields
        .iter()
        .filter(|f| {
            !f.attrs
                .iter()
                .any(|a| a.path.is_ident("generated") || a.path.is_ident("primary_key"))
        })
        .map(|field| {
            let mut field = field.clone();

            let t = field.ty;
            let new_type = syn::parse(quote!(Option<#t>).into()).unwrap();

            field.ty = new_type;

            field
        })
        .collect();

    let ident = quote::format_ident!("Update{}", &orig_ident);

    let table_name_attribute = input.attrs.iter().find(|a| a.path.is_ident("table_name"));
    let update_struct = quote::quote! {
        #[derive(::diesel::Queryable)]
        #[derive(::diesel::AsChangeset)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
        #[derive(Default)]
        #table_name_attribute
        struct #ident {
            #(#non_generated_fields),*
        }
    };

    let tokens = quote! {
        #update_struct

        #[::rocket::patch("/<id>", format = "json", data = "<value>")]
        async fn update_fn(db: #db_ident, id: i32, value: ::rocket::serde::json::Json<#ident>) -> ::rocket::serde::json::Json<#orig_ident> {
            let value = value.into_inner();

            let result = db.run(move |conn| {
                diesel::update(crate::schema::#table_name::table.find(id))
                    .set(&value)
                    .get_result(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };
    tokens.into()
}

#[proc_macro_derive(CrudDelete, attributes(crud_db, table_name))]
pub fn derive_crud_delete(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::ItemStruct);

    let db_ident = quote::format_ident!("Db");

    let table_name = get_table_name(&input.attrs).unwrap();

    let tokens = quote! {
        #[::rocket::delete("/<id>")]
        async fn delete_fn(db: #db_ident, id: i32) -> ::rocket::serde::json::Json<usize> {

            let result = db.run(move |conn| {
                diesel::delete(crate::schema::#table_name::table.find(id)).execute(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };
    tokens.into()
}
