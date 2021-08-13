use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{derive::common::derive_auth_param, props::CrudProps};

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

    let auth_param = derive_auth_param(props);
    let auth_pass = if props.auth {
        Some(quote!(auth_user,))
    } else {
        None
    };
    let auth_check = if props.auth {
        Some(quote! {
            let row = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first::<#ident>(conn)
            })
            .await?;
            if !<#ident as ::rp1::CheckPermissions>::allow_update(&row, &value, &auth_user) {
                return Err(::rp1::CrudError::NotFound);
            }
        })
    } else {
        None
    };

    let validate = if cfg!(feature = "validation") {
        Some(quote::quote! {
            use ::validator::Validate;
            value.validate()?;
        })
    } else {
        None
    };

    let tokens = quote! {
        #update_type

        async fn update_fn_help(
            db: #database_struct,
            id: #primary_type,
            value: #update_ident,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            #auth_check

            #validate

            Ok(::rocket::serde::json::Json(db.run(move |conn| {
                diesel::update(#schema_path::#table_name::table.find(id))
                    .set(&value)
                    .get_result(conn)
            })
            .await?))
        }

        #[::rocket::patch("/<id>", format = "json", data = "<value>")]
        async fn update_fn_json(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::serde::json::Json<#update_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            update_fn_help(db, id, value, #auth_pass).await
        }

        #[::rocket::patch("/form/<id>", data = "<value>")]
        async fn update_fn_form(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::form::Form<#update_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            update_fn_help(db, id, value, #auth_pass).await
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

    let derive_validate = if cfg!(feature = "validation") {
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
        pub struct #update_ident {
            #(#fields),*
        }

        impl ::rp1::CrudUpdatable for #ident {
            type UpdateType = #update_ident;
        }
    }
}

pub(crate) fn derive_crud_without_update(props: &CrudProps) -> TokenStream {
    let CrudProps { ident, .. } = props;
    quote! {
        impl ::rp1::CrudUpdatable for #ident {
            type UpdateType = ();
        }
    }
}
