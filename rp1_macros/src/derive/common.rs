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
    let fields = &props.fields;
    quote! {
        #[allow(non_camel_case_types)]
        #[derive(::rocket::FromFormField, Debug)]
        pub enum Fields {
            #(#fields),*
        }

        impl ::std::fmt::Display for Fields {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", match self {
                    #(Fields::#fields => stringify!(#fields)),*
                })
            }
        }

        impl ::std::string::ToString for Fields {
            fn to_string(&self) -> String {
                match self {
                    #(Fields::#fields => stringify!(#fields).to_owned()),*
                }
            }
        }
    }
}

pub(crate) fn derive_partial_result_struct(props: &CrudProps) -> TokenStream {
    let fields = &props.fields.iter().map(|f| f.with_wrapped_option()).collect::<Vec<_>>();
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
