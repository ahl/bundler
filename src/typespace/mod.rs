mod type_common;
mod type_enum;
mod type_struct;

use syn::parse_quote;
pub use type_common::*;
pub use type_enum::*;
pub use type_struct::*;

use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
    fmt::Display,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

use crate::namespace::Namespace;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeId {
    id: u64,
}

impl Display for TypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NameBuilder {
    Unset,
    Fixed(String),
    Hints(Vec<NameBuilderHint>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum NameBuilderHint {
    Title(String),
    Parent(TypeId, String),
}

// 6/25/2025
// I think I need a builder form e.g. of an enum or struct and then the
// finalized form which probably is basically what typify shows today in its
// public interface.

pub struct Typespace {
    types: BTreeMap<TypeId, Type>,
}
// TODO this impl is intended just for goofing around. I'm sort of wondering if
// these types aren't just "builders"
impl Typespace {
    pub fn render(&self) -> String {
        let types = self.types.iter().map(|(id, typ)| match typ {
            Type::Enum(type_enum) => {
                let TypeEnum {
                    name,
                    description,
                    default,
                    tag_type,
                    variants,
                    deny_unknown_fields,
                    built,
                } = type_enum;
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
                        rust_name,
                        rename,
                        description,
                        details,
                    } = variant;
                    let name = format_ident!("{}", rust_name);
                    let variant_serde = rename.as_ref().map(|n| {
                        quote! {
                            #[serde(rename = #n)]
                        }
                    });
                    let description = description.as_ref().map(|desc| quote! { #[doc = #desc ]});

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
                            let properties = properties
                                .iter()
                                .map(|struct_prop| self.render_struct_property(struct_prop));
                            quote! {
                                {
                                    #( #properties, )*
                                }
                            }
                        }
                    };

                    quote! {
                        #description
                        #variant_serde
                        #name #data
                    }
                });

                let name = built.as_ref().unwrap().name.to_string();
                let name_ident = format_ident!("{name}");

                quote! {
                    // TODO I want to have the original unique id available
                    #description
                    #[derive(::serde::Deserialize, ::serde::Serialize)]
                    #serde
                    pub enum #name_ident {
                        #( #variants, )*
                    }
                }
            }
            Type::Struct(type_struct) => {
                println!("{:#?}", type_struct);
                todo!()
            }
            _ => quote! {},
        });
        let file = parse_quote! {
            #( #types )*
        };
        prettyplease::unparse(&file)
    }

    fn render_ident(&self, id: &TypeId) -> TokenStream {
        let ty = self.types.get(id).unwrap();
        match ty {
            Type::Enum(type_enum) => {
                let name = type_enum.built.as_ref().unwrap().name.to_string();
                let name_ident = format_ident!("{name}");
                name_ident.into_token_stream()
            }
            Type::Struct(_) => {
                quote! { Ref<"???"> }
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
                    ::std::collections::BTreeMap<#key_ident, #value_ident>
                }
            }
            // Type::Set(_) => todo!(),
            // Type::Array(_, _) => todo!(),
            // Type::Tuple(items) => todo!(),
            // Type::Unit => todo!(),
            Type::Boolean => quote! { bool },
            Type::Integer(name) | Type::Float(name) => syn::parse_str::<syn::TypePath>(name)
                .unwrap()
                .to_token_stream(),
            Type::String => quote! { String },
            Type::JsonValue => quote! { ::serde_json::Value },
            _ => quote! { () },
        }
    }

    fn render_struct_property(
        &self,
        StructProperty {
            rust_name,
            json_name,
            state,
            description,
            type_id,
        }: &StructProperty,
    ) -> TokenStream {
        let description = description.as_ref().map(|text| {
            quote! {
                #[doc = #text]
            }
        });

        let mut serde_options = Vec::new();

        match json_name {
            StructPropertySerde::None => {}
            StructPropertySerde::Rename(s) => {
                serde_options.push(quote! {
                    rename = #s
                });
            }
            StructPropertySerde::Flatten => {
                serde_options.push(quote! {
                    #[serde(flatten)]
                });
            }
        };

        let ty_ident = self.render_ident(type_id);

        let ty_ident = match state {
            StructPropertyState::Required => ty_ident,
            StructPropertyState::Optional => {
                serde_options.push(quote! {
                    skip_serializing_if = "::std::option::Option::is_none"
                });
                // TODO 7/10/2025
                // This is interesting and may present an opportunity for
                // customization. Say the type itself is an Option (e.g.
                // because there's a oneOf[null, object]). In this case we've
                // traditionally compressed this down to a single Option, but
                // we could potentially model this as some other type.
                quote! {
                    ::std::option::Option<#ty_ident>
                }
            }
            StructPropertyState::Default(json_value) => todo!(),
        };

        let serde = (!serde_options.is_empty()).then(|| {
            quote! {
                #[serde(
                    #( #serde_options ),*
                )]
            }
        });

        quote! {
            #description
            #serde
            #rust_name: #ty_ident
        }
    }
}

