use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

use crate::props::CrudProps;

pub(crate) fn derive_auth_param(props: &CrudProps) -> Option<TokenStream> {
    let ident = &props.ident;
    if props.auth {
        Some(quote! {
            auth_user: <#ident as ::rp1::access_control::CheckPermissions>::AuthUser,
        })
    } else {
        None
    }
}

pub(crate) fn derive_field_list(props: &CrudProps) -> TokenStream {
    let &CrudProps {
        schema_path,
        table_name,
        ..
    } = &props;
    let fields = &props
        .fields
        .iter()
        .map(|f| f.clone().ident)
        .collect::<Vec<_>>();

    quote! {
        #[allow(non_camel_case_types)]
        #[derive(::rocket::FromFormField, Debug, PartialEq, Eq)]
        pub enum Fields {
            #(#fields),*
        }

        impl Fields {
            pub fn all() -> Vec<Fields> {
                vec![
                    #(Fields::#fields),*
                ]
            }

            pub fn selected(include: Vec<Fields>, exclude: Vec<Fields>) -> Vec<Fields> {
                let include = if include.is_empty() {
                    Fields::all()
                } else {
                    include
                };
                include.into_iter().filter(|f| !exclude.contains(f)).collect()
            }
        }

        impl ::std::fmt::Display for Fields {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", match self {
                    #(Fields::#fields => stringify!(#fields)),*
                })
            }
        }
    }
}

pub(crate) fn derive_partial_result_struct(props: &CrudProps) -> TokenStream {
    let fields = &props
        .fields
        .iter()
        .map(|f| f.ensure_option())
        .collect::<Vec<_>>();
    let partial_ident = &props.partial_ident;
    let ItemStruct {
        attrs, generics, ..
    } = &props.item;

    quote! {
        #(#attrs)*
        #[derive(serde::Serialize, diesel::Queryable, validator::Validate)]
        pub struct #partial_ident #generics {
            #(#fields),*
        }
    }
}
