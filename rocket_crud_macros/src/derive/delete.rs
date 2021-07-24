use proc_macro2::TokenStream;
use syn::Ident;
use quote::{format_ident, quote};

use crate::props::CrudProps;

pub(crate) fn derive_crud_delete(props: &CrudProps) -> (TokenStream, Vec<Ident>) {

    let CrudProps {
        database_struct,
        table_name,
        schema_path,
        primary_type,
        permissions_guard,
        ..
    } = props;

    let tokens = quote! {
        #[::rocket::delete("/<id>")]
        async fn delete_fn(
            db: #database_struct,
            _permissions_guard: #permissions_guard,
            id: #primary_type,
        ) -> ::rocket_crud::RocketCrudResponse<usize>
        {
            use ::rocket_crud::helper::{ok_to_response, db_error_to_response};

            db.run(move |conn| {
                diesel::delete(#schema_path::#table_name::table.find(id)).execute(conn)
            })
            .await
            .map_or_else(db_error_to_response, ok_to_response)
        }
    };

    (tokens, vec![format_ident!("delete_fn")])
}
