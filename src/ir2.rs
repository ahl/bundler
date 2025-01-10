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
    AnyOf(Vec<SchemaRef>),
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
    pub fn simplify(self, done: &BTreeMap<SchemaRef, Schema>) -> State {
        match self {
            Schema::Anything => State::Canonical(self),
            Schema::Nothing => State::Canonical(self),
            Schema::DollarRef(ref rr) => {
                if done.contains_key(&SchemaRef::Id(rr.clone())) {
                    State::Canonical(self)
                } else {
                    State::Stuck(self)
                }
            }
            Schema::DynamicRef(_) => State::Stuck(self),
            Schema::Constant(_) => State::Canonical(self),
            Schema::Value(_) => State::Canonical(self),
            Schema::AllOf(ref subchema_refs) => {
                if let Some(subschemas) = subchema_refs
                    .iter()
                    .map(|schema_ref| resolve(done, schema_ref))
                    .collect::<Option<Vec<_>>>()
                {
                    let schema = merge_all(subschemas, done);
                    State::Simplified(schema)
                } else {
                    State::Stuck(self)
                }
            }
            Schema::ExclusiveOneOf(ref subschemas) => {
                if subschemas.iter().all(|ss_ref| done.contains_key(ss_ref)) {
                    // TODO I think I need to eliminate oneOf subschemas
                    State::Canonical(self)
                } else {
                    State::Stuck(self)
                }
            }
            Schema::AnyOf(_) => State::Todo,
        }
    }

    fn is_canonical(&self, done: &BTreeMap<SchemaRef, Schema>) -> bool {
        match self {
            Schema::Anything => true,
            Schema::Nothing => true,
            Schema::DollarRef(_) => false,
            Schema::DynamicRef(_) => false,
            Schema::Value(_) => true,
            Schema::Constant(_) => true,
            Schema::AllOf(_) => false,
            Schema::AnyOf(_) => false,
            Schema::ExclusiveOneOf(vec) => vec.iter().all(|schema_ref| {
                let Some(schema) = done.get(schema_ref) else {
                    return false;
                };

                match schema {
                    Schema::ExclusiveOneOf(_) => false,
                    _ => schema.is_canonical(done),
                }
            }),
        }
    }
}

fn resolve<'a>(
    done: &'a BTreeMap<SchemaRef, Schema>,
    schema_ref: &SchemaRef,
) -> Option<(SchemaRef, &'a Schema)> {
    let mut schema_ref = schema_ref.clone();
    loop {
        let schema = done.get(&schema_ref)?;
        let Schema::DollarRef(dollar_ref) = schema else {
            break Some((schema_ref, schema));
        };
        schema_ref = SchemaRef::Id(dollar_ref.clone());
    }
}

fn merge_all(subschemas: Vec<(SchemaRef, &Schema)>, done: &BTreeMap<SchemaRef, Schema>) -> Schema {
    let len = subschemas.len();
    println!(
        "merge {}",
        serde_json::to_string_pretty(&subschemas).unwrap()
    );

    for (_, schema) in &subschemas {
        assert!(schema.is_canonical(done));
    }

    let mut xors = Vec::new();
    let mut rest = Vec::new();
    for (schema_ref, schema) in subschemas {
        match schema {
            Schema::ExclusiveOneOf(ss) => xors.push(ss),
            _ => rest.push((schema_ref, schema)),
        }
    }
    if let Some(subschemas) = xors.pop() {
        let mut all_of_groups = subschemas
            .iter()
            .map(|schema_ref| (schema_ref, vec![schema_ref]))
            .collect::<Vec<_>>();

        for xor_schema in xors {
            all_of_groups = all_of_groups
                .into_iter()
                .flat_map(|(representative, group)| {
                    xor_schema
                        .iter()
                        .filter(|&schema_ref| {
                            !trivially_incompatible(done, representative, schema_ref)
                        })
                        .map(move |schema_ref| {
                            let mut new_group = group.clone();
                            new_group.push(schema_ref);
                            (representative, new_group)
                        })
                })
                .collect::<Vec<_>>();
        }

        for (_, group) in &mut all_of_groups {
            group.extend(rest.iter().map(|(schema_ref, _)| schema_ref));
        }

        println!(
            "? {}",
            serde_json::to_string_pretty(&all_of_groups).unwrap()
        );
        println!("?! {} {}", len, all_of_groups.len());

        todo!()
    } else {
        todo!()
    }
}

fn trivially_incompatible(
    done: &BTreeMap<SchemaRef, Schema>,
    a: &SchemaRef,
    b: &SchemaRef,
) -> bool {
    let (_, aaa) = resolve(done, a).unwrap();
    let (_, bbb) = resolve(done, b).unwrap();

    match (aaa, bbb) {
        (Schema::Value(aaaa), Schema::Value(bbbb)) =>
        {
            #[allow(clippy::match_like_matches_macro)]
            !match (aaaa, bbbb) {
                (SchemaValue::Boolean, SchemaValue::Boolean) => true,
                (SchemaValue::Object { .. }, SchemaValue::Object { .. }) => true,
                _ => false,
            }
        }
        _ => false,
    }
}
