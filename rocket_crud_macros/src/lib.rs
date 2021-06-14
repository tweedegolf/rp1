extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::parse_macro_input;

// Helper function, some values need to be true by default
fn enabled() -> bool {
    true
}
#[derive(Debug, darling::FromMeta)]
struct CrudPropsBuilder {
    #[darling(rename = "database")]
    database_struct: syn::Path,
    #[darling(default, rename = "schema")]
    schema_path: Option<syn::Path>,
    #[darling(default = "enabled")]
    create: bool,
    #[darling(default = "enabled")]
    read: bool,
    #[darling(default = "enabled")]
    update: bool,
    #[darling(default = "enabled")]
    delete: bool,
    #[darling(default = "enabled")]
    list: bool,
    #[darling(default, rename = "module")]
    module_name: Option<syn::Ident>,
    #[darling(skip)] // TODO: allow specifying the identifier for the new struct
    new_ident: Option<syn::Ident>,
    #[darling(skip)] // TODO: allow specifying the identifier for the update struct
    update_ident: Option<syn::Ident>,
    #[darling(default)]
    table_name: Option<syn::Ident>,
}

#[derive(Debug)]
struct CrudProps {
    database_struct: syn::Path,
    schema_path: syn::Path,
    create: bool,
    read: bool,
    update: bool,
    delete: bool,
    list: bool,
    module_name: syn::Ident,
    ident: syn::Ident,
    new_ident: syn::Ident,
    update_ident: syn::Ident,
    table_name: syn::Ident,
}

/// Attributes that we must drop from the final output.
/// Also we treat fields with these attributes differently in some cases
fn must_skip_attribute(a: &syn::Attribute) -> bool {
    !a.path.is_ident("generated") && !a.path.is_ident("primary_key")
}

#[proc_macro_attribute]
pub fn crud(args: TokenStream, item: TokenStream) -> TokenStream {
    use darling::FromMeta;
    use inflector::cases::snakecase::to_snake_case;

    let mut input = parse_macro_input!(item as syn::ItemStruct);
    let attr_args = parse_macro_input!(args as syn::AttributeArgs);

    let props = match CrudPropsBuilder::from_list(&attr_args) {
        Ok(v) => CrudProps {
            database_struct: v.database_struct,
            schema_path: v.schema_path.unwrap_or_else(|| syn::parse_str("crate::schema").unwrap()),
            ident: input.ident.clone(),
            new_ident: v.new_ident.unwrap_or_else(|| quote::format_ident!("New{}", &input.ident)),
            update_ident: v.update_ident.unwrap_or_else(|| quote::format_ident!("Update{}", &input.ident)),
            module_name: v.module_name.unwrap_or_else(|| syn::Ident::new(&to_snake_case(&input.ident.to_string()), Span::call_site())),
            create: v.create,
            read: v.read,
            update: v.update,
            delete: v.delete,
            list: v.list,
            table_name: v.table_name.unwrap_or_else(|| format_ident!("{}", to_snake_case(&input.ident.to_string()))),
        },
        Err(e) => return e.write_errors().into(),
    };

    let visibility = input.vis.clone();
    input.vis = syn::Visibility::Public(syn::VisPublic {
        pub_token: syn::Token!(pub)([proc_macro2::Span::call_site()]),
    });

    let mut tokens = vec![];
    let mut funcs = vec![];
    if props.create {
        tokens.push(derive_crud_insertable(&input, &props));

        let (toks, mut func) = derive_crud_create(&props);
        tokens.push(toks);
        funcs.append(&mut func);
    }

    if props.read {
        let (toks, mut func) = derive_crud_read(&props);
        tokens.push(toks);
        funcs.append(&mut func);
    }

    if props.update {
        tokens.push(derive_crud_updatable(&input, &props));

        let (toks, mut func) = derive_crud_update(&props);
        tokens.push(toks);
        funcs.append(&mut func);
    }

    if props.delete {
        let (toks, mut func) = derive_crud_delete(&props);
        tokens.push(toks);
        funcs.append(&mut func);
    }

    if props.list {
        let (toks, mut func) = derive_crud_list(&props);
        tokens.push(toks);
        funcs.append(&mut func);
    }

    let module_name = props.module_name;
    let ident = props.ident;
    let schema_path = props.schema_path;
    let table_name = props.table_name;

    for f in input.fields.iter_mut() {
        f.attrs.retain(|a| !must_skip_attribute(a));
    }

    let tokens = quote::quote! {

        mod #module_name {
            use super::*;
            use diesel::prelude::*;
            use #schema_path::#table_name;

            #input

            #(#tokens)*

            impl #ident {
                pub fn get_routes() -> Vec<::rocket::Route> {
                    rocket::routes![#(#funcs),*]
                }
            }
        }

        #visibility use self::#module_name::#ident;

    };
    tokens.into()
}

