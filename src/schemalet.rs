use std::{collections::BTreeMap, fmt::Display, ops::Deref};

use serde::Serialize;
use url::Url;

use crate::{bootstrap, Resolved};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaRef {
    Id(String),
    Partial(String, String),

    // TODO Could this be yes/no?
    Merge(Vec<SchemaRef>),
    YesNo {
        yes: Box<SchemaRef>,
        no: Vec<SchemaRef>,
    },
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
            SchemaRef::Merge(schema_refs) => {
                f.write_str("<merge> [\n")?;
                for schema_ref in schema_refs {
                    f.write_str("  ")?;
                    schema_ref.fmt(f)?;
                    f.write_str("\n")?;
                }
                f.write_str("]")
            }
            SchemaRef::YesNo { yes, no } => {
                f.write_str("<yes/no> [\n  ")?;
                yes.fmt(f)?;
                f.write_str("\n")?;
                for schema_ref in no {
                    f.write_str("  ")?;
                    schema_ref.fmt(f)?;
                    f.write_str("\n")?;
                }
                f.write_str("]")
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
    YesNo { yes: SchemaRef, no: Vec<SchemaRef> },
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

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub enum SchemaletType {
    Boolean,
    Array,
    Object,
    String,
    Integer,
    Number,
    Null,
}

// TODO don't worry about naming for now, but this will probably be the most
// relevant output type
#[derive(Serialize, Debug, Clone)]
pub struct CanonicalSchemalet {
    #[serde(flatten)]
    pub metadata: SchemaletMetadata,
    pub details: CanonicalSchemaletDetails,
}

impl Deref for CanonicalSchemalet {
    type Target = CanonicalSchemaletDetails;

    fn deref(&self) -> &Self::Target {
        &self.details
    }
}

impl CanonicalSchemaletDetails {
    fn get_type(&self) -> Option<SchemaletType> {
        match self {
            CanonicalSchemaletDetails::Constant(value) => match value {
                serde_json::Value::Null => Some(SchemaletType::Null),
                serde_json::Value::Bool(_) => Some(SchemaletType::Boolean),
                serde_json::Value::Number(_) => {
                    todo!()
                }
                serde_json::Value::String(_) => Some(SchemaletType::String),
                serde_json::Value::Array(_) => Some(SchemaletType::Array),
                serde_json::Value::Object(_) => Some(SchemaletType::Object),
            },
            CanonicalSchemaletDetails::Anything => None,
            CanonicalSchemaletDetails::Nothing => None,
            // TODO maybe we should handle this differently?
            CanonicalSchemaletDetails::Reference(_) => todo!(),
            CanonicalSchemaletDetails::ExclusiveOneOf { typ, .. } => typ.clone(),
            CanonicalSchemaletDetails::Value(value) => match value {
                SchemaletValue::Boolean => Some(SchemaletType::Boolean),
                SchemaletValue::Array { .. } => Some(SchemaletType::Array),
                SchemaletValue::Object(_) => Some(SchemaletType::Object),
                SchemaletValue::String { .. } => Some(SchemaletType::String),
                SchemaletValue::Integer { .. } => Some(SchemaletType::Integer),
                SchemaletValue::Number { .. } => Some(SchemaletType::Number),
            },
        }
    }

    fn is_nothing(&self) -> bool {
        matches!(self, CanonicalSchemaletDetails::Nothing)
    }
}

#[derive(Serialize, Debug, Clone)]
pub enum CanonicalSchemaletDetails {
    Anything,
    Nothing,
    Constant(serde_json::Value),
    // TODO 6/14/2025 not 100% sure where this is going to be used, but it
    // might be interesting
    // TODO 6/14/2025 yeah this is going to be important: we're going to want
    // to make sure we don't lose description data e.g. so that a struct field
    // has a comment and so does its type. We'll want to keep metadata. Typify
    // will need to deal with it by walking this linked list.
    Reference(SchemaRef),
    ExclusiveOneOf {
        /// Cached type iff all subschemas share a single type.
        typ: Option<SchemaletType>,
        /// Component subschemas.
        subschemas: Vec<SchemaRef>,
    },
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
            SchemaletDetails::OneOf(..) => todo!(),
            SchemaletDetails::Not(..) => todo!(),
            SchemaletDetails::IfThen(..) => todo!(),
            SchemaletDetails::IfThenElse(..) => todo!(),
            SchemaletDetails::RawRef(_) => todo!(),
            SchemaletDetails::RawDynamicRef(_) => todo!(),
            SchemaletDetails::AllOf(schema_refs) => {
                if let Some(subschemas) = resolve_all(done, &schema_refs) {
                    println!("{}", serde_json::to_string_pretty(&subschemas).unwrap());
                    merge_all(metadata, subschemas, done)
                } else {
                    State::Stuck(Schemalet {
                        metadata,
                        details: SchemaletDetails::AllOf(schema_refs),
                    })
                }
            }
            SchemaletDetails::AnyOf(schema_refs) => {
                if let Some(subschemas) = resolve_all(done, &schema_refs) {
                    println!(
                        "canonical anyof {}",
                        serde_json::to_string_pretty(&subschemas).unwrap()
                    );
                    expand_any_of(metadata, subschemas)
                } else {
                    State::Stuck(Schemalet {
                        metadata,
                        details: SchemaletDetails::AnyOf(schema_refs),
                    })
                }
            }
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
            SchemaletDetails::Value(value) => {
                todo!();
                State::Canonical(CanonicalSchemalet {
                    metadata,
                    details: CanonicalSchemaletDetails::Value(value),
                })
            }
            SchemaletDetails::ExclusiveOneOf(schema_refs) => {
                if let Some(subschemas) = resolve_all(done, &schema_refs) {
                    let subschemas = subschemas
                        .into_iter()
                        .filter(|(_, schemalet)| !schemalet.is_nothing())
                        .collect::<Vec<_>>();

                    let new_schema = match subschemas.len() {
                        0 => CanonicalSchemalet {
                            metadata,
                            details: CanonicalSchemaletDetails::Nothing,
                        },

                        1 => {
                            let xxx = subschemas.into_iter().next().unwrap().0;
                            CanonicalSchemalet {
                                metadata,
                                details: CanonicalSchemaletDetails::Reference(xxx),
                            }
                        }

                        _ => {
                            let typ = subschemas
                                .iter()
                                .map(|(_, schemalet)| schemalet.get_type())
                                .reduce(|a, b| match (a, b) {
                                    (Some(aa), Some(bb)) if aa == bb => Some(aa),
                                    _ => None,
                                })
                                .flatten();
                            let subschemas = subschemas
                                .into_iter()
                                .map(|(schema_ref, _)| schema_ref)
                                .collect();

                            CanonicalSchemalet {
                                metadata,
                                details: CanonicalSchemaletDetails::ExclusiveOneOf {
                                    typ,
                                    subschemas,
                                },
                            }
                        }
                    };

                    // TODO we need to remove any `Never` schemalets and then
                    // special case 1 => the type, and 0 => Never
                    // TODO memoize the type
                    State::Canonical(new_schema)
                } else {
                    State::Stuck(Schemalet {
                        metadata,
                        details: SchemaletDetails::ExclusiveOneOf(schema_refs),
                    })
                }
            }
            SchemaletDetails::YesNo { yes, no } => {
                let ryes = resolve(done, &yes);
                let rno = no
                    .iter()
                    .map(|sr| resolve(done, sr))
                    .collect::<Option<Vec<_>>>();
                if let (Some(yes), Some(no)) = (ryes, rno) {
                    println!(
                        "yes/no {}",
                        serde_json::to_string_pretty(&serde_json::json!({ "yes": yes, "no": no }))
                            .unwrap()
                    );
                    merge_yes_no(yes, no)
                } else {
                    State::Stuck(Schemalet {
                        metadata,
                        details: SchemaletDetails::YesNo { yes, no },
                    })
                }
            }
        }
    }
}

