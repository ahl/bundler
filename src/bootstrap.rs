use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

// bootstrapping schema
use serde::{Deserialize, Serialize};

use crate::{bool_or::ObjectOrBool, ir, ir2, Bundle, Document, Error, Resolved};

type SchemaOrBool = ObjectOrBool<Schema>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    schema: Option<String>,
    #[serde(rename = "$id", default, skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    #[serde(
        rename = "$dynamicAnchor",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    dynamic_anchor: Option<String>,
    #[serde(
        rename = "$dynamicRef",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    dynamic_ref: Option<String>,
    #[serde(rename = "$ref", default, skip_serializing_if = "Option::is_none")]
    r#ref: Option<String>,
    #[serde(
        rename = "$vocabulary",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    // TODO ignoring the validation of this one for now, and just using Value.
    vocabulary: Option<serde_json::Value>,
    #[serde(rename = "$comment", default, skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    // TODO I wonder if I should ignore defs?
    #[serde(rename = "$defs", default, skip_serializing_if = "BTreeMap::is_empty")]
    defs: BTreeMap<String, SchemaOrBool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    r#type: Option<Type>,

    // Objects
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    properties: BTreeMap<String, SchemaOrBool>,
    #[serde(
        rename = "additionalProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    additional_properties: Option<SchemaOrBool>,
    #[serde(
        rename = "propertyNames",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    property_names: Option<SchemaOrBool>,

    // Arrays
    #[serde(default, skip_serializing_if = "Option::is_none")]
    items: Option<SchemaOrBool>,
    #[serde(rename = "minItems", default, skip_serializing_if = "Option::is_none")]
    min_items: Option<u64>,
    #[serde(
        rename = "uniqueItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    unique_items: Option<bool>,

    // Subschemas
    #[serde(rename = "allOf", default, skip_serializing_if = "Option::is_none")]
    all_of: Option<NonEmpty<Vec<SchemaOrBool>>>,
    #[serde(rename = "anyOf", default, skip_serializing_if = "Option::is_none")]
    any_of: Option<NonEmpty<Vec<SchemaOrBool>>>,

    // Strings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    format: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    deprecated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    r#enum: Option<Vec<serde_json::Value>>,

    // In the real schema this probably needs to be something that can handle
    // integers and floats, but a u64 here is fine for the bootstrap schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    minimum: Option<i64>,
    #[serde(
        rename = "exclusiveMinimum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    exclusive_minimum: Option<i64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Type {
    Single(SimpleType),
    Array(NonEmpty<BTreeSet<SimpleType>>),
}

impl Type {
    fn convert(self) -> ir::Type {
        match self {
            Type::Single(t) => ir::Type::Single(t.convert()),
            Type::Array(tt) => ir::Type::Array(tt.0.into_iter().map(SimpleType::convert).collect()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
enum SimpleType {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}

impl SimpleType {
    fn convert(self) -> ir::SimpleType {
        match self {
            SimpleType::Array => ir::SimpleType::Array,
            SimpleType::Boolean => ir::SimpleType::Boolean,
            SimpleType::Integer => ir::SimpleType::Integer,
            SimpleType::Null => ir::SimpleType::Null,
            SimpleType::Number => ir::SimpleType::Number,
            SimpleType::Object => ir::SimpleType::Object,
            SimpleType::String => ir::SimpleType::String,
        }
    }

    fn all() -> impl Iterator<Item = SimpleType> {
        SimpleTypeIter(SimpleType::Array)
    }
}

struct SimpleTypeIter(SimpleType);

impl Iterator for SimpleTypeIter {
    type Item = SimpleType;

    fn next(&mut self) -> Option<Self::Item> {
        let next = match self.0 {
            SimpleType::Array => SimpleType::Boolean,
            SimpleType::Boolean => SimpleType::Integer,
            SimpleType::Integer => SimpleType::Null,
            SimpleType::Null => SimpleType::Number,
            SimpleType::Number => SimpleType::Object,
            SimpleType::Object => SimpleType::String,
            SimpleType::String => return None,
        };

        let out = self.0.clone();
        self.0 = next;
        Some(out)
    }
}

#[derive(Clone, Debug)]
struct NonEmpty<T>(T);

impl<T> Deref for NonEmpty<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Serialize for NonEmpty<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for NonEmpty<T>
where
    T: Deserialize<'de> + IsEmpty,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::try_from(T::deserialize(deserializer).unwrap())
            .map_err(|msg| <D::Error as serde::de::Error>::invalid_length(0, &msg))
    }
}

impl<T> NonEmpty<T>
where
    T: IsEmpty,
{
    // TODO TryFrom
    pub fn try_from(values: T) -> Result<Self, &'static str> {
        if values.is_empty() {
            Err("at least one item is required")
        } else {
            Ok(Self(values))
        }
    }

    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn as_inner(&self) -> &T {
        &self.0
    }
}

trait IsEmpty {
    fn is_empty(&self) -> bool;
}

impl<T> IsEmpty for Vec<T> {
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }
}
impl<T> IsEmpty for BTreeSet<T> {
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }
}

