use std::{collections::BTreeMap, fmt::Display};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaRef {
    /// The canonical $id for the referenced schema.
    Id(String),
    /// A subset of a schema that describes a concrete value.
    Partial(String, String),
}

impl Serialize for SchemaRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl Display for SchemaRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaRef::Id(id) => {
                write!(f, "id: {id}")
            }
            SchemaRef::Partial(id, partial) => {
                write!(f, "part: {id}@{partial}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Schema {
    Anything,
    Nothing,
    DollarRef(String),
    DynamicRef(String),
    Value(SchemaValue),
    Constant(Constant),
    AllOf(Vec<SchemaRef>),
    ExclusiveOneOf(Vec<SchemaRef>),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Constant(pub serde_json::Value);

impl PartialOrd for Constant {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for Constant {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        panic!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum SchemaValue {
    Boolean,
    Array {
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<SchemaRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_items: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unique_items: Option<bool>,
    },
    Object {
        properties: BTreeMap<String, SchemaRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        additional_properties: Option<SchemaRef>,
    },
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        pattern: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        format: Option<String>,
    },
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclusive_minimum: Option<i64>,
    },
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        minimum: Option<i64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        exclusive_minimum: Option<i64>,
    },
}

pub enum State {
    Canonical(Schema),
    Simplified(Schema),
    Stuck(Schema),
    Todo,
}

impl Schema {
    pub fn simplify(self) -> State {
        match self {
            Schema::Anything => State::Canonical(self),
            Schema::Nothing => State::Canonical(self),
            Schema::DollarRef(_) => State::Todo,
            Schema::DynamicRef(_) => State::Todo,
            Schema::Constant(_) => State::Canonical(self),
            Schema::Value(_) => State::Canonical(self),
            Schema::AllOf(_) => State::Todo,
            Schema::ExclusiveOneOf(vec) => State::Todo,
        }
    }
}
