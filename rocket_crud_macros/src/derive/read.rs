use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::props::CrudProps;

pub(crate) fn derive_crud_read(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        ident,
        schema_path,
        table_name,
        primary_type,
        permissions_guard,
        ..
    } = props;

    let tokens = quote! {

        #[::rocket::get("/<id>")]
        async fn read_fn(
            db: #database_struct,
            _permissions_guard: #permissions_guard,
            id: #primary_type,
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            use ::rocket_crud::helper::{ok_to_response, db_error_to_response};

            db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first(conn)
            })
            .await
            .map_or_else(db_error_to_response, ok_to_response)
        }
    };

    (tokens, vec![format_ident!("read_fn")])
}