impl SchemaOrBool {
    fn iter_schema_impl(&self, path: String) -> impl Iterator<Item = (String, &Schema)> {
        let ss = match self {
            SchemaOrBool::Object(schema) => schema.iter_schema_impl(path).collect(),
            SchemaOrBool::Bool(_) => vec![],
        };
        ss.into_iter()
    }

    fn walk_schemas(&self, path: String) {
        match self {
            SchemaOrBool::Object(schema) => schema.walk_schemas(path),
            SchemaOrBool::Bool(_) => (),
        }
    }
}

impl Schema {
    pub fn iter_schema(&self) -> impl Iterator<Item = (String, &Self)> {
        self.iter_schema_impl("#".to_string())
    }

    fn iter_schema_impl(&self, path: String) -> impl Iterator<Item = (String, &Self)> {
        let Self {
            defs,
            properties,
            all_of,
            any_of,
            items,
            additional_properties,
            ..
        } = self;
        let mut out = Vec::new();
        out.push((path.clone(), self));
        all_of
            .iter()
            .flat_map(|x| x.as_inner().iter())
            .enumerate()
            .for_each(|(ii, schema)| {
                let path = format!("{path}/allOf/{ii}");
                out.extend(schema.iter_schema_impl(path))
            });
        any_of
            .iter()
            .flat_map(|x| x.as_inner().iter())
            .enumerate()
            .for_each(|(ii, schema)| {
                let path = format!("{path}/anyOf/{ii}");
                out.extend(schema.iter_schema_impl(path))
            });
        items.iter().for_each(|schema| {
            out.extend(schema.iter_schema_impl(format!("{path}/items")));
        });
        additional_properties.iter().for_each(|schema| {
            out.extend(schema.iter_schema_impl(format!("{path}/additionalProperties")));
        });
        properties.iter().for_each(|(name, schema)| {
            out.extend(schema.iter_schema_impl(format!("{path}/properties/{name}")));
        });
        defs.iter().for_each(|(name, schema)| {
            out.extend(schema.iter_schema_impl(format!("{path}/$defs/{name}")));
        });
        out.into_iter()
    }
    pub fn walk_schemas(&self, path: String) {
        let Self {
            dynamic_anchor,
            dynamic_ref,
            r#ref,
            defs,
            properties,
            all_of,
            any_of,
            items,
            additional_properties,
            ..
        } = self;

        println!("path: {path}");
        if let Some(dynamic_anchor) = dynamic_anchor {
            println!("dyn anch {}", dynamic_anchor);
        }
        if let Some(dynamic_ref) = dynamic_ref {
            println!("dyn ref {}", dynamic_ref);
        }
        if let Some(reff) = r#ref {
            println!("ref {}", reff);
        }

