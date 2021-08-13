extern crate proc_macro;

mod crud_impl;
mod derive;
mod error;
mod props;

pub(crate) use error::*;

use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs};

#[doc = include_str!("../docs/crud.md")]
#[proc_macro_attribute]
pub fn crud(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    match crud_impl::crud_impl(args, item.into()) {
        Ok(res) => res.into(),
        Err(e) => e.into_compile_error().into(),
    }
}
