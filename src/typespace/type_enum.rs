use syn::Variant;

use crate::{
    namespace::Name,
    typespace::{InternalId, JsonValue, NameBuilder, StructProperty},
};

#[derive(Debug, Clone)]
pub struct TypeEnum<Id> {
    pub name: NameBuilder<InternalId<Id>>,
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub tag_type: EnumTagType,
    pub variants: Vec<EnumVariant<Id>>,
    pub deny_unknown_fields: bool,

    pub built: Option<TypeEnumBuilt<Id>>,
}

#[derive(Debug, Clone)]
pub(crate) struct TypeEnumBuilt<Id> {
    pub name: Name<InternalId<Id>>,
}

impl<Id> TypeEnum<Id>
where
    Id: Clone,
{
    pub fn new(
        name: NameBuilder<Id>,
        description: Option<String>,
        default: Option<JsonValue>,
        tag_type: EnumTagType,
        variants: Vec<EnumVariant<Id>>,
        deny_unknown_fields: bool,
    ) -> Self {
        let name = name.into();
        Self {
            name,
            description,
            default,
            tag_type,
            variants,
            deny_unknown_fields,
            built: None,
        }
    }

    pub(crate) fn children(&self) -> Vec<InternalId<Id>> {
        self.variants
            .iter()
            .flat_map(|variant| variant.children())
            .collect()
    }

    pub(crate) fn children_with_context(&self) -> Vec<(InternalId<Id>, String)> {
        self.variants
            .iter()
            .flat_map(|variant| variant.children_with_context())
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EnumTagType {
    /// serde external tagging (serde's default)
    External,
    /// serde internal tagging
    Internal { tag: String },
    /// serde adjacent tagging
    Adjacent { tag: String, content: String },
    /// serde untagged
    Untagged,
}

// TODO 6/24/2025
// Do I want the variants to have tagging? I mean we could support the variant
// tagging for untagged if we wanted. Also how would we support more custom
// enums ala typify#811
// 6/28/2025
// Answer: No. Recall that the untagged variant markers need to be at the end
// of the type which makes it kind of a pain in the neck.

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumVariant<Id> {
    pub rust_name: String,
    pub rename: Option<String>,
    // TODO need a name for serialization?
    // pub json_name: String,
    pub description: Option<String>,
    pub details: VariantDetails<Id>,
}
impl<Id> EnumVariant<Id>
where
    Id: Clone,
{
    fn children(&self) -> Vec<InternalId<Id>> {
        match &self.details {
            VariantDetails::Simple => Vec::new(),
            VariantDetails::Item(id) => vec![id.clone()],
            VariantDetails::Tuple(items) => items.clone(),
            VariantDetails::Struct(items) => {
                items.iter().map(|prop| prop.type_id.clone()).collect()
            }
        }
    }

    fn children_with_context(&self) -> Vec<(InternalId<Id>, String)> {
        match &self.details {
            VariantDetails::Simple => Vec::new(),
            VariantDetails::Item(id) => vec![(id.clone(), self.rust_name.clone())],
            VariantDetails::Tuple(items) => items
                .iter()
                .enumerate()
                .map(|(ii, id)| (id.clone(), format!("{}.{}", &self.rust_name, ii)))
                .collect(),
            VariantDetails::Struct(items) => items
                .iter()
                .map(|prop| {
                    (
                        prop.type_id.clone(),
                        format!("{}.{}", &self.rust_name, &prop.rust_name),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariantDetails<Id> {
    Simple,
    Item(InternalId<Id>),
    Tuple(Vec<InternalId<Id>>),
    Struct(Vec<StructProperty<Id>>),
}

impl<Id> VariantDetails<Id> {
    pub fn new_simple() -> Self {
        VariantDetails::Simple
    }

    pub fn new_item(id: Id) -> Self {
        VariantDetails::Item(id.into())
    }
}