        all_of
            .iter()
            .flat_map(|x| x.as_inner().iter())
            .enumerate()
            .for_each(|(ii, schema)| {
                schema.walk_schemas(format!("{path}/allOf/{ii}"));
            });
        any_of
            .iter()
            .flat_map(|x| x.as_inner().iter())
            .enumerate()
            .for_each(|(ii, schema)| {
                schema.walk_schemas(format!("{path}/anyOf/{ii}"));
            });
        items.iter().for_each(|schema| {
            schema.walk_schemas(format!("{path}/items"));
        });
        additional_properties.iter().for_each(|schema| {
            schema.walk_schemas(format!("{path}/additionalProperties"));
        });
        properties.iter().for_each(|(name, schema)| {
            schema.walk_schemas(format!("{path}/properties/{name}"));
        });
        defs.iter().for_each(|(name, schema)| {
            schema.walk_schemas(format!("{path}/$defs/{name}"));
        });
    }

    pub(crate) fn populate_document(document: &mut Document) {
        let schema: Schema = serde_json::from_value(document.content.clone())
            .unwrap_or_else(|e| panic!("failed to parse '{}': {}", document.id, e));

        println!("{:?}", schema.id);

        for (path, ss) in schema.iter_schema() {
            let Self {
                dynamic_anchor,
                dynamic_ref,
                r#ref,
                ..
            } = ss;
            if let Some(dynamic_anchor) = dynamic_anchor {
                println!("dyn anch {} => {}", dynamic_anchor, path);
            }
            if let Some(dynamic_ref) = dynamic_ref {
                println!("dyn ref {}", dynamic_ref);
            }
            if let Some(reff) = r#ref {
                println!("ref {}", reff);
            }
        }
    }

    pub(crate) fn make_document(value: serde_json::Value) -> Result<Document, Error> {
        let doc = Schema::deserialize(&value).map_err(|_| Error)?;

        // TODO what to do if there's no $id?
        let id = doc.id.clone().unwrap();
        // TODO ditto the schema value
        let schema = doc.schema.clone().unwrap();

        let dyn_anchors = doc
            .iter_schema()
            .filter_map(|(path, subschema)| {
                subschema
                    .dynamic_anchor
                    .as_ref()
                    .map(|dd| (dd.clone(), path))
            })
            .collect();

        let document = Document {
            id,
            content: value,
            schema,
            anchors: Default::default(),
            dyn_anchors,
        };
        Ok(document)
    }

    // TODO keeping this around; I'm killing the idea of a generic schema
    // 12/15/2024, but we'll probably bring it back.
    pub(crate) fn to_generic(bundler: &Bundle, context: crate::Context, value: &serde_json::Value) {
        let schema = Schema::deserialize(value).unwrap();

        // TODO
        // I think the goal here was to convert relative references into
        // absolute references. Presumably the idea is to deal with dynamic
        // references as well.
        for (path, schema) in schema.iter_schema() {
            if let Some(reference) = &schema.r#ref {
                let Resolved {
                    context,
                    value,
                    schema,
                } = bundler.resolve(&context, reference).unwrap();
            }
        }
    }

    pub(crate) fn xxx_to_ir(
        resolved: &Resolved<'_>,
    ) -> anyhow::Result<Vec<(ir::SchemaRef, ir::Schema)>> {
        let bootstrap_schema = SchemaOrBool::deserialize(resolved.value)?;

        let mut work = vec![(
            ir::SchemaRef::Where(resolved.context.id.clone()),
            &bootstrap_schema,
        )];
        let mut out = Vec::new();

        while let Some((schema_ref, bootstrap_schema)) = work.pop() {
            println!("got inner work");
            println!(
                "{:#?} {}",
                schema_ref,
                serde_json::to_string_pretty(bootstrap_schema).unwrap()
            );
            match bootstrap_schema {
                ObjectOrBool::Bool(value) => {
                    let ir = ir::Schema {
                        metadata: Default::default(),
                        details: if *value {
                            ir::SchemaDetails::Anything
                        } else {
                            ir::SchemaDetails::Nothing
                        },
                    };

                    out.push((schema_ref, ir));
                }

                ObjectOrBool::Object(schema) => {
                    Self::xxx_to_ir_schema(resolved, schema.as_ref(), &mut work, &mut out)?;
                }
            }
        }

        Ok(out)
    }

    fn xxx_to_ir_schema<'a>(
        resolved: &Resolved<'_>,
        schema: &'a Schema,
        work: &mut Vec<(ir::SchemaRef, &'a SchemaOrBool)>,
        out: &mut Vec<(ir::SchemaRef, ir::Schema)>,
    ) -> anyhow::Result<()> {
        let mut parts = Vec::new();
        match &schema.r#type {
            Some(Type::Single(t)) => {
                let subparts = Self::xxx_to_ir_schema_for_type(resolved, schema, t, work, out)?;
                parts.push(subparts);
            }
            Some(Type::Array(ts)) => {
                // TODO this isn't right; I need to create an "exclusive one
                // of" in here somehow...
                let xxx = ts
                    .iter()
                    .map(|t| {
                        let xxx = Self::xxx_to_ir_schema_for_type(resolved, schema, t, work, out)?;
                        let key = xxx.0.clone();
                        out.push(xxx);
                        anyhow::Result::Ok(key)
                    })
                    .collect::<anyhow::Result<Vec<_>>>()?;
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "type array");
                parts.push((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::ExclusiveOneOf(xxx),
                    },
                ))
            }
            None => {
                // todo!()
                // TODO Any type is fine. if *some* type-specific values are
                // set... we'll need to figure something out...
            }
        }

        if let Some(ref_target) = &schema.r#ref {
            let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "$ref");
            parts.push((
                key,
                ir::Schema {
                    metadata: Default::default(),
                    details: ir::SchemaDetails::DollarRef(ref_target.clone()),
                },
            ));
        }

        if let Some(dyn_tag) = &schema.dynamic_ref {
            let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "$dynamicRef");
            parts.push((
                key,
                ir::Schema {
                    metadata: Default::default(),
                    details: ir::SchemaDetails::DynamicRef(dyn_tag.clone()),
                },
            ));
        }

        if let Some(all_of) = &schema.all_of {
            let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "allOf");
            let list = subschema_list("allOf", all_of, work, resolved);
            parts.push((
                key,
                ir::Schema {
                    metadata: Default::default(),
                    details: ir::SchemaDetails::AllOf(list),
                },
            ));
        }

        if let Some(any_of) = &schema.any_of {
            let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "allOf");
            let list = subschema_list("anyOf", any_of, work, resolved);
            parts.push((
                key,
                ir::Schema {
                    metadata: Default::default(),
                    details: ir::SchemaDetails::AnyOf(list),
                },
            ));
        }

        if let Some(enum_values) = &schema.r#enum {
            let xxx = enum_values
                .iter()
                .enumerate()
                .map(|(index, value)| {
                    let key =
                        ir::SchemaRef::Where(format!("{}/enum/{}", resolved.context.id, index));
                    out.push((
                        key.clone(),
                        ir::Schema {
                            metadata: Default::default(),
                            details: ir::SchemaDetails::Constant(value.clone()),
                        },
                    ));

                    key
                })
                .collect();
            let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "enum");
            parts.push((
                key,
                ir::Schema {
                    metadata: Default::default(),
                    details: ir::SchemaDetails::ExclusiveOneOf(xxx),
                },
            ))
        }

        println!("{:#?}", parts);

        let key = ir::SchemaRef::Where(resolved.context.id.clone());

        assert_ne!(parts.len(), 0);
        let ir = if parts.len() == 1 {
            parts.into_iter().next().unwrap().1
        } else {
            let ir = ir::Schema {
                metadata: Default::default(),
                details: ir::SchemaDetails::AllOf(
                    parts
                        .iter()
                        .map(|(schema_ref, _)| schema_ref.clone())
                        .collect(),
                ),
            };
            out.extend(parts);
            ir
        };

        out.push((key, ir));

        Ok(())
    }

    fn xxx_to_ir_schema_for_type<'a>(
        resolved: &Resolved<'_>,
        schema: &'a Schema,
        t: &SimpleType,
        work: &mut Vec<(ir::SchemaRef, &'a SchemaOrBool)>,
        out: &mut Vec<(ir::SchemaRef, ir::Schema)>,
    ) -> anyhow::Result<(ir::SchemaRef, ir::Schema)> {
        println!("t = {:#?}", t);
        match t {
            SimpleType::Array => {
                let items = schema.items.as_ref().map(|it_schema| {
                    let key = ir::SchemaRef::Where(format!("{}#/items", resolved.context.id));
                    work.push((key.clone(), it_schema));
                    key
                });
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "array");
                Ok((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::Value(ir::SchemaDetailsValue::Array(
                            ir::SchemaDetailsArray {
                                items,
                                min_items: schema.min_items,
                                unique_items: schema.unique_items.unwrap_or(false),
                            },
                        )),
                    },
                ))
            }
            SimpleType::Boolean => {
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "boolean");
                Ok((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::Value(ir::SchemaDetailsValue::Boolean),
                    },
                ))
            }
            SimpleType::Integer => {
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "integer");
                Ok((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::Value(ir::SchemaDetailsValue::Integer),
                    },
                ))
            }
            SimpleType::Null => todo!(),
            SimpleType::Number => {
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "number");
                Ok((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::Value(ir::SchemaDetailsValue::Number),
                    },
                ))
            }
            SimpleType::Object => {
                let properties = schema
                    .properties
                    .iter()
                    .map(|(prop_name, prop_schema)| {
                        let key = ir::SchemaRef::Where(format!(
                            "{}#/properties/{}",
                            resolved.context.id, prop_name
                        ));
                        work.push((key.clone(), prop_schema));
                        (prop_name.clone(), key)
                    })
                    .collect();
                let additional_properties =
                    schema.additional_properties.as_ref().map(|ap_schema| {
                        let key = ir::SchemaRef::Where(format!(
                            "{}#/additionalProperties",
                            resolved.context.id
                        ));
                        work.push((key.clone(), ap_schema));
                        key
                    });
                // Required not required for bootstrapping.
                let required = Default::default();
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "object");
                Ok((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::Value(ir::SchemaDetailsValue::Object(
                            ir::SchemaDetailsObject {
                                properties,
                                additional_properties,
                                required,
                            },
                        )),
                    },
                ))
            }
            SimpleType::String => {
                let key = ir::SchemaRef::Partial(resolved.context.id.clone(), "string");
                Ok((
                    key,
                    ir::Schema {
                        metadata: Default::default(),
                        details: ir::SchemaDetails::Value(ir::SchemaDetailsValue::String),
                    },
                ))
            }
        }
    }

    // pub(crate) fn xxx_to_ir_schema(resolved: &Resolve<'_>, schema: &Schema, work: &mut Vec<&Resolved<'_>>, out: &mut

    // pub(crate) fn to_ir(value: &serde_json::Value) -> ir::Schema {
    //     let schema = Schema::deserialize(value).unwrap();
    //     schema.convert()
    // }

    // fn convert(self) -> ir::Schema {
    //     let Schema {
    //         schema: _,
    //         id,
    //         dynamic_anchor,
    //         dynamic_ref,
    //         r#ref,
    //         vocabulary: _,
    //         comment,
    //         defs: _,
    //         title,
    //         r#type,
    //         properties,
    //         all_of,
    //         any_of,
    //         items,
    //         min_items,
    //         pattern,
    //         format,
    //         additional_properties,
    //         deprecated,
    //         default,
    //         property_names,
    //         minimum,
    //         exclusive_minimum,
    //         r#enum,
    //         unique_items,
    //     } = self;

    //     ir::Schema {
    //         metadata: ir::SchemaMetadata {
    //             id,
    //             title,
    //             comment,
    //             default,
    //         },
    //         details: ir::SchemaDetails::Any {
    //             dynamic_anchor,
    //             dynamic_ref,
    //             r#ref,
    //             r#type: r#type.map(Type::convert),
    //             properties: properties
    //                 .into_iter()
    //                 .map(|(key, schema)| (key, schema.convert()))
    //                 .collect(),
    //             all_of: all_of.map(|v| v.0.into_iter().map(SchemaOrBool::convert).collect()),
    //             any_of: any_of.map(|v| v.0.into_iter().map(SchemaOrBool::convert).collect()),
    //             one_of: None,
    //             items: items.map(SchemaOrBool::convert),
    //             min_items,
    //             pattern,
    //             format,
    //             additional_properties: additional_properties.map(SchemaOrBool::convert),
    //             deprecated,
    //             property_names: property_names.map(SchemaOrBool::convert),
    //             minimum,
    //             exclusive_minimum,
    //             r#enum,
    //             unique_items,
    //         },
    //     }
    // }
}

