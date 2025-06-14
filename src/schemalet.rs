use std::{collections::BTreeMap, fmt::Display};

use serde::Serialize;
use url::Url;

use crate::{bootstrap, Resolved};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

impl Display for SchemaRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaRef::Id(id) => f.write_str(id),
            SchemaRef::Partial(id, part) => {
                f.write_str(id)?;
                f.write_str(" @@ ")?;
                f.write_str(part)
            }
        }
    }
}

impl Serialize for SchemaRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = self.to_string();
        s.serialize(serializer)
    }
}

/// A Schemalet is a self-contained, bounded schema that references any
/// subordinate schemas rather than including them inline.
#[derive(Serialize, Debug, Clone)]
pub struct Schemalet {
    #[serde(flatten)]
    pub metadata: SchemaletMetadata,
    pub details: SchemaletDetails,
}

#[derive(Default, Serialize, Debug, Clone)]
pub struct SchemaletMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<serde_json::Value>,
}

#[derive(Serialize, Debug, Clone)]
pub enum SchemaletDetails {
    // Native
    Anything,
    Nothing,

    // Subschemas
    OneOf(Vec<SchemaRef>),
    AnyOf(Vec<SchemaRef>),
    AllOf(Vec<SchemaRef>),
    Not(SchemaRef),
    IfThen(SchemaRef, SchemaRef),
    IfThenElse(SchemaRef, SchemaRef, SchemaRef),
    RawRef(String),
    RawDynamicRef(String),
    Constant(serde_json::Value),
    Value(SchemaletValue),

    // Synthetic
    ExclusiveOneOf(Vec<SchemaRef>),
    ResolvedRef(SchemaRef),
    ResolvedDynamicRef(SchemaRef),
}

#[derive(Debug, Clone, Serialize)]
pub enum SchemaletValue {
    Boolean,
    Array {
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<SchemaRef>,
        #[serde(skip_serializing_if = "Option::is_none")]
        min_items: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unique_items: Option<bool>,
    },
    Object(SchemaletValueObject),
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

// TODO don't worry about naming for now, but this will probably be the most
// relevant output type
#[derive(Serialize, Debug, Clone)]
pub struct CanonicalSchemalet {
    #[serde(flatten)]
    pub metadata: SchemaletMetadata,
    pub details: CanonicalSchemaletDetails,
}

#[derive(Serialize, Debug, Clone)]
pub enum CanonicalSchemaletDetails {
    Anything,
    Nothing,
    Constant(serde_json::Value),
    // TODO 6/14/2025 not 100% sure where this is going to be used, but it
    // might be interesting
    Reference(SchemaRef),
    ExclusiveOneOf(Vec<SchemaRef>),
    // TODO 6/14/2025 This is wrong. I know I'm going to need constraints (both
    // affirmative and negative), and we need to handle constant values more
    // similarly, etc. Also "Anything", but we'll roll with that.
    Value(SchemaletValue),
}

pub enum State {
    Stuck(Schemalet),
    Simplified(Schemalet, Vec<(SchemaRef, Schemalet)>),
    Canonical(CanonicalSchemalet),
}

#[derive(Debug, Clone, Serialize)]
pub struct SchemaletValueObject {
    pub properties: BTreeMap<String, SchemaRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
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