fn derive_crud_insertable(input: &syn::ItemStruct, props: &CrudProps) -> proc_macro2::TokenStream {
    let non_generated_fields: Vec<_> = input
        .fields
        .iter()
        .filter(|f| !f.attrs.iter().any(must_skip_attribute))
        .collect();

    let new_ident = &props.new_ident;
    let orig_ident = &props.ident;
    let table_name = props.table_name.to_string();

    let tokens = quote::quote! {
        #[derive(::diesel::Insertable)]
        #[derive(::diesel::Queryable)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
        #[table_name = #table_name]
        struct #new_ident {
            #(#non_generated_fields),*
        }

        impl ::rocket_crud::CrudInsertableMarker for #orig_ident {}
    };
    tokens
}

fn derive_crud_create(props: &CrudProps) -> (proc_macro2::TokenStream, Vec<syn::Ident>) {
    let CrudProps { database_struct, new_ident, ident, table_name, schema_path, .. } = props;

    let tokens = quote! {
        #[::rocket::post("/", format = "json", data = "<value>")]
        async fn create_fn(db: #database_struct, value: ::rocket::serde::json::Json<#new_ident>) -> ::rocket::serde::json::Json<#ident> {
            let value = value.into_inner();

            let result = db.run(move |conn| {
                diesel::insert_into(#schema_path::#table_name::table)
                    .values(&value)
                    .get_result(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };
    (tokens, vec![format_ident!("create_fn")])
}

fn derive_crud_read(props: &CrudProps) -> (proc_macro2::TokenStream, Vec<syn::Ident>) {
    let CrudProps { database_struct, ident, schema_path, table_name, .. } = props;

    let tokens = quote! {
        #[::rocket::get("/<id>")]
        async fn read_fn(db: #database_struct, id: i32) -> (::rocket::http::Status, ::rocket_crud::Either<::rocket::serde::json::Json<#ident>, ::rocket::serde::json::Value>) {
            use ::rocket::http::Status;
            use ::diesel::result::Error;
            use ::rocket_crud::Either;
            use ::rocket::serde::json::{Json, json};


            let result = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first(conn)
            })
            .await;
            match result {
                Ok(res) => (Status::Ok, Either::Left(Json(res))),
                Err(Error::NotFound) => (Status::NotFound, Either::Right(json!({
                    "error": 404,
                }))),
                Err(e) => (Status::InternalServerError, Either::Right(json!({
                    "error": 500,
                }))),
            }
        }
    };

    (tokens, vec![format_ident!("read_fn")])
}

fn derive_crud_updatable(input: &syn::ItemStruct, props: &CrudProps) -> proc_macro2::TokenStream {
    let non_generated_fields: Vec<_> = input
        .fields
        .iter()
        .filter(|f| !f.attrs.iter().any(must_skip_attribute))
        .map(|field| {
            let mut field = field.clone();

            let t = field.ty;
            let new_type = syn::parse(quote!(Option<#t>).into()).unwrap();

            field.ty = new_type;

            field
        })
        .collect();

    let table_name = props.table_name.to_string();

    let CrudProps { ident, update_ident, .. } = props;

    quote::quote! {
        #[derive(::diesel::Queryable)]
        #[derive(::diesel::AsChangeset)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
        #[derive(Default)]
        #[table_name = #table_name]
        struct #update_ident {
            #(#non_generated_fields),*
        }

        impl ::rocket_crud::CrudUpdatableMarker for #ident {}
    }
}

fn derive_crud_update(props: &CrudProps) -> (proc_macro2::TokenStream, Vec<syn::Ident>) {
    let CrudProps { database_struct, ident, update_ident, schema_path, table_name, .. } = props;

    let tokens = quote! {
        #[::rocket::patch("/<id>", format = "json", data = "<value>")]
        async fn update_fn(db: #database_struct, id: i32, value: ::rocket::serde::json::Json<#update_ident>) -> ::rocket::serde::json::Json<#ident> {
            let value = value.into_inner();

            let result = db.run(move |conn| {
                diesel::update(#schema_path::#table_name::table.find(id))
                    .set(&value)
                    .get_result(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };
    (tokens, vec![format_ident!("update_fn")])
}

fn derive_crud_delete(props: &CrudProps) -> (proc_macro2::TokenStream, Vec<syn::Ident>) {
    let CrudProps { database_struct, table_name, schema_path, .. } = props;

    let tokens = quote! {
        #[::rocket::delete("/<id>")]
        async fn delete_fn(db: #database_struct, id: i32) -> ::rocket::serde::json::Json<usize> {

            let result = db.run(move |conn| {
                diesel::delete(#schema_path::#table_name::table.find(id)).execute(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };

    (tokens, vec![format_ident!("delete_fn")])
}

fn derive_crud_list(props: &CrudProps) -> (proc_macro2::TokenStream, Vec<syn::Ident>) {
    let CrudProps { database_struct, ident, schema_path, table_name, .. } = props;
    let tokens = quote! {
        #[::rocket::get("/")]
        async fn list_fn(db: #database_struct) -> ::rocket::serde::json::Json<Vec<#ident>> {
            let result = db.run(move |conn| {
                #schema_path::#table_name::table.load(conn)
            })
            .await
            .unwrap();

            ::rocket::serde::json::Json(result)
        }
    };

    (tokens, vec![format_ident!("list_fn")])
}
