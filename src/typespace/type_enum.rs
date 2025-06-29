use crate::typespace::{JsonValue, StructProperty};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeEnum<Id> {
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub tag_type: EnumTagType,
    pub variants: Vec<EnumVariant<Id>>,
    pub deny_unknown_fields: bool,
}
impl<Id> TypeEnum<Id>
where
    Id: Clone,
{
    pub(crate) fn children(&self) -> Vec<Id> {
        self.variants
            .iter()
            .flat_map(|variant| variant.children())
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
    pub variant_name: String,
    // TODO need a name for serialization?
    // pub json_name: String,
    pub description: Option<String>,
    pub details: VariantDetails<Id>,
}
impl<Id> EnumVariant<Id>
where
    Id: Clone,
{
    fn children(&self) -> Vec<Id> {
        match &self.details {
            VariantDetails::Simple => Vec::new(),
            VariantDetails::Item(id) => vec![id.clone()],
            VariantDetails::Tuple(items) => items.clone(),
            VariantDetails::Struct(items) => {
                items.iter().map(|prop| prop.type_id.clone()).collect()
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariantDetails<Id> {
    Simple,
    Item(Id),
    Tuple(Vec<Id>),
    Struct(Vec<StructProperty<Id>>),
}
