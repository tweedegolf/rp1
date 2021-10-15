use proc_macro2::TokenStream;
use quote::quote;

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
