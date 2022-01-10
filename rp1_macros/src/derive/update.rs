use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{derive::common::derive_auth_param, props::CrudProps};

pub(crate) fn derive_crud_update(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        ident,
        patch_ident,
        put_ident,
        schema_path,
        table_name,
        primary_type,
        ..
    } = props;

    let update_types = derive_update_types(props);

    let auth_param = derive_auth_param(props);
    let auth_pass = if props.auth {
        Some(quote!(auth_user,))
    } else {
        None
    };
    let auth_put_check = if props.auth {
        Some(quote! {
            if !<#ident as ::rp1::CheckPermissions>::allow_update(&row, &value, &auth_user) {
                return Err(::rp1::CrudError::NotFound);
            }
        })
    } else {
        None
    };
    let auth_patch_check = if props.auth {
        Some(quote! {
            let row = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first::<#ident>(conn)
            })
            .await?;
            let put_value = #put_ident::create(&row, &value);
            if !<#ident as ::rp1::CheckPermissions>::allow_update(&row, &put_value, &auth_user) {
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
        #update_types

        async fn update_put_fn_help(
            db: #database_struct,
            id: #primary_type,
            value: #put_ident,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let row = db.run(move |conn| {
                #schema_path::#table_name::table
                    .find(id)
                    .first::<#ident>(conn)
            })
            .await?;

            #auth_put_check

            value.validate_update(&row)?;

            #validate

            Ok(::rocket::serde::json::Json(db.run(move |conn| {
                diesel::update(#schema_path::#table_name::table.find(id))
                    .set(&value)
                    .get_result(conn)
            })
            .await?))
        }

        async fn update_patch_fn_help(
            db: #database_struct,
            id: #primary_type,
            value: #patch_ident,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            #auth_patch_check

            #validate

            Ok(::rocket::serde::json::Json(db.run(move |conn| {
                diesel::update(#schema_path::#table_name::table.find(id))
                    .set(&value)
                    .get_result(conn)
            })
            .await?))
        }

        #[::rocket::patch("/<id>", format = "json", data = "<value>")]
        async fn update_patch_fn_json(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::serde::json::Json<#patch_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            update_patch_fn_help(db, id, value, #auth_pass).await
        }

        #[::rocket::patch("/<id>", format = "form", data = "<value>")]
        async fn update_patch_fn_form(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::form::Form<#patch_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            update_patch_fn_help(db, id, value, #auth_pass).await
        }

        #[::rocket::put("/<id>", format = "json", data = "<value>")]
        async fn update_put_fn_json(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::serde::json::Json<#put_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            update_put_fn_help(db, id, value, #auth_pass).await
        }

        #[::rocket::put("/<id>", format = "form", data = "<value>")]
        async fn update_put_fn_form(
            db: #database_struct,
            id: #primary_type,
            value: ::rocket::serde::json::Json<#put_ident>,
            #auth_param
        ) -> ::rp1::CrudJsonResult<#ident>
        {
            let value = value.into_inner();
            update_put_fn_help(db, id, value, #auth_pass).await
        }
    };
    (
        tokens,
        vec![
            format_ident!("update_patch_fn_json"),
            format_ident!("update_patch_fn_form"),
            format_ident!("update_put_fn_json"),
            format_ident!("update_put_fn_form"),
        ],
    )
}

fn derive_update_types(props: &CrudProps) -> TokenStream {
    let patch_fields = props.patch_fields();
    let put_fields = props.put_fields();
    let table_name = props.table_name.to_string();
    let patch_field_names = patch_fields
        .iter()
        .map(|f| f.ident.clone())
        .collect::<Vec<_>>();
    let non_patch_fields = props
        .non_user_supplied_fields()
        .map(|f| f.ident.clone())
        .collect::<Vec<_>>();

    // Only forward serde attributes for now
    let attrs = props.item.attrs
        .iter()
        .filter(|attr| attr.path.is_ident("serde"))
        .cloned()
        .collect::<Vec<_>>();

    let CrudProps {
        ident,
        patch_ident,
        put_ident,
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
        #(#attrs)*
        #[table_name = #table_name]
        pub struct #patch_ident {
            #(#patch_fields),*
        }

        #[derive(::diesel::Queryable)]
        #[derive(::diesel::AsChangeset)]
        #[derive(::rocket::form::FromForm)]
        #[derive(::serde::Deserialize)]
        #derive_validate
        #(#attrs)*
        #[table_name = #table_name]
        pub struct #put_ident {
            #(#put_fields),*
        }

        impl #put_ident {
            pub fn create(base: &#ident, patch: &#patch_ident) -> #put_ident {
                #(
                    let #patch_field_names = if let Some(ref v) = patch.#patch_field_names {
                        v.clone()
                    } else {
                        base.#patch_field_names.clone()
                    };
                )*

                #put_ident {
                    #(#patch_field_names),*,
                    #(#non_patch_fields: base.#non_patch_fields.clone()),*,
                }
            }

            pub fn validate_update(&self, base: &#ident) -> ::rp1::CrudResult<()> {
                #(
                    if self.#non_patch_fields != base.#non_patch_fields {
                        return Err(::rp1::CrudError::UnchangeableField(stringify!(#non_patch_fields).to_owned()));
                    }
                )*

                Ok(())
            }

            pub fn into_patch(self, base: &#ident) -> #patch_ident {
                #patch_ident {
                    #(
                        #patch_field_names:
                            if self.#patch_field_names == base.#patch_field_names {
                                None
                            } else {
                                Some(self.#patch_field_names)
                            }
                    ),*
                }
            }
        }

        impl ::rp1::CrudUpdatable for #ident {
            type PatchType = #patch_ident;
            type PutType = #put_ident;
        }
    }
}

pub(crate) fn derive_crud_without_update(props: &CrudProps) -> TokenStream {
    let CrudProps { ident, .. } = props;
    quote! {
        impl ::rp1::CrudUpdatable for #ident {
            type PatchType = ();
            type PutType = ();
        }
    }
}
