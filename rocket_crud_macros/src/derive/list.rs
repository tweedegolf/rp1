use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Field, Ident};

use crate::{derive::common::derive_auth_param, props::CrudProps};

pub(crate) fn derive_crud_list(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        ident,
        schema_path,
        table_name,
        filter_ident,
        ..
    } = props;

    let auth_param = derive_auth_param(props);
    let auth_filter = if props.auth {
        quote!{
            let filter = <#ident as ::rocket_crud::CheckPermissions>::filter_list(&auth_user);
            let query = filter.apply(query);
        }
    } else {
        quote!{
            let query = Some(query);
        }
    };

    let sortable_field_names = props
        .sortable_fields()
        .map(|f| f.ident.clone())
        .collect::<Vec<_>>();

    let filterable_fields = props.filterable_fields().collect::<Vec<_>>();
    let filter_field_names = filterable_fields
        .iter()
        .map(|f| f.ident.clone())
        .collect::<Vec<_>>();

    let filter_fields: Vec<_> = filterable_fields
        .iter()
        .map(|f| {
            use syn::parse::Parser;

            let ty = &f.ty;
            let ident = &f.ident;

            Field::parse_named
                .parse2(quote! {
                    #ident: Vec<::rocket_crud::FilterOperator<#ty>>
                })
                .unwrap()
        })
        .collect();
    let max_limit = props.max_limit;

    let filter_parse_stmts = filterable_fields
        .iter()
        .map(|f| {
            let field_name = f.ident.clone();
            // parse is not implemented for Option<T>, so we add a special case for it
            if f.is_option {
                quote! {
                    stringify!(#field_name) => {
                        if value == "" {
                            match ::rocket_crud::FilterOperator::from_none(field_operator) {
                                Ok(v) => self.spec.#field_name.push(v),
                                Err(e) => self.errors.push(::rocket_crud::ParseError::from(e).into()),
                            }
                        } else {
                            match ::rocket_crud::FilterOperator::try_parse_option(field_operator, value) {
                                Ok(v) => self.spec.#field_name.push(v),
                                Err(e) => self.errors.push(::rocket_crud::ParseError::from(e).into()),
                            }
                        }
                    }
                }
            } else {
                quote! {
                    stringify!(#field_name) => {
                        match ::rocket_crud::FilterOperator::try_parse(field_operator, value) {
                            Ok(v) => self.spec.#field_name.push(v),
                            Err(e) => self.errors.push(::rocket_crud::ParseError::from(e).into()),
                        }
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    let filter_apply_stmts = filterable_fields
        .iter()
        .map(|f| {
            let ident = &f.ident;

            quote! {
                for op in filter.#ident.iter() {
                    use ::rocket_crud::FilterOperator;
                    use #schema_path::#table_name::columns;
                    query = match op {
                        FilterOperator::Eq(val) => query.filter(columns::#ident.eq(val)),
                        FilterOperator::Ne(val) => query.filter(columns::#ident.ne(val)),
                        FilterOperator::Gt(val) => query.filter(columns::#ident.gt(val)),
                        FilterOperator::Ge(val) => query.filter(columns::#ident.ge(val)),
                        FilterOperator::Lt(val) => query.filter(columns::#ident.lt(val)),
                        FilterOperator::Le(val) => query.filter(columns::#ident.le(val)),
                        FilterOperator::EqAny(val) => query.filter(columns::#ident.eq_any(val)),
                    };
                }
            }
        })
        .collect::<Vec<_>>();

    let tokens = quote! {
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(::rocket::FromFormField, Debug)]
        pub enum SortableFields {
            #(#sortable_field_names),*
        }

        impl ::std::fmt::Display for SortableFields {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", match self {
                    #(SortableFields::#sortable_field_names => stringify!(#sortable_field_names)),*
                })
            }
        }

        use ::rocket::request::{self, Request, FromRequest};

        #[derive(Debug)]
        pub struct #filter_ident {
            #(#filter_fields),*
        }

        impl ::rocket_crud::CrudFilterSpec for #ident {
            type FilterSpecType = #filter_ident;
        }

        impl Default for #filter_ident {
            fn default() -> #filter_ident {
                #filter_ident {
                    #(#filter_field_names: vec![]),*
                }
            }
        }

        #[doc(hidden)]
        pub struct FilterSpecContext<'r> {
            spec: #filter_ident,
            errors: ::rocket::form::Errors<'r>,
        }

        impl<'r> FilterSpecContext<'r> {
            fn push(&mut self, mut field_name: rocket::form::name::NameView<'r>, value: &'r str) {
                use std::convert::{TryFrom, TryInto};

                let field_filtered = match field_name.key() {
                    Some(k) => k,
                    None => {
                        self.errors.push(::rocket::form::error::ErrorKind::Unexpected.into());
                        return;
                    },
                };
                field_name.shift();
                let field_operator = field_name.key().map(|k| k.as_str()).unwrap_or("eq");

                match field_filtered.as_str() {
                    #(#filter_parse_stmts,)*
                    _ => {
                        self.errors.push(::rocket::form::error::ErrorKind::Unexpected.into());
                    },
                };
            }
        }

        #[rocket::async_trait]
        impl<'r> ::rocket::form::FromForm<'r> for #filter_ident {
            type Context = FilterSpecContext<'r>;

            fn init(opts: ::rocket::form::Options) -> Self::Context {
                FilterSpecContext {
                    spec: Default::default(),
                    errors: ::rocket::form::Errors::new(),
                }
            }

            fn push_value(ctxt: &mut Self::Context, field: ::rocket::form::ValueField<'r>) {
                ctxt.push(field.name, field.value);
            }

            async fn push_data(ctxt: &mut Self::Context, field: ::rocket::form::DataField<'r, '_>) {
                use rocket::data::ToByteUnit;

                let limit = 256.kibibytes();
                let bytes = match field.data.open(limit).into_bytes().await {
                    Ok(b) => b,
                    Err(e) => {
                        ctxt.errors.push(e.into());
                        return;
                    },
                };
                if !bytes.is_complete() {
                    ctxt.errors.push(rocket::form::error::ErrorKind::from((None, Some(limit))).into());
                    return;
                }
                let bytes = bytes.into_inner();
                let bytes = rocket::request::local_cache!(field.request, bytes);
                let data = match std::str::from_utf8(bytes) {
                    Ok(d) => d.into(),
                    Err(e) => {
                        ctxt.errors.push(e.into());
                        return;
                    },
                };
                ctxt.push(field.name, data);
            }

            fn finalize(ctxt: Self::Context) -> ::rocket::form::Result<'r, Self> {
                if ctxt.errors.is_empty() {
                    Ok(ctxt.spec)
                } else {
                    Err(ctxt.errors)
                }
            }
        }


        #[::rocket::get("/?<sort>&<offset>&<limit>&<filter>")]
        async fn list_fn(
            db: #database_struct,
            sort: Vec<::rocket_crud::SortSpec<SortableFields>>,
            filter: #filter_ident,
            offset: Option<i64>,
            limit: Option<i64>,
            #auth_param
        ) -> ::rocket_crud::CrudJsonResult<Vec<#ident>>
        {
            let offset = i64::max(0, offset.unwrap_or(0));
            let limit = i64::max(1, i64::min(#max_limit, limit.unwrap_or(#max_limit)));
            let results = db.run(move |conn| {
                use ::rocket_crud::SortDirection;
                use ::diesel::expression::Expression;
                let mut query = #schema_path::#table_name::table.offset(offset).limit(limit).into_boxed();
                for sort_spec in sort {
                    match sort_spec.field {
                        #(SortableFields::#sortable_field_names => {
                            query = if sort_spec.direction == SortDirection::Asc {
                                query.then_order_by(#schema_path::#table_name::columns::#sortable_field_names.asc())
                            } else {
                                query.then_order_by(#schema_path::#table_name::columns::#sortable_field_names.desc())
                            };
                        }),*
                    }
                }
                #(#filter_apply_stmts)*

                #auth_filter

                query.map(|q| q.load(conn))
            })
            .await
            .ok_or_else(|| ::rocket_crud::CrudError::Forbidden)??;

            Ok(::rocket::serde::json::Json(results))
        }
    };

    (tokens, vec![format_ident!("list_fn")])
}
