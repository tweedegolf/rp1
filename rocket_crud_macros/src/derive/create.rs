use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::props::CrudProps;

pub(crate) fn derive_crud_create(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        new_ident,
        ident,
        table_name,
        schema_path,
        permissions_guard,
        ..
    } = props;

    let validate = if cfg!(feature = "validator") {
        Some(quote::quote! {
            use ::rocket_crud::helper::validation_error_to_response;
            use ::validator::Validate;
            match value.validate() {
                Ok(_) => {},
                Err(e) => return validation_error_to_response(e),
            };
        })
    } else {
        None
    };

    let new_type_tokens = derive_new_type(&props);

    let tokens = quote! {
        #new_type_tokens

        async fn create_fn_help(
            db: #database_struct,
            value: #new_ident
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            use ::rocket_crud::helper::{ok_to_response, db_error_to_response};

            #validate

            db.run(move |conn| {
                diesel::insert_into(#schema_path::#table_name::table)
                    .values(&value)
                    .get_result(conn)
            })
            .await
            .map_or_else(db_error_to_response, ok_to_response)
        }

        #[::rocket::post("/", format = "json", data = "<value>")]
        async fn create_fn_json(
            db: #database_struct,
            _permissions_guard: #permissions_guard,
            value: ::rocket::serde::json::Json<#new_ident>
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            let value = value.into_inner();
            create_fn_help(db, value).await
        }

        #[::rocket::post("/form", data = "<value>")]
        async fn create_fn_form(
            db: #database_struct,
            _permissions_guard: #permissions_guard,
            value: ::rocket::form::Form<#new_ident>
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            let value = value.into_inner();
            create_fn_help(db, value).await
        }
    };

    (
        tokens,
        vec![
            format_ident!("create_fn_form"),
            format_ident!("create_fn_json"),
        ],
    )
}

fn derive_new_type(props: &CrudProps) -> TokenStream {
    let new_ident = &props.new_ident;
    let orig_ident = &props.ident;
    let table_name = props.table_name.to_string();
    let fields = props.user_supplied_fields();

    let derive_validate = if cfg!(feature = "validator") {
        Some(quote::quote! {
            #[derive(::validator::Validate)]
        })
    } else {
        None
    };

    let tokens = quote::quote! {
        #[derive(::diesel::Insertable)]
        #[derive(::diesel::Queryable)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
        #derive_validate
        #[table_name = #table_name]
        struct #new_ident {
            #(#fields),*
        }

        impl ::rocket_crud::CrudInsertableMarker for #orig_ident {}
    };
    tokens
}
