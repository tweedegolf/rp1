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
        ..
    } = props;

    let tokens = quote! {

        #[::rocket::get("/<id>")]
        async fn read_fn(
            db: #database_struct,
            auth_user: <#ident as ::rocket_crud::access_control::CheckPermissions>::AuthUser,
            id: #primary_type,
        ) -> ::rocket_crud::RocketCrudResponse<#ident>
        {
            use ::rocket_crud::helper::{ok_to_response, db_error_to_response};
            use ::rocket_crud::access_control::CheckPermissions;

            // let auth_user = todo!();

            let db_result  = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first::<#ident>(conn)
            })
            .await;


            match db_result {
                Err(e) => db_error_to_response(e),
                Ok(user) => {

                    if <#ident as CheckPermissions>::allow_read(&user, &auth_user) {
                        ok_to_response(user)
                    } else {
                        panic!()
                    }

                }
            }
        }
    };

    (tokens, vec![format_ident!("read_fn")])
}
