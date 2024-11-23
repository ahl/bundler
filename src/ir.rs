use std::collections::{BTreeMap, BTreeSet};

use crate::bool_or::ObjectOrBool;

type SchemaOrBool = ObjectOrBool<Schema>;

#[derive(Debug)]
pub struct Schema {
    pub metadata: SchemaMetadata,
    pub details: SchemaDetails,
}

#[derive(Debug)]
pub struct SchemaMetadata {
    pub id: Option<String>,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug)]
pub enum SchemaDetails {
    Any {
        dynamic_anchor: Option<String>,
        dynamic_ref: Option<String>,
        r#ref: Option<String>,

        r#type: Option<Type>,
        properties: BTreeMap<String, SchemaOrBool>,

        all_of: Option<Vec<SchemaOrBool>>,
        any_of: Option<Vec<SchemaOrBool>>,
        one_of: Option<Vec<SchemaOrBool>>,

        items: Option<SchemaOrBool>,
        min_items: Option<u64>,
        pattern: Option<String>,
        format: Option<String>,

        additional_properties: Option<SchemaOrBool>,
        deprecated: Option<bool>,

        property_names: Option<SchemaOrBool>,
        minimum: Option<u64>,
        exclusive_minimum: Option<u64>,
        r#enum: Option<Vec<serde_json::Value>>,

        unique_items: Option<bool>,
    },
    Todo {},
}
#[derive(Clone, Debug)]
pub enum Type {
    Single(SimpleType),
    Array(BTreeSet<SimpleType>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimpleType {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}

impl Schema {
    pub fn list_subschema_refs(&self) -> Vec<String> {
        let mut out = Vec::new();
        self.list_subschema_refs_impl(&mut out);
        out
    }

    fn list_subschema_refs_impl(&self, out: &mut Vec<String>) {
        let SchemaDetails::Any {
            r#ref,
            all_of,
            any_of,
            one_of,
            ..
        } = &self.details
        else {
            panic!("this should only be called on raw schemas")
        };

        if let Some(r) = r#ref {
            out.push(r.clone());
        }

        all_of
            .iter()
            .chain(any_of)
            .chain(one_of)
            .flatten()
            .for_each(|subschema| match subschema {
                ObjectOrBool::Bool(_) => (),
                ObjectOrBool::Object(ss) => ss.list_subschema_refs_impl(out),
            });
    }
}
