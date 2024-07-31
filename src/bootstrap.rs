use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Deref,
};

// bootstrapping schema
use serde::{Deserialize, Serialize};

use crate::{bool_or::ObjectOrBool, Bundle, Document, Error, Resolved};

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
    // TODO ignoring the value of this one for now
    vocabulary: Option<serde_json::Value>,
    #[serde(rename = "$comment", default, skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
    #[serde(rename = "$defs", default, skip_serializing_if = "BTreeMap::is_empty")]
    defs: BTreeMap<String, SchemaOrBool>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    r#type: Option<Type>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    properties: BTreeMap<String, SchemaOrBool>,
    #[serde(rename = "allOf", default, skip_serializing_if = "Option::is_none")]
    all_of: Option<NonEmpty<Vec<SchemaOrBool>>>,
    #[serde(rename = "anyOf", default, skip_serializing_if = "Option::is_none")]
    any_of: Option<NonEmpty<Vec<SchemaOrBool>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    items: Option<SchemaOrBool>,
    #[serde(rename = "minItems", default, skip_serializing_if = "Option::is_none")]
    min_items: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pattern: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    format: Option<String>,

    #[serde(
        rename = "additionalProperties",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    additional_properties: Option<SchemaOrBool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    deprecated: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    default: Option<serde_json::Value>,

    #[serde(
        rename = "propertyNames",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    property_names: Option<SchemaOrBool>,
    // In the real schema this probably needs to be something that can handle
    // integers and floats, but a u64 here is fine for the bootstrap schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    minimum: Option<u64>,
    #[serde(
        rename = "exclusiveMinimum",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    exclusive_minimum: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    r#enum: Option<Vec<serde_json::Value>>,

    #[serde(
        rename = "uniqueItems",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    unique_items: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Type {
    Single(SimpleType),
    Array(NonEmpty<BTreeSet<SimpleType>>),
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
}
