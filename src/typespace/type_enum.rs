use crate::typespace::{JsonValue, StructProperty};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeEnum<Id> {
    // TODO need fancy name logic
    pub name: String,
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub tag_type: EnumTagType,
    pub variants: Vec<EnumVariant<Id>>,
    pub deny_unknown_fields: bool,
    //     pub bespoke_impls: BTreeSet<TypeEntryEnumImpl>,
    //     pub schema: SchemaWrapper,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct EnumVariant<Id> {
    pub json_name: String,
    pub variant_name: String,
    pub description: Option<String>,
    pub details: VariantDetails<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum VariantDetails<Id> {
    Simple,
    Item(Id),
    Tuple(Vec<Id>),
    Struct(Vec<StructProperty<Id>>),
}