fn subschema_list<'a>(
    path: &'static str,
    subschemas: &'a [ObjectOrBool<Schema>],
    work: &mut Vec<(ir::SchemaRef, &'a ObjectOrBool<Schema>)>,
    resolved: &Resolved<'_>,
) -> Vec<ir::SchemaRef> {
    subschemas
        .iter()
        .enumerate()
        .map(|(index, ao_schema)| {
            let key = ir::SchemaRef::Where(format!("{}#/{}/{}", resolved.context.id, path, index));
            work.push((key.clone(), ao_schema));
            key
        })
        .collect()
}

// impl SchemaOrBool {
//     fn convert(self) -> ObjectOrBool<ir::Schema> {
//         match self {
//             // TODO interesting
//             ObjectOrBool::Bool(b) => ObjectOrBool::Bool(b),
//             ObjectOrBool::Object(s) => ObjectOrBool::Object(s.convert().into()),
//         }
//     }
// }

pub(crate) fn xxx_to_ir2(
    resolved: &Resolved<'_>,
) -> anyhow::Result<Vec<(ir2::SchemaRef, ir2::Schema)>> {
    let bootstrap_schema = SchemaOrBool::deserialize(resolved.value)?;

    let mut input = vec![(resolved.context.id.clone(), &bootstrap_schema)];
    let mut output = Vec::new();

    while let Some((id, bootstrap_subschema)) = input.pop() {
        bootstrap_subschema.to_ir2(&mut input, &mut output, &id)?;
    }

    Ok(output)
}

