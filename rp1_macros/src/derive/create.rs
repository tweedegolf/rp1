use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{derive::common::derive_auth_param, props::CrudProps};

pub(crate) fn derive_crud_create(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        new_ident,
        ident,
        table_name,
        schema_path,
        ..
    } = props;

    let validate = if cfg!(feature = "validation") {
        Some(quote::quote! {
            use ::validator::Validate;
            value.validate()?;
        })
    } else {
        None
    };

    let auth_param = derive_auth_param(props);
    let auth_pass = if props.auth {
        Some(quote!(auth_user,))
    } else {
        None
    };
    let auth_check = if props.auth {
        Some(quote! {
            if !<#ident as ::rp1::CheckPermissions>::allow_create(&value, &auth_user) {
                return Err(::rp1::CrudError::Forbidden);
            }
        })
    } else {
        None
    };

    let new_type_tokens = derive_new_type(&props);

    let tokens = quote! {
        #new_type_tokens

        async fn create_fn_help(
            db: #database_struct,
            value: #new_ident,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            #auth_check

            #validate

            Ok(::rocket::serde::json::Json(db.run(move |conn| {
                diesel::insert_into(#schema_path::#table_name::table)
                    .values(&value)
                    .get_result(conn)
            }).await?))
        }

        #[::rocket::post("/", format = "json", data = "<value>")]
        async fn create_fn_json(
            db: #database_struct,
            value: ::rocket::serde::json::Json<#new_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            create_fn_help(db, value, #auth_pass).await
        }

        #[::rocket::post("/", format = "form", data = "<value>")]
        async fn create_fn_form(
            db: #database_struct,
            value: ::rocket::form::Form<#new_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            create_fn_help(db, value, #auth_pass).await
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
    let CrudProps {
        ident, new_ident, ..
    } = props;
    let table_name = props.table_name.to_string();
    let fields = props.user_supplied_fields();

    // Only forward serde attributes for now
    let attrs = props
        .item
        .attrs
        .iter()
        .filter(|attr| attr.path.is_ident("serde"))
        .cloned()
        .collect::<Vec<_>>();

    let derive_validate = if cfg!(feature = "validation") {
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
        #(#attrs)*
        #[table_name = #table_name]
        pub struct #new_ident {
            #(#fields),*
        }

        impl ::rp1::CrudInsertable for #ident {
            type InsertType = #new_ident;
        }
    };
    tokens
}

pub(crate) fn derive_crud_without_create(props: &CrudProps) -> TokenStream {
    let CrudProps { ident, .. } = props;
    quote! {
        impl ::rp1::CrudInsertable for #ident {
            type InsertType = ();
        }
    }
}
