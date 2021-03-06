use std::convert::{TryFrom, TryInto};

use crate::{Error, Result};
use darling::FromMeta;
use inflector::cases::snakecase::to_snake_case;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::{
    token::Bracket, AttrStyle, Attribute, Field, GenericArgument, Ident, ItemStruct, Path, Token,
    Type, Visibility,
};

/// Helper for deserializing macro props when the default is true
fn enabled() -> bool {
    true
}

// taken from https://github.com/diesel-rs/diesel/blob/master/diesel_derives/src/util.rs#L28
fn is_option_ty(ty: &Type) -> bool {
    option_ty_arg(ty).is_some()
}

fn option_ty_arg(ty: &Type) -> Option<&Type> {
    use syn::PathArguments::AngleBracketed;

    match *ty {
        Type::Path(ref ty) => {
            let last_segment = ty.path.segments.iter().last().unwrap();
            match last_segment.arguments {
                AngleBracketed(ref args) if last_segment.ident == "Option" => {
                    match args.args.iter().last() {
                        Some(&GenericArgument::Type(ref ty)) => Some(ty),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct CrudField {
    pub ident: Ident,
    pub vis: Visibility,
    pub ty: Type,
    pub attrs: Vec<Attribute>,
    pub is_generated: bool,
    pub is_primary_key: bool,
    pub is_sortable: bool,
    pub is_filterable: bool,
    pub is_option: bool,
}

impl CrudField {
    pub fn with_wrapped_option(&self) -> CrudField {
        let mut cloned = self.clone();
        if self.is_option {
            let attr = Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket(Span::call_site()),
                path: syn::parse2(quote! { serde }).unwrap(),
                tokens: quote! { (default, deserialize_with = "::rp1::helper::double_option", skip_serializing_if = "Option::is_none") },
            };
            cloned.attrs.push(attr);
        } else {
            let attr = Attribute {
                pound_token: Token![#](Span::call_site()),
                style: AttrStyle::Outer,
                bracket_token: Bracket(Span::call_site()),
                path: syn::parse2(quote! { serde }).unwrap(),
                tokens: quote! { (skip_serializing_if = "Option::is_none") },
            };
            cloned.attrs.push(attr);
        }

        let ty = &self.ty;
        // parsing should never fail here as #ty was already a previously valid type
        cloned.ty = syn::parse2(quote!(Option<#ty>)).expect("Invalid type formed");

        cloned
    }

    pub fn ensure_option(&self) -> CrudField {
        if self.is_option {
            self.clone()
        } else {
            let mut cloned = self.clone();
            let ty = &self.ty;
            cloned.ty = syn::parse2(quote!(Option<#ty>)).expect("Invalid type formed");
            cloned
        }
    }
}

impl ToTokens for CrudField {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let CrudField {
            attrs,
            ident,
            ty,
            vis,
            ..
        } = self;
        tokens.append_all(quote! {
            #(#attrs)*
            #vis #ident: #ty
        });
    }
}

impl TryFrom<&Field> for CrudField {
    type Error = Error;

    fn try_from(value: &Field) -> Result<Self> {
        let ident = value
            .ident
            .clone()
            .ok_or(Error::UnnamedFieldsNotSupported)?;
        let mut is_generated = false;
        let mut is_primary_key = false;
        let mut is_sortable = true;
        let mut is_filterable = true;
        for attr in value.attrs.iter() {
            if attr.path.is_ident("generated") {
                is_generated = true;
            }

            if attr.path.is_ident("primary_key") {
                is_primary_key = true;
            }

            if attr.path.is_ident("not_sortable") {
                is_sortable = false;
            }

            if attr.path.is_ident("not_filterable") {
                is_filterable = false;
            }
        }

        let attrs = value
            .attrs
            .iter()
            .filter(|a| {
                !a.path.is_ident("generated")
                    && !a.path.is_ident("primary_key")
                    && !a.path.is_ident("not_sortable")
                    && !a.path.is_ident("not_filterable")
            })
            .cloned()
            .collect();

        let is_option = is_option_ty(&value.ty);

        Ok(CrudField {
            ident,
            vis: value.vis.clone(),
            ty: value.ty.clone(),
            attrs,
            is_generated,
            is_primary_key,
            is_sortable,
            is_filterable,
            is_option,
        })
    }
}

/// This struct is a deserialization of all properties that the macro accepts.
///
/// This struct should immediately be converted to [CrudProps].
#[derive(Debug, FromMeta)]
pub struct CrudPropsBuilder {
    #[darling(rename = "database")]
    database_struct: Option<Path>,
    #[darling(default, rename = "schema")]
    schema_path: Option<Path>,
    #[darling(default = "enabled")]
    create: bool,
    #[darling(default = "enabled")]
    read: bool,
    #[darling(default = "enabled")]
    update: bool,
    #[darling(default = "enabled")]
    delete: bool,
    #[darling(default = "enabled")]
    list: bool,
    #[darling(default = "enabled")]
    partials: bool,
    #[darling(default, rename = "module")]
    module_name: Option<Ident>,
    #[darling(default, rename = "table")]
    table_name: Option<Ident>,
    #[darling(default)]
    max_limit: Option<i64>,
    #[darling(default = "enabled")]
    auth: bool,
}

impl CrudPropsBuilder {
    pub fn build(self, mut item: ItemStruct) -> Result<CrudProps> {
        let fields = item
            .fields
            .iter()
            .map(|f| f.try_into())
            .collect::<Result<Vec<_>>>()?;
        let primary_type = fields
            .iter()
            .filter(|f: &&CrudField| f.is_primary_key)
            .map(|f| f.ty.clone())
            .collect::<Vec<_>>();
        if primary_type.len() == 0 {
            return Err(Error::MissingPrimaryKey);
        } else if primary_type.len() > 1 {
            return Err(Error::AggregatePrimaryKeyNotSupported);
        }

        let primary_type = primary_type[0].clone();

        let original_visibility = item.vis.clone();
        item.vis = syn::Visibility::Public(syn::VisPublic {
            pub_token: syn::Token!(pub)([proc_macro2::Span::call_site()]),
        });

        Ok(CrudProps {
            database_struct: self
                .database_struct
                .unwrap_or_else(|| format_ident!("Db").into()),
            schema_path: self
                .schema_path
                .unwrap_or_else(|| syn::parse_str("crate::schema").unwrap()),
            ident: item.ident.clone(),
            new_ident: format_ident!("New{}", &item.ident),
            patch_ident: format_ident!("UpdatePatch{}", &item.ident),
            put_ident: format_ident!("UpdatePut{}", &item.ident),
            filter_ident: format_ident!("{}FilterSpec", &item.ident),
            partial_ident: format_ident!("Partial{}", &item.ident),
            partial_output_ident: format_ident!("PartialOutput{}", &item.ident),
            module_name: self
                .module_name
                .unwrap_or_else(|| format_ident!("{}", to_snake_case(&item.ident.to_string()))),
            create: self.create,
            read: self.read,
            list: self.list,
            update: self.update,
            delete: self.delete,
            partials: self.partials,
            table_name: self
                .table_name
                .unwrap_or_else(|| format_ident!("{}", to_snake_case(&item.ident.to_string()))),
            max_limit: self.max_limit.unwrap_or(100),
            primary_type,
            original_visibility,
            fields,
            item,
            auth: self.auth,
        })
    }
}

/// Properties for generating the current CRUD items.
#[derive(Debug)]
pub struct CrudProps {
    pub(crate) item: ItemStruct,
    pub(crate) database_struct: Path,
    pub(crate) schema_path: Path,
    pub(crate) create: bool,
    pub(crate) read: bool,
    pub(crate) update: bool,
    pub(crate) delete: bool,
    pub(crate) list: bool,
    pub(crate) partials: bool,
    pub(crate) module_name: Ident,
    pub(crate) ident: Ident,
    pub(crate) new_ident: Ident,
    pub(crate) patch_ident: Ident,
    pub(crate) put_ident: Ident,
    pub(crate) filter_ident: Ident,
    pub(crate) partial_ident: Ident,
    pub(crate) partial_output_ident: Ident,
    pub(crate) table_name: Ident,
    pub(crate) primary_type: Type,
    pub(crate) max_limit: i64,
    pub(crate) original_visibility: Visibility,
    pub(crate) fields: Vec<CrudField>,
    pub(crate) auth: bool,
}

impl CrudProps {
    pub(crate) fn sortable_fields(&self) -> impl Iterator<Item = &CrudField> {
        self.fields.iter().filter(|f| f.is_sortable)
    }

    pub(crate) fn filterable_fields(&self) -> impl Iterator<Item = &CrudField> {
        self.fields.iter().filter(|f| f.is_filterable)
    }

    pub(crate) fn patch_fields(&self) -> Vec<CrudField> {
        self.user_supplied_fields()
            .map(|f| f.with_wrapped_option())
            .collect()
    }

    pub(crate) fn non_user_supplied_fields(&self) -> impl Iterator<Item = &CrudField> {
        self.fields
            .iter()
            .filter(|f| f.is_generated || f.is_primary_key)
    }

    pub(crate) fn put_fields(&self) -> impl Iterator<Item = &CrudField> {
        self.fields.iter()
    }

    pub(crate) fn user_supplied_fields(&self) -> impl Iterator<Item = &CrudField> {
        self.fields
            .iter()
            .filter(|f| !f.is_generated && !f.is_primary_key)
    }
}
