use std::collections::BTreeMap;

use serde::Serialize;

use crate::{bootstrap, Resolved};

#[derive(Clone, Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaRef {
    Id(String),
    Partial(String, String),
}

impl SchemaRef {
    pub fn partial(&self, part: &str) -> Self {
        let SchemaRef::Id(id) = self else { panic!() };
        SchemaRef::Partial(id.clone(), part.to_string())
    }

    pub fn append(&self, fragment: &str) -> Self {
        let SchemaRef::Id(id) = self else { panic!() };
        SchemaRef::Id(format!("{id}/{fragment}"))
    }

    pub fn id(&self) -> String {
        let SchemaRef::Id(id) = self else { panic!() };
        id.clone()
    }
}

/// A Schemalet is a self-contained, bounded schema that references any
/// subordinate schemas rather than including them inline.
#[derive(Serialize)]
pub struct Schemalet {
    #[serde(skip_serializing_if = "SchemaletMetadata::is_default")]
    pub metadata: SchemaletMetadata,
    pub details: SchemaletDetails,
}

#[derive(Default, Serialize)]
pub struct SchemaletMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<serde_json::Value>,
}

#[derive(Serialize)]
pub enum SchemaletDetails {
    Anything,
    Nothing,
    ExclusiveOneOf(Vec<SchemaRef>),
    AllOf(Vec<SchemaRef>),
    AnyOf(Vec<SchemaRef>),
    RawRef(String),
    RawDynamicRef(String),
    Constant(serde_json::Value),
    Value(SchemaletValue),
}

#[derive(Serialize)]
pub enum SchemaletValue {
    Boolean,
    SchemaValueObject(SchemaletValueObject),
}

#[derive(Serialize)]
pub struct SchemaletValueObject {
    pub properties: BTreeMap<String, SchemaRef>,
    pub additional_properties: Option<SchemaRef>,
}

impl Schemalet {
    pub fn new(details: SchemaletDetails, metadata: SchemaletMetadata) -> Self {
        Self { metadata, details }
    }

    pub fn from_details(details: SchemaletDetails) -> Self {
        Self {
            metadata: Default::default(),
            details,
        }
    }
}

impl SchemaletMetadata {
    fn is_default(&self) -> bool {
        let Self {
            title,
            description,
            examples,
        } = self;

        title.is_none() && description.is_none() && examples.is_empty()
    }
}

pub fn to_schemalets(resolved: &Resolved<'_>) -> anyhow::Result<Vec<(SchemaRef, Schemalet)>> {
    match resolved.schema {
        "https://json-schema.org/draft/2020-12/schema" => bootstrap::to_schemalets(resolved),
        _ => todo!(),
    }
}
