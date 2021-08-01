use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{derive::common::derive_auth_param, props::CrudProps};

pub(crate) fn derive_crud_read(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        ident,
        schema_path,
        table_name,
        primary_type,
        ..
    } = props;

    let auth_param = derive_auth_param(props);
    let auth_check = if props.auth {
        quote!{
            if <#ident as ::rocket_crud::CheckPermissions>::allow_read(&row, &auth_user) {
                row
            } else {
                return Err(::rocket_crud::CrudError::NotFound);
            }
        }
    } else {
        quote!(row)
    };

    let tokens = quote! {

        #[::rocket::get("/<id>")]
        async fn read_fn(
            db: #database_struct,
            id: #primary_type,
            #auth_param
        ) -> ::rocket_crud::CrudJsonResult<#ident>
        {
            let row = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first::<#ident>(conn)
            })
            .await?;
            let row = #auth_check;
            Ok(::rocket::serde::json::Json(row))
        }
    };

    (tokens, vec![format_ident!("read_fn")])
}