pub struct TypespaceBuilder {
    types: BTreeMap<TypeId, Type>,
    last: TypeId,
}

impl Default for TypespaceBuilder {
    fn default() -> Self {
        Self {
            types: Default::default(),
            last: TypeId { id: 0 },
        }
    }
}

// TODO this impl is intended just for goofing around. I'm sort of wondering if
// these types aren't just "builders"
impl TypespaceBuilder {
    pub fn allocate_id(&mut self) -> TypeId {
        let out = self.last.clone();
        self.last.id += 1;
        out
    }

    pub fn render(&self) -> String {
        let types = self.types.iter().map(|(id, typ)| {
            match typ {
                Type::Enum(type_enum) => {
                    let TypeEnum {
                        name,
                        description,
                        default,
                        tag_type,
                        variants,
                        deny_unknown_fields,
                        ..
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
                            rust_name,
                            rename,
                            description,
                            details,
                        } = variant;
                        let name = format_ident!("{}", rust_name);
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

                    // let xxx_doc_str = id.to_string();
                    // let xxx_doc = quote! { #[doc = #xxx_doc_str] };

                    quote! {
                        // #xxx_doc
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

    fn render_ident(&self, id: &TypeId) -> TokenStream {
        let ty = self.types.get(id).unwrap();
        match ty {
            Type::Enum(_) | Type::Struct(_) => {
                // let ref_str = id.to_string();
                let ref_str = "???";
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

impl TypespaceBuilder {
    pub fn insert(&mut self, id: TypeId, typ: Type) {
        match self.types.entry(id.into()) {
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(typ.into());
            }
            Entry::Occupied(occupied_entry) => {
                let key = occupied_entry.key();
                todo!()
            }
        }
    }

    pub fn finalize(self) -> Result<Typespace, ()> {
        // Basic steps:
        // 1. Construct the parent and child adjacency lists
        // 2. Figure out names for all types that need them
        // 3. Break containment cycles with Box types
        // 4. Propagate trait impls
        // 5. Type-specific finalization

        let Self { mut types, last } = self;

        // TODO 7/2/2025
        // It's all graphs. Think about everything as a graph traversal.

        let id_to_children = types
            .iter()
            .map(|(id, typ)| (id, typ.children()))
            .collect::<BTreeMap<_, _>>();

        // Build forward and backward adjacency lists.
        let mut id_to_parents = BTreeMap::<_, Vec<_>>::new();

        for (id, children) in &id_to_children {
            for child_id in children {
                // Ensure that all referenced types exist
                assert!(types.contains_key(child_id));

                id_to_parents
                    .entry(child_id.clone())
                    .or_default()
                    .push(id.clone());
            }
        }

        // Figure out names for the types that need names.
        let mut work = VecDeque::new();

        for (id, typ) in &types {
            // If it's not a named type, continue
            if !typ.is_named() {
                continue;
            }

            for (child_id, child_sigil) in typ.children_with_context() {
                work.push_back((id.clone(), child_id, child_sigil))
            }
        }

        let mut name_hints = BTreeMap::<_, Vec<NameBuilderHint>>::new();

        while let Some((parent_id, child_id, child_sigil)) = work.pop_front() {
            let child_typ = types.get(&child_id).unwrap();

            if child_typ.is_named() {
                name_hints
                    .entry(child_id)
                    .or_default()
                    .push(NameBuilderHint::Parent(parent_id.clone(), child_sigil));
            } else {
                for (grandchild_id, grandchild_sigil) in child_typ.children_with_context() {
                    work.push_back((
                        parent_id.clone(),
                        grandchild_id,
                        format!("{child_sigil}-=-{grandchild_sigil}"),
                    ))
                }
            }
        }

        println!("{:#?}", name_hints);

        types.iter_mut().for_each(|(id, typ)| {
            if let Some(hints) = name_hints.remove(id) {
                typ.add_name_hints(hints);
            }
        });

        let mut namespace = Namespace::<TypeId>::default();

        for (id, typ) in &mut types {
            match typ {
                Type::Enum(type_enum) => {
                    let name = match &type_enum.name {
                        NameBuilder::Unset => unreachable!(),
                        NameBuilder::Fixed(s) => {
                            let nn = namespace.make_name(id.clone());
                            nn.set_name(s);
                            nn
                        }
                        NameBuilder::Hints(hints) => {
                            let nn = namespace.make_name(id.clone());

                            for hint in hints {
                                match hint {
                                    NameBuilderHint::Title(_) => todo!(),
                                    NameBuilderHint::Parent(id, s) => {
                                        nn.derive_name(id, s);
                                    }
                                }
                            }
                            nn
                        }
                    };
                    type_enum.built = Some(TypeEnumBuilt { name });
                }
                Type::Struct(type_struct) => todo!(),
                _ => {}
            }

            println!("{:#?}", typ);
        }

        let n2 = namespace.finalize().unwrap();

        // TODO 7/1/2025
        // Let's do names first.

        // TODO Make sure that all referenced schemas are present.
        // TODO break cycles
        // TODO resolve names
        // TODO propagate trait impls

        // Break cycles

        Ok(Typespace { types })
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Enum(TypeEnum),
    Struct(TypeStruct),

    Native(String),
    Option(TypeId),

    Box(TypeId),
    Vec(TypeId),
    Map(TypeId, TypeId),
    Set(TypeId),
    Array(TypeId, usize),
    Tuple(Vec<TypeId>),
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

impl Type {
    fn add_name_hints(&mut self, hints: Vec<NameBuilderHint>) {
        if let Some(name) = match self {
            Type::Enum(type_enum) => Some(&mut type_enum.name),
            Type::Struct(type_struct) => Some(&mut type_struct.name),
            _ => None,
        } {
            match name {
                NameBuilder::Unset => *name = NameBuilder::Hints(hints),
                NameBuilder::Fixed(_) => {}
                NameBuilder::Hints(_) => unreachable!(),
            }
        }
    }

    fn get_name(&self) -> Option<&NameBuilder> {
        match self {
            Type::Enum(type_enum) => Some(&type_enum.name),
            Type::Struct(type_struct) => Some(&type_struct.name),
            _ => None,
        }
    }
    fn is_named(&self) -> bool {
        match self {
            Type::Enum(type_enum) => true,
            Type::Struct(type_struct) => true,
            _ => false,
        }
    }

    pub fn children(&self) -> Vec<TypeId> {
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

    fn children_with_context(&self) -> Vec<(TypeId, String)> {
        match self {
            Type::Enum(type_enum) => type_enum.children_with_context(),
            Type::Struct(type_struct) => type_struct.children_with_context(),
            Type::Native(_) => todo!(),
            Type::Option(_) => todo!(),
            Type::Box(_) => todo!(),
            Type::Vec(id) => vec![(id.clone(), "item".to_string())],
            Type::Map(key_id, value_id) => vec![
                (key_id.clone(), "key".to_string()),
                (value_id.clone(), "value".to_string()),
            ],
            Type::Set(_) => todo!(),
            Type::Array(_, _) => todo!(),
            Type::Tuple(items) => todo!(),

            Type::Unit => Vec::new(),
            Type::Boolean => Vec::new(),
            Type::Integer(_) => Vec::new(),
            Type::Float(_) => Vec::new(),
            Type::String => Vec::new(),
            Type::JsonValue => Vec::new(),
        }
    }

    pub fn contained_children_mut(&mut self) -> Vec<&mut TypeId> {
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