fn merge_yes_no(
    yes: (SchemaRef, &CanonicalSchemalet),
    no: Vec<(SchemaRef, &CanonicalSchemalet)>,
) -> State {
    if let Some(typ) = yes.1.get_type() {
        if no.iter().all(|(_, no_subschema)| {
            no_subschema
                .get_type()
                .map_or(false, |no_typ| no_typ != typ)
        }) {
            return State::Simplified(
                Schemalet {
                    metadata: Default::default(),
                    details: SchemaletDetails::ResolvedRef(yes.0),
                },
                Default::default(),
            );
        }
    }

    match &yes.1.details {
        CanonicalSchemaletDetails::Anything => todo!(),
        CanonicalSchemaletDetails::Nothing => State::Simplified(
            Schemalet {
                metadata: Default::default(),
                details: SchemaletDetails::ResolvedRef(yes.0),
            },
            Default::default(),
        ),

        CanonicalSchemaletDetails::Constant(value) => todo!(),
        CanonicalSchemaletDetails::Reference(schema_ref) => todo!(),

        CanonicalSchemaletDetails::ExclusiveOneOf { typ, subschemas } => {
            todo!()
        }

        CanonicalSchemaletDetails::Value(schemalet_value) => {
            todo!()
        }
    }
}

fn expand_any_of(
    metadata: SchemaletMetadata,
    subschemas: Vec<(SchemaRef, &CanonicalSchemalet)>,
) -> State {
    let len = subschemas.len();

    // TODO this could be a lot smarter by looking at the schemas
    let permutations = (1..(1 << len))
        .map(|bitmap| {
            let mut yes = Vec::new();
            let mut no = Vec::new();

            for (ii, (schema_ref, _)) in subschemas.iter().enumerate() {
                if (1 << ii) & bitmap != 0 {
                    yes.push(schema_ref.clone());
                } else {
                    no.push(schema_ref.clone());
                }
            }

            (yes, no)
        })
        .collect::<Vec<_>>();
    println!("yes/no {:#?}", permutations);

    let mut new_work = Vec::new();
    let mut new_subschemas = Vec::new();

    for (yes, no) in permutations {
        let yes = match yes.as_slice() {
            [] => unreachable!(),
            [solo] => solo.clone(),
            all => {
                let schema_refs = all.iter().cloned().collect::<Vec<_>>();
                let merge_ref = SchemaRef::Merge(schema_refs.clone());
                let merge = Schemalet {
                    metadata: Default::default(),
                    details: SchemaletDetails::AllOf(schema_refs),
                };

                new_work.push((merge_ref.clone(), merge));
                merge_ref
            }
        };

        let new_ref = SchemaRef::YesNo {
            yes: Box::new(yes.clone()),
            no: no.clone(),
        };

        let new_subschema = Schemalet {
            metadata: Default::default(),
            details: SchemaletDetails::YesNo { yes, no },
        };

        new_work.push((new_ref.clone(), new_subschema));
        new_subschemas.push(new_ref);
    }

    let new_schemalet = Schemalet {
        metadata,
        details: SchemaletDetails::ExclusiveOneOf(new_subschemas),
    };

    State::Simplified(new_schemalet, new_work)
}