impl SchemaOrBool {
    fn to_ir2<'a>(
        &'a self,
        input: &mut Vec<(String, &'a SchemaOrBool)>,
        output: &mut Vec<(ir2::SchemaRef, ir2::Schema)>,
        id: &String,
    ) -> anyhow::Result<()> {
        match self {
            ObjectOrBool::Bool(value) => {
                let ir = if *value {
                    ir2::Schema::Anything
                } else {
                    ir2::Schema::Nothing
                };
                output.push((ir2::SchemaRef::Id(id.clone()), ir));
                Ok(())
            }
            ObjectOrBool::Object(schema) => schema.to_ir2(input, output, id),
        }
    }
}

impl Schema {
    fn to_ir2<'a>(
        &'a self,
        input: &mut Vec<(String, &'a SchemaOrBool)>,
        output: &mut Vec<(ir2::SchemaRef, ir2::Schema)>,
        id: &String,
    ) -> anyhow::Result<()> {
        println!();
        println!("subschema");
        println!("{}", serde_json::to_string_pretty(self).unwrap());
        let Self {
            schema: _,
            id: _,
            dynamic_anchor: _,
            dynamic_ref,
            r#ref,
            vocabulary: _,
            comment: _,
            defs: _,
            title: _,
            r#type,
            properties: _,
            additional_properties: _,
            property_names: _,
            items,
            min_items,
            unique_items,
            all_of,
            any_of,
            pattern,
            format,
            deprecated: _,
            default: _,
            r#enum,
            minimum,
            exclusive_minimum,
        } = self;

        // let mut parts = Vec::new();

        let value = match r#type {
            Some(Type::Array(types)) => {
                let mut subtypes = Vec::new();
                for tt in types.iter() {
                    let (schema_ref, ir) = self.to_ir2_for_type(input, id, tt)?;
                    output.push((schema_ref.clone(), ir));
                    subtypes.push(schema_ref);
                }

                let value_id = ir2::SchemaRef::Partial(id.clone(), "value".to_string());
                Some((value_id, ir2::Schema::ExclusiveOneOf(subtypes)))
            }
            Some(Type::Single(tt)) => Some(self.to_ir2_for_type(input, id, tt)?),
            None => None,
        };