    pub fn simplify(self, done: &BTreeMap<SchemaRef, CanonicalSchemalet>) -> State {
        let Self { metadata, details } = self;
        match details {
            SchemaletDetails::OneOf(schema_refs) => todo!(),
            SchemaletDetails::Not(schema_ref) => todo!(),
            SchemaletDetails::IfThen(schema_ref, schema_ref1) => todo!(),
            SchemaletDetails::IfThenElse(schema_ref, schema_ref1, schema_ref2) => todo!(),
            SchemaletDetails::RawRef(_) => todo!(),
            SchemaletDetails::RawDynamicRef(_) => todo!(),

            SchemaletDetails::AllOf(schema_refs) => {
                if let Some(subschemas) = schema_refs
                    .iter()
                    .map(|schema_ref| resolve(done, schema_ref))
                    .collect::<Option<Vec<_>>>()
                {
                    println!("{}", serde_json::to_string_pretty(&subschemas).unwrap());
                    merge_all(subschemas, done)
                } else {
                    State::Stuck(Schemalet {
                        metadata,
                        details: SchemaletDetails::AllOf(schema_refs),
                    })
                }
            }

            SchemaletDetails::AnyOf(schema_refs) => State::Stuck(Schemalet {
                metadata,
                details: SchemaletDetails::AnyOf(schema_refs),
            }),

            SchemaletDetails::Anything => State::Canonical(CanonicalSchemalet {
                metadata,
                details: CanonicalSchemaletDetails::Anything,
            }),
            SchemaletDetails::Nothing => State::Canonical(CanonicalSchemalet {
                metadata,
                details: CanonicalSchemaletDetails::Nothing,
            }),
            SchemaletDetails::Constant(value) => State::Canonical(CanonicalSchemalet {
                metadata,
                details: CanonicalSchemaletDetails::Constant(value),
            }),

            SchemaletDetails::ResolvedDynamicRef(reference)
            | SchemaletDetails::ResolvedRef(reference) => State::Canonical(CanonicalSchemalet {
                metadata,
                details: CanonicalSchemaletDetails::Reference(reference),
            }),

            SchemaletDetails::Value(value) => State::Canonical(CanonicalSchemalet {
                metadata,
                details: CanonicalSchemaletDetails::Value(value),
            }),

            SchemaletDetails::ExclusiveOneOf(schema_refs) => {
                if schema_refs
                    .iter()
                    .all(|schema_ref| done.contains_key(schema_ref))
                {
                    State::Canonical(CanonicalSchemalet {
                        metadata,
                        details: CanonicalSchemaletDetails::ExclusiveOneOf(schema_refs),
                    })
                } else {
                    State::Stuck(Schemalet {
                        metadata,
                        details: SchemaletDetails::ExclusiveOneOf(schema_refs),
                    })
                }
            }
        }
    }
}

// TODO 6/14/2025 not fully sure why we need the done map...
fn merge_all(
    subschemas: Vec<(SchemaRef, &CanonicalSchemalet)>,
    done: &BTreeMap<SchemaRef, CanonicalSchemalet>,
) -> State {
    // Separate out xors (disjunctions) from other schemas.
    let mut xors = Vec::new();
    let mut rest = Vec::new();
    for (schema_ref, schema) in subschemas {
        match &schema.details {
            CanonicalSchemaletDetails::ExclusiveOneOf(ss) => xors.push(ss),
            _ => rest.push((schema_ref, schema)),
        }
    }

    if let Some(subschemas) = xors.pop() {
        let mut merge_groups = subschemas
            .iter()
            .map(|schema_ref| (schema_ref, vec![schema_ref]))
            .collect::<Vec<_>>();

        for subschemas in xors {
            merge_groups = merge_groups
                .into_iter()
                .flat_map(|(representative, group)| {
                    subschemas
                        .iter()
                        .filter(|schema_ref| {
                            !trivially_incompatible(done, representative, schema_ref)
                        })
                        .map(move |schema_ref| {
                            let mut new_group = group.clone();
                            new_group.push(schema_ref);
                            (representative, new_group)
                        })
                })
                .collect::<Vec<_>>()
        }

        let mut merge_groups = merge_groups
            .into_iter()
            .map(|(_, group)| group)
            .collect::<Vec<_>>();

        for group in &mut merge_groups {
            for (schema_ref, _) in &rest {
                group.push(schema_ref);
            }
        }

        println!(
            "groups {}",
            serde_json::to_string_pretty(&merge_groups).unwrap()
        );
    }

    todo!()
}

fn trivially_incompatible(
    done: &BTreeMap<SchemaRef, CanonicalSchemalet>,
    a: &SchemaRef,
    b: &SchemaRef,
) -> bool {
    let (_, aaa) = resolve(done, a).unwrap();
    let (_, bbb) = resolve(done, b).unwrap();

    match (&aaa.details, &bbb.details) {
        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Boolean),
            CanonicalSchemaletDetails::Value(SchemaletValue::Boolean),
        ) => false,
        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Array { .. }),
            CanonicalSchemaletDetails::Value(SchemaletValue::Array { .. }),
        ) => false,
        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Object(_)),
            CanonicalSchemaletDetails::Value(SchemaletValue::Object(_)),
        ) => false,
        (
            CanonicalSchemaletDetails::Value(SchemaletValue::String { .. }),
            CanonicalSchemaletDetails::Value(SchemaletValue::String { .. }),
        ) => false,
        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Integer { .. }),
            CanonicalSchemaletDetails::Value(SchemaletValue::Integer { .. }),
        ) => false,
        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Number { .. }),
            CanonicalSchemaletDetails::Value(SchemaletValue::Number { .. }),
        ) => false,

        (CanonicalSchemaletDetails::Value(_), CanonicalSchemaletDetails::Value(_)) => true,
        _ => todo!(),
    }
}

pub fn to_schemalets(resolved: &Resolved<'_>) -> anyhow::Result<Vec<(SchemaRef, Schemalet)>> {
    match resolved.schema {
        "https://json-schema.org/draft/2020-12/schema" => bootstrap::to_schemalets(resolved),
        _ => todo!(),
    }
}

trait Refers {
    fn refers(&self) -> Option<&SchemaRef>;
}

impl Refers for Schemalet {
    fn refers(&self) -> Option<&SchemaRef> {
        match &self.details {
            SchemaletDetails::ResolvedRef(reference)
            | SchemaletDetails::ResolvedDynamicRef(reference) => Some(reference),
            _ => None,
        }
    }
}

impl Refers for CanonicalSchemalet {
    fn refers(&self) -> Option<&SchemaRef> {
        if let CanonicalSchemaletDetails::Reference(reference) = &self.details {
            Some(reference)
        } else {
            None
        }
    }
}

fn resolve<'a, T>(
    wip: &'a BTreeMap<SchemaRef, T>,
    schema_ref: &SchemaRef,
) -> Option<(SchemaRef, &'a T)>
where
    T: Refers,
{
    let mut schema_ref = schema_ref;
    loop {
        let schemalet = wip.get(&schema_ref)?;
        if let Some(reference) = schemalet.refers() {
            schema_ref = reference;
        } else {
            break Some((schema_ref.clone(), schemalet));
        }
    }
}
