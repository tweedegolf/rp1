use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{derive::common::derive_auth_param, props::CrudProps};

pub(crate) fn derive_crud_delete(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        ident,
        database_struct,
        table_name,
        schema_path,
        primary_type,
        ..
    } = props;

    let auth_param = derive_auth_param(props);
    let auth_check = if props.auth {
        Some(quote! {
            let row = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first::<#ident>(conn)
            })
            .await?;
            if !<#ident as ::rp1::CheckPermissions>::allow_delete(&row, &auth_user) {
                return Err(::rp1::CrudError::NotFound);
            }
        })
    } else {
        None
    };

    let tokens = quote! {
        #[::rocket::delete("/<id>")]
        async fn delete_fn(
            db: #database_struct,
            id: #primary_type,
            #auth_param
        ) -> ::rp1::CrudResult<::rocket::serde::json::Value>
        {
            #auth_check

            let deleted = db.run(move |conn| {
                diesel::delete(#schema_path::#table_name::table.find(id)).execute(conn)
            })
            .await?;
            Ok(::rocket::serde::json::json!({
                "deleted": deleted,
            }))
        }
    };

    (tokens, vec![format_ident!("delete_fn")])
}