// TODO 6/14/2025 not fully sure why we need the done map...
fn merge_all(
    metadata: SchemaletMetadata,
    subschemas: Vec<(SchemaRef, &CanonicalSchemalet)>,
    done: &BTreeMap<SchemaRef, CanonicalSchemalet>,
) -> State {
    // Separate out xors (disjunctions) from other schemas.
    let mut xors = Vec::new();
    let mut rest = Vec::new();
    for (schema_ref, schema) in subschemas {
        match &schema.details {
            CanonicalSchemaletDetails::ExclusiveOneOf { subschemas, .. } => xors.push(subschemas),
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

        // TODO do we know anything about the cardinality of `groups` at this
        // point i.e. do we know that it's >1?

        println!(
            "groups {}",
            serde_json::to_string_pretty(&merge_groups).unwrap()
        );

        // let xxx = merge_groups.iter().map(|group| {
        //     let subschemas = group
        //         .iter()
        //         .map(|schema_ref| {
        //             resolve(done, schema_ref)
        //                 .expect("already resolved previously, so should be infallible")
        //                 .1
        //         })
        //         .collect::<Vec<_>>();
        //     merge_all_values(subschemas)
        //     todo!();
        //     todo!()
        // });

        let mut new_work = Vec::new();
        let mut new_subschemas = Vec::new();

        for group in merge_groups {
            let refs = group.into_iter().cloned().collect::<Vec<_>>();
            let new_schemaref = SchemaRef::Merge(refs.clone());
            let new_schemalet = Schemalet {
                metadata: Default::default(),
                details: SchemaletDetails::AllOf(refs.clone()),
            };

            new_work.push((new_schemaref.clone(), new_schemalet));
            new_subschemas.push(new_schemaref);
        }

        let new_schemalet = Schemalet {
            metadata,
            details: SchemaletDetails::ExclusiveOneOf(new_subschemas),
        };

        State::Simplified(new_schemalet, new_work)
    } else {
        // Here we know that we've got a flat collection of canonical
        // schemalets with no nesting. We can also assume that the list of
        // subschemas is non-empty.

        let subschemas = rest
            .into_iter()
            .map(|(_, schemalet)| schemalet)
            .collect::<Vec<_>>();

        // TODO 6/14/2025
        // I need to be thoughtful about when I can and don't preserve
        // metadata. For example, some metadata might become comments on struct
        // fields.

        let mut merged_details = CanonicalSchemaletDetails::Anything;

        for subschema in subschemas {
            merged_details = merge_two(&merged_details, &subschema.details);
        }

        println!(
            "merged {}",
            serde_json::to_string_pretty(&merged_details).unwrap()
        );

        let new_schemalet = CanonicalSchemalet {
            metadata,
            details: merged_details,
        };

        State::Canonical(new_schemalet)
    }
}

fn merge_two(
    a: &CanonicalSchemaletDetails,
    b: &CanonicalSchemaletDetails,
) -> CanonicalSchemaletDetails {
    match (a.get_type(), b.get_type()) {
        (Some(aa), Some(bb)) if aa != bb => return CanonicalSchemaletDetails::Nothing,
        _ => (),
    }
    match (a, b) {
        (CanonicalSchemaletDetails::Anything, other)
        | (other, CanonicalSchemaletDetails::Anything) => other.clone(),

        (CanonicalSchemaletDetails::Nothing, _) | (_, CanonicalSchemaletDetails::Nothing) => {
            CanonicalSchemaletDetails::Nothing
        }

        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Boolean),
            CanonicalSchemaletDetails::Value(SchemaletValue::Boolean),
        ) => CanonicalSchemaletDetails::Value(SchemaletValue::Boolean),

        (
            CanonicalSchemaletDetails::Value(SchemaletValue::Object(aa)),
            CanonicalSchemaletDetails::Value(SchemaletValue::Object(bb)),
        ) => merge_two_objects(aa, bb),

        _ => todo!(
            "merge_two {}",
            serde_json::to_string_pretty(&[a, b]).unwrap()
        ),
    }
}

