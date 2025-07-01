mod type_common;
mod type_enum;
mod type_struct;

use syn::parse_quote;
pub use type_common::*;
pub use type_enum::*;
pub use type_struct::*;

use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet},
    fmt::Display,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

// 6/25/2025
// I think I need a builder form e.g. of an enum or struct and then the
// finalized form which probably is basically what typify shows today in its
// public interface.

pub trait TypeId: Ord + Display + std::fmt::Debug + Clone {}

pub struct TypespaceBuilder<Id> {
    types: BTreeMap<Id, Type<Id>>,
}

impl<Id> Default for TypespaceBuilder<Id> {
    fn default() -> Self {
        Self {
            types: Default::default(),
        }
    }
}

// TODO this impl is intended just for goofing around. I'm sort of wondering if
// these types aren't just "builders"
impl<Id> TypespaceBuilder<Id>
where
    Id: TypeId,
{
    pub fn render(&self) -> String {
        let types = self.types.iter().map(|(id, typ)| {
            match typ {
                Type::Enum(type_enum) => {
                    let TypeEnum {
                        description,
                        default,
                        tag_type,
                        variants,
                        deny_unknown_fields,
                    } = type_enum;
                    // let name = format_ident!("{}", name);
                    let description = description.as_ref().map(|desc| quote! { #[doc = #desc ]});
                    let serde = match tag_type {
                        EnumTagType::External => TokenStream::new(),
                        EnumTagType::Internal { tag } => quote! {
                            #[serde(tag = #tag)]
                        },
                        EnumTagType::Adjacent { tag, content } => quote! {
                            #[serde(tag = #tag, content = #content)]
                        },
                        EnumTagType::Untagged => quote! {
                            #[serde(untagged)]
                        },
                    };

                    let variants = variants.iter().map(|variant| {
                        let EnumVariant {
                            variant_name,
                            description,
                            details,
                        } = variant;
                        let name = format_ident!("{}", variant_name);
                        let description =
                            description.as_ref().map(|desc| quote! { #[doc = #desc ]});

                        let data = match details {
                            VariantDetails::Simple => TokenStream::new(),
                            VariantDetails::Item(item) => {
                                let item_ident = self.render_ident(item);
                                quote! {
                                    (#item_ident)
                                }
                            }
                            VariantDetails::Tuple(items) => todo!(),
                            VariantDetails::Struct(properties) => {
                                let properties = properties.iter().map(
                                    |StructProperty {
                                         rust_name,
                                         json_name,
                                         state,
                                         description,
                                         type_id,
                                     }| {
                                        let description = description
                                            .as_ref()
                                            .map(|desc| quote! { #[doc = #desc ]});

                                        let serde = match json_name {
                                            StructPropertySerde::None => TokenStream::new(),
                                            StructPropertySerde::Rename(s) => quote! {
                                                #[serde(rename = #s)]
                                            },
                                            StructPropertySerde::Flatten => quote! {
                                                #[serde(flatten)]
                                            },
                                        };

                                        let xxx_ident = self.render_ident(type_id);

                                        quote! {
                                            #description
                                            #serde
                                            #rust_name: #xxx_ident
                                        }
                                    },
                                );
                                quote! {
                                    {
                                        #( #properties, )*
                                    }
                                }
                            }
                        };

                        quote! {
                            #description
                            #name #data
                        }
                    });

                    let xxx_doc_str = id.to_string();
                    let xxx_doc = quote! { #[doc = #xxx_doc_str] };

                    quote! {
                        #xxx_doc
                        #description
                        #serde
                        pub enum Unknown {
                            #( #variants, )*
                        }
                    }
                }
                Type::Struct(type_struct) => {
                    println!("{:#?}", type_struct);
                    todo!()
                }
                _ => quote! {},
            }
        });
        let file = parse_quote! {
            #( #types )*
        };
        prettyplease::unparse(&file)
    }

    fn render_ident(&self, id: &Id) -> TokenStream {
        let ty = self.types.get(id).unwrap();
        match ty {
            Type::Enum(_) | Type::Struct(_) => {
                let ref_str = id.to_string();
                quote! { Ref<#ref_str> }
            }
            // Type::Native(_) => todo!(),
            // Type::Option(_) => todo!(),
            // Type::Box(_) => todo!(),
            Type::Vec(inner_id) => {
                let inner_ident = self.render_ident(inner_id);
                quote! {
                    ::std::vec::Vec<#inner_ident>
                }
            }
            Type::Map(key_id, value_id) => {
                let key_ident = self.render_ident(key_id);
                let value_ident = self.render_ident(value_id);
                quote! {
                    ::std::btreemap::BTreeMap<#key_ident, #value_ident>
                }
            }
            // Type::Set(_) => todo!(),
            // Type::Array(_, _) => todo!(),
            // Type::Tuple(items) => todo!(),
            // Type::Unit => todo!(),
            Type::Boolean => quote! { boolean },
            Type::Integer(name) | Type::Float(name) => syn::parse_str::<syn::TypePath>(name)
                .unwrap()
                .to_token_stream(),
            Type::String => quote! { String },
            Type::JsonValue => quote! { ::serde_json::Value },
            _ => quote! { () },
        }
    }
}

pub struct Typespace<Id> {
    types: BTreeMap<Id, Type<Id>>,
}

impl<Id> TypespaceBuilder<Id>
where
    Id: TypeId,
{
    pub fn insert(&mut self, id: Id, typ: Type<Id>)
    where
        Id: Ord,
    {
        match self.types.entry(id) {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(typ);
            }
            Entry::Occupied(occupied_entry) => {
                let key = occupied_entry.key();
                todo!()
            }
        }
    }

    pub fn finalize(self) -> Result<Typespace<Id>, ()> {
        let Self { types } = self;

        // Build forward and backward adjacency lists.
        let children = types
            .iter()
            .map(|(id, typ)| (id.clone(), typ.children()))
            .collect::<BTreeMap<_, _>>();
        let mut parents = BTreeMap::<Id, Vec<Id>>::new();

        for (id, ch) in &children {
            for child in ch {
                let xxx = parents.entry(child.clone()).or_default();
                xxx.push(id.clone());
            }
        }

        // TODO Make sure that all referenced schemas are present.
        // TODO break cycles
        // TODO resolve names
        // TODO propagate trait impls

        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum Type<Id> {
    Enum(TypeEnum<Id>),
    Struct(TypeStruct<Id>),

    Native(String),

    Option(Id),

    Box(Id),
    Vec(Id),
    Map(Id, Id),
    Set(Id),
    Array(Id, usize),
    Tuple(Vec<Id>),
    Unit,
    Boolean,
    /// Integers
    Integer(String),
    /// Floating point numbers; not Eq, Ord, or Hash
    Float(String),
    /// Strings... which we handle a little specially.
    String,
    /// serde_json::Value which we also handle specially.
    JsonValue,
}

impl<Id> Type<Id>
where
    Id: Clone,
{
    pub fn children(&self) -> Vec<Id> {
        match self {
            Type::Enum(type_enum) => type_enum.children(),
            Type::Struct(type_struct) => type_struct.children(),
            Type::Boolean => Vec::new(),
            Type::String => Vec::new(),
            Type::Native(_) => Vec::new(),

            Type::Option(id)
            | Type::Box(id)
            | Type::Vec(id)
            | Type::Set(id)
            | Type::Array(id, _) => vec![id.clone()],

            Type::Map(key_id, value_id) => vec![key_id.clone(), value_id.clone()],
            Type::Tuple(items) => items.clone(),

            Type::Unit => Vec::new(),
            Type::Integer(_) => Vec::new(),
            Type::Float(_) => Vec::new(),
            Type::JsonValue => Vec::new(),
        }
    }

    pub fn contained_children_mut(&mut self) -> Vec<&mut Id> {
        match self {
            Type::Enum(TypeEnum { variants, .. }) => todo!(),
            Type::Struct(TypeStruct { properties, .. }) => todo!(),
            Type::Native(_) => todo!(),
            Type::Option(_) => todo!(),
            Type::Box(_) => todo!(),
            Type::Vec(_) => todo!(),
            Type::Map(_, _) => todo!(),
            Type::Set(_) => todo!(),
            Type::Array(_, _) => todo!(),
            Type::Tuple(items) => todo!(),
            Type::Unit => todo!(),
            Type::Boolean => todo!(),
            Type::Integer(_) => todo!(),
            Type::Float(_) => todo!(),
            Type::String => todo!(),
            Type::JsonValue => todo!(),
        }
    }
}
