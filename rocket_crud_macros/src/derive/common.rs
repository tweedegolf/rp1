use proc_macro2::TokenStream;
use quote::quote;

use crate::props::CrudProps;

pub(crate) fn derive_auth_param(props: &CrudProps) -> Option<TokenStream> {
    let ident = &props.ident;
    if props.auth {
        Some(quote!{
            auth_user: <#ident as ::rocket_crud::access_control::CheckPermissions>::AuthUser,
        })
    } else {
        None
    }
}
