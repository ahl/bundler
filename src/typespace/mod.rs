mod type_common;
mod type_enum;
mod type_struct;

use syn::parse_quote;
pub use type_common::*;
pub use type_enum::*;
pub use type_struct::*;

use std::{collections::BTreeMap, fmt::Display};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

// 6/25/2025
// I think I need a builder form e.g. of an enum or struct and then the
// finalized form which probably is basically what typify shows today in its
// public interface.

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
    Id: Ord + Display + std::fmt::Debug,
{
    pub fn render(&self) -> String {
        let types = self.types.values().map(|typ| {
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

                    quote! {
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
            // Type::Map(_, _) => todo!(),
            // Type::Set(_) => todo!(),
            // Type::Array(_, _) => todo!(),
            // Type::Tuple(items) => todo!(),
            // Type::Unit => todo!(),
            Type::Boolean => quote! { boolean },
            // Type::Integer(_) => todo!(),
            // Type::Float(_) => todo!(),
            Type::String => quote! { String },
            // Type::JsonValue => todo!(),
            _ => quote! { () },
        }
    }
}

pub struct Typespace<Id> {
    types: BTreeMap<Id, Type<Id>>,
}

impl<Id> TypespaceBuilder<Id> {
    pub fn insert(&mut self, id: Id, typ: Type<Id>)
    where
        Id: Ord,
    {
        // TODO do some validation of types, e.g. that variant names are
        // unique.
        self.types.insert(id, typ);
    }

    pub fn finalize(self) -> Result<Typespace<Id>, ()> {
        let Self { types } = self;

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

    // TODO not sure about our plan on options...
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
}