fn merge_two_objects(
    aa: &SchemaletValueObject,
    bb: &SchemaletValueObject,
) -> CanonicalSchemaletDetails {
    let prop_names = aa.properties.keys().chain(bb.properties.keys());
    let properties = prop_names
        .map(
            |prop_name| match (aa.properties.get(prop_name), bb.properties.get(prop_name)) {
                (None, None) => unreachable!("must exist in one or the other"),
                (None, Some(prop_ref)) | (Some(prop_ref), None) => {
                    // TODO need to consider the *other* object's
                    // additionalProperties field.
                    (prop_name.clone(), prop_ref.clone())
                }
                (Some(_), Some(_)) => todo!(),
            },
        )
        .collect();

    let additional_properties = match (&aa.additional_properties, &bb.additional_properties) {
        (None, None) => None,
        (None, Some(other)) | (Some(other), None) => Some(other.clone()),
        (Some(_), Some(_)) => todo!(),
    };

    CanonicalSchemaletDetails::Value(SchemaletValue::Object(SchemaletValueObject {
        properties,
        additional_properties,
    }))
}

fn trivially_incompatible(
    done: &BTreeMap<SchemaRef, CanonicalSchemalet>,
    a: &SchemaRef,
    b: &SchemaRef,
) -> bool {
    let (_, aaa) = resolve(done, a).unwrap();
    let (_, bbb) = resolve(done, b).unwrap();

    match (aaa.get_type(), bbb.get_type()) {
        (Some(a_type), Some(b_type)) if a_type != b_type => true,
        _ => false,
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

fn resolve_all<'a, T, I>(
    wip: &'a BTreeMap<SchemaRef, T>,
    schemas: I,
) -> Option<Vec<(SchemaRef, &'a T)>>
where
    T: Refers,
    I: IntoIterator<Item = &'a SchemaRef>,
{
    schemas
        .into_iter()
        .map(|schema_ref| resolve(wip, schema_ref))
        .collect()
}