        let all_of = Self::to_ir2_subschemas(input, id, "allOf", all_of.as_ref());
        let any_of = Self::to_ir2_subschemas(input, id, "anyOf", any_of.as_ref());

        let subref = r#ref.clone().map(|subref| {
            let value_id = ir2::SchemaRef::Partial(id.clone(), "$ref".to_string());
            let ir = ir2::Schema::DollarRef(subref);
            (value_id, ir)
        });
        let dynref = dynamic_ref.clone().map(|subref| {
            let value_id = ir2::SchemaRef::Partial(id.clone(), "$dynamicRef".to_string());
            let ir = ir2::Schema::DynamicRef(subref);
            (value_id, ir)
        });

        let everything = [value, all_of, any_of, subref, dynref]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let output_ref = ir2::SchemaRef::Id(id.clone());

        let ir = if everything.len() == 1 {
            let (_, ir) = everything.into_iter().next().unwrap();
            ir
        } else {
            let ir = ir2::Schema::AllOf(
                everything
                    .iter()
                    .map(|(schema_ref, _)| schema_ref)
                    .cloned()
                    .collect(),
            );
            output.extend(everything);
            ir
        };

        output.push((output_ref, ir));

        // assert!(r#enum.is_none());

        Ok(())
    }

    fn to_ir2_subschemas<'a>(
        input: &mut Vec<(String, &'a ObjectOrBool<Schema>)>,
        id: &String,
        label: &str,
        maybe_subschemas: Option<&'a NonEmpty<Vec<ObjectOrBool<Schema>>>>,
    ) -> Option<(ir2::SchemaRef, ir2::Schema)> {
        let all_of = maybe_subschemas.map(|subschemas| {
            let subschemas = subschemas
                .iter()
                .enumerate()
                .map(|(ii, subschema)| {
                    let a_id = format!("{}/{}/{}", id, label, ii);
                    input.push((a_id.clone(), subschema));
                    ir2::SchemaRef::Id(a_id)
                })
                .collect::<Vec<_>>();
            let all_of_id = ir2::SchemaRef::Partial(id.clone(), label.to_string());
            (all_of_id, ir2::Schema::AllOf(subschemas))
        });
        all_of
    }

    fn to_ir2_for_type<'a>(
        &'a self,
        input: &mut Vec<(String, &'a SchemaOrBool)>,
        id: &String,
        tt: &SimpleType,
    ) -> anyhow::Result<(ir2::SchemaRef, ir2::Schema)> {
        match tt {
            SimpleType::Array => {
                let items = match &self.items {
                    Some(items_schema) => {
                        let sub_id = format!("{}/items", id);
                        input.push((sub_id.clone(), items_schema));
                        Some(ir2::SchemaRef::Id(sub_id))
                    }
                    None => None,
                };
                let schema_ref = ir2::SchemaRef::Partial(id.clone(), "array".to_string());
                let ir = ir2::Schema::Value(ir2::SchemaValue::Array {
                    items,
                    min_items: self.min_items,
                    unique_items: self.unique_items,
                });
                Ok((schema_ref, ir))
            }
            SimpleType::Boolean => {
                let schema_ref = ir2::SchemaRef::Partial(id.clone(), "boolean".to_string());
                let ir = ir2::Schema::Value(ir2::SchemaValue::Boolean);
                Ok((schema_ref, ir))
            }
            SimpleType::Integer => {
                let schema_ref = ir2::SchemaRef::Partial(id.clone(), "integer".to_string());
                let ir = ir2::Schema::Value(ir2::SchemaValue::Integer {
                    minimum: self.minimum,
                    exclusive_minimum: self.exclusive_minimum,
                });
                Ok((schema_ref, ir))
            }
            SimpleType::Null => todo!(),
            SimpleType::Number => {
                let schema_ref = ir2::SchemaRef::Partial(id.clone(), "number".to_string());
                let ir = ir2::Schema::Value(ir2::SchemaValue::Number {
                    minimum: self.minimum,
                    exclusive_minimum: self.exclusive_minimum,
                });
                Ok((schema_ref, ir))
            }
            SimpleType::Object => {
                let mut properties = BTreeMap::new();
                for (prop_name, prop_schema) in &self.properties {
                    let prop_id = format!("{}/properties/{}", id, prop_name);
                    input.push((prop_id.clone(), prop_schema));
                    properties.insert(prop_name.clone(), ir2::SchemaRef::Id(prop_id.clone()));
                }
                let additional_properties = match &self.additional_properties {
                    Some(ap_schema) => {
                        let ap_id = format!("{}/additionalProperties", id);
                        input.push((ap_id.clone(), ap_schema));
                        Some(ir2::SchemaRef::Id(ap_id))
                    }
                    None => None,
                };

                let ir = ir2::Schema::Value(ir2::SchemaValue::Object {
                    properties,
                    additional_properties,
                });
                let schema_ref = ir2::SchemaRef::Partial(id.clone(), "object".to_string());

                Ok((schema_ref, ir))
            }
            SimpleType::String => {
                let schema_ref = ir2::SchemaRef::Partial(id.clone(), "string".to_string());
                let ir = ir2::Schema::Value(ir2::SchemaValue::String {
                    pattern: self.pattern.clone(),
                    format: self.format.clone(),
                });
                Ok((schema_ref, ir))
            }
        }
    }
}
