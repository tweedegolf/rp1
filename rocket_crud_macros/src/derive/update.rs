use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::props::CrudProps;

pub(crate) fn derive_crud_update(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        ident,
        update_ident,
        schema_path,
        table_name,
        primary_type,
        ..
    } = props;

    let update_type = derive_update_type(props);

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

    let tokens = quote! {
        #update_type

        async fn update_fn_help(
            db: #database_struct,
            id: #primary_type,
            value: #update_ident
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            use ::rocket_crud::helper::{ok_to_response, db_error_to_response};

            #validate

            db.run(move |conn| {
                diesel::update(#schema_path::#table_name::table.find(id))
                    .set(&value)
                    .get_result(conn)
            })
            .await
            .map_or_else(db_error_to_response, ok_to_response)
        }

        #[::rocket::patch("/<id>", format = "json", data = "<value>")]
        async fn update_fn_json(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::serde::json::Json<#update_ident>
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            let value = value.into_inner();
            update_fn_help(db, id, value).await
        }

        #[::rocket::patch("/form/<id>", data = "<value>")]
        async fn update_fn_form(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::form::Form<#update_ident>
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            let value = value.into_inner();
            update_fn_help(db, id, value).await
        }
    };
    (
        tokens,
        vec![
            format_ident!("update_fn_json"),
            format_ident!("update_fn_form"),
        ],
    )
}

fn derive_update_type(props: &CrudProps) -> TokenStream {
    let fields = props.updatable_fields();
    let table_name = props.table_name.to_string();

    let CrudProps {
        ident,
        update_ident,
        ..
    } = props;

    let derive_validate = if cfg!(feature = "validator") {
        Some(quote! {
            #[derive(::validator::Validate)]
        })
    } else {
        None
    };

    quote! {
        #[derive(::diesel::Queryable)]
        #[derive(::diesel::AsChangeset)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
        #derive_validate
        #[derive(Default)]
        #[table_name = #table_name]
        struct #update_ident {
            #(#fields),*
        }

        impl ::rocket_crud::CrudUpdatableMarker for #ident {}
    }
}
