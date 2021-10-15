use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Field, Ident, ItemStruct};

use crate::{derive::common::derive_auth_param, props::CrudProps};

pub(crate) fn derive_crud_list(props: &CrudProps) -> (TokenStream, Vec<Ident>) {
    let CrudProps {
        database_struct,
        ident,
        schema_path,
        table_name,
        filter_ident,
        partial_ident,
        partial_output_ident,
        ..
    } = props;

    let partials = props.partials;
    let auth_param = derive_auth_param(props);
    let auth_filter = if props.auth {
        quote! {
            let filter = <#ident as ::rp1::CheckPermissions>::filter_list(&auth_user);
            let query = filter.apply(query);
        }
    } else {
        quote! {
            let query = Some(query);
        }
    };

    let output_ident = if partials {
        &props.partial_output_ident
    } else {
        &props.ident
    };
    let partial_struct = if partials {
        Some(derive_partial_result_struct(props))
    } else {
        None
    };
    let rocket_attr = if partials {
        quote!(#[::rocket::get("/?<sort>&<offset>&<limit>&<filter>&<include>&<exclude>")])
    } else {
        quote!(#[::rocket::get("/?<sort>&<offset>&<limit>&<filter>")])
    };
    let partial_params = if partials {
        Some(quote! {
            include: Vec<Fields>,
            exclude: Vec<Fields>,
        })
    } else {
        None
    };
    let select_statements = if partials {
        derive_select_statement(&props)
    } else {
        quote! { #schema_path::#table_name::all_columns }
    };
    let selected_fields_stmt = if partials {
        Some(quote! { let selected = Fields::selected(include, exclude); let selected_out = selected.clone(); })
    } else {
        None
    };
    let partial_result_type = if partials {
        quote!(Vec<#partial_ident>)
    } else {
        quote!(Vec<#output_ident>)
    };
    let partial_result_map = if partials {
        Some(quote! {
            let results = results.into_iter().map(|e| #partial_output_ident::from_partial(e, &selected_out)).collect::<Result<Vec<#output_ident>, ::rp1::CrudError>>()?;
        })
    } else {
        None
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
                    #ident: Vec<::rp1::FilterOperator<#ty>>
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
                            match ::rp1::FilterOperator::from_none(field_operator) {
                                Ok(v) => self.spec.#field_name.push(v),
                                Err(e) => self.errors.push(::rocket::form::Error::custom(e)),
                            }
                        } else {
                            match ::rp1::FilterOperator::try_parse_option(field_operator, value) {
                                Ok(v) => self.spec.#field_name.push(v),
                                Err(e) => self.errors.push(::rocket::form::Error::custom(e)),
                            }
                        }
                    }
                }
            } else {
                quote! {
                    stringify!(#field_name) => {
                        match ::rp1::FilterOperator::try_parse(field_operator, value) {
                            Ok(v) => self.spec.#field_name.push(v),
                            Err(e) => self.errors.push(::rocket::form::Error::custom(e)),
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
                    use ::rp1::FilterOperator;
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
        #partial_struct

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        #[derive(::rocket::FromFormField, Debug)]
        pub enum SortableFields {
            #(#sortable_field_names),*
        }

        use ::rocket::request::{self, Request, FromRequest};

        #[derive(Debug)]
        pub struct #filter_ident {
            #(#filter_fields),*
        }

        impl ::rp1::CrudFilterSpec for #ident {
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
                        self.errors.push(::rocket::form::Error::custom(::rp1::ParseError::UnknownField(field_name.to_string())));
                        return;
                    },
                };
                field_name.shift();
                let field_operator = field_name.key().map(|k| k.as_str()).unwrap_or("eq");

                match field_filtered.as_str() {
                    #(#filter_parse_stmts,)*
                    _ => {
                        self.errors.push(::rocket::form::Error::custom(::rp1::ParseError::UnknownField(field_filtered.as_str().to_owned())));
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


        #rocket_attr
        async fn list_fn(
            db: #database_struct,
            sort: Result<Vec<::rp1::SortSpec<SortableFields>>, ::rocket::form::Errors<'_>>,
            filter: Result<#filter_ident, ::rocket::form::Errors<'_>>,
            offset: Option<i64>,
            limit: Option<i64>,
            #partial_params
            #auth_param
        ) -> ::rp1::CrudJsonResult<Vec<#output_ident>>
        {
            let sort = sort.map_err(|e| ::rp1::CrudError::InvalidSortSpec(e.to_string()))?;
            let filter = filter.map_err(|e| ::rp1::CrudError::InvalidFilterSpec(e.to_string()))?;
            #selected_fields_stmt
            let offset = i64::max(0, offset.unwrap_or(0));
            let limit = i64::max(1, i64::min(#max_limit, limit.unwrap_or(#max_limit)));
            let results: #partial_result_type = db.run(move |conn| {
                use ::rp1::SortDirection;
                use ::diesel::expression::Expression;
                let mut query = #schema_path::#table_name::table
                    .select(#select_statements)
                    .offset(offset)
                    .limit(limit)
                    .into_boxed();
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
            .ok_or_else(|| ::rp1::CrudError::Forbidden)??;

            #partial_result_map
            Ok(::rocket::serde::json::Json(results))
        }
    };

    (tokens, vec![format_ident!("list_fn")])
}

fn derive_partial_result_struct(props: &CrudProps) -> TokenStream {
    let partial_fields = props
        .fields
        .iter()
        .map(|f| f.ensure_option())
        .collect::<Vec<_>>();
    let partial_output_fields = props
        .fields
        .iter()
        .map(|f| f.with_wrapped_option())
        .collect::<Vec<_>>();
    let field_maps = props
        .fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if f.is_option {
                quote! {
                    #ident: if fields.contains(&Fields::#ident) {
                        Some(partial.#ident)
                    } else {
                        None
                    }
                }
            } else {
                quote! {
                    #ident: if fields.contains(&Fields::#ident) {
                        Some(partial.#ident.ok_or(::rp1::CrudError::DbValueError)?)
                    } else {
                        None
                    }
                }
            }
        }).collect::<Vec<_>>();
    let partial_ident = &props.partial_ident;
    let partial_output_ident = &props.partial_output_ident;
    let ItemStruct {
        attrs, generics, ..
    } = &props.item;

    quote! {
        #(#attrs)*
        #[derive(::diesel::Queryable, ::serde::Serialize, ::validator::Validate)]
        pub struct #partial_ident #generics {
            #(#partial_fields),*
        }

        #(#attrs)*
        #[derive(::serde::Serialize, ::validator::Validate)]
        pub struct #partial_output_ident #generics {
            #(#partial_output_fields),*
        }

        impl #partial_output_ident {
            pub fn from_partial(partial: #partial_ident, fields: &Vec<Fields>) -> ::rp1::CrudResult<#partial_output_ident> {
                Ok(#partial_output_ident {
                    #(#field_maps),*
                })
            }
        }
    }
}

fn derive_select_statement(props: &CrudProps) -> TokenStream {
    let fields = props.fields.iter().map(|f| {
        let name = &f.ident;
        if f.is_option {
            quote! {
                if selected.contains(&Fields::#name) {
                    diesel::dsl::sql(stringify!(#name))
                } else {
                    diesel::dsl::sql("null")
                }
            }
        } else {
            quote! {
                if selected.contains(&Fields::#name) {
                    diesel::dsl::sql(stringify!(#name))
                } else {
                    diesel::dsl::sql("null")
                }
            }
        }
    });

    quote! { (#(#fields,)*) }
}
