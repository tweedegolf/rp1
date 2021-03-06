use proc_macro2::TokenStream;
use quote::quote;
use syn::{AttributeArgs, ItemStruct};

use crate::props::{CrudProps, CrudPropsBuilder};

pub fn crud_impl(args: AttributeArgs, item: TokenStream) -> crate::Result {
    use darling::FromMeta;

    let input: ItemStruct = syn::parse2(item)?;
    let props = CrudPropsBuilder::from_list(&args)?.build(input)?;

    let mut tokens = vec![];
    let mut routes = vec![];

    tokens.push(crate::derive::common::derive_field_list(&props));

    if props.create {
        let (toks, mut func) = crate::derive::create::derive_crud_create(&props);
        tokens.push(toks);
        routes.append(&mut func);
    } else {
        tokens.push(crate::derive::create::derive_crud_without_create(&props));
    }

    if props.read {
        let (toks, mut func) = crate::derive::read::derive_crud_read(&props);
        tokens.push(toks);
        routes.append(&mut func);
    }

    if props.update {
        let (toks, mut func) = crate::derive::update::derive_crud_update(&props);
        tokens.push(toks);
        routes.append(&mut func);
    } else {
        tokens.push(crate::derive::update::derive_crud_without_update(&props));
    }

    if props.delete {
        let (toks, mut func) = crate::derive::delete::derive_crud_delete(&props);
        tokens.push(toks);
        routes.append(&mut func);
    }

    if props.list {
        let (toks, mut func) = crate::derive::list::derive_crud_list(&props);
        tokens.push(toks);
        routes.append(&mut func);
    }

    let CrudProps {
        module_name,
        ident,
        schema_path,
        table_name,
        original_visibility,
        fields,
        ..
    } = props;

    let ItemStruct {
        attrs, generics, ..
    } = props.item;

    let tokens = quote! {

        mod #module_name {
            use super::*;
            use diesel::prelude::*;
            use #schema_path::#table_name;

            #(#attrs)*
            #[derive(serde::Serialize, diesel::Queryable, validator::Validate)]
            pub struct #ident #generics {
                #(#fields),*
            }

            #(#tokens)*

            impl #ident {
                pub fn get_routes() -> Vec<::rocket::Route> {
                    rocket::routes![#(#routes),*]
                }
            }

            impl ::rp1::CrudStruct for #ident {
                type TableType = #schema_path::#table_name::table;
            }
        }

        #original_visibility use self::#module_name::#ident;
    };
    Ok(tokens)
}
