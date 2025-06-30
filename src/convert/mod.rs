mod array;
mod object;
mod one_of;

use std::collections::BTreeMap;

use crate::{
    schemalet::{
        CanonicalSchemalet, CanonicalSchemaletDetails, SchemaRef, SchemaletDetails, SchemaletValue,
    },
    typespace::{Type, TypespaceBuilder},
};

// TODO naming?
pub struct Converter {
    typespace: TypespaceBuilder<SchemaRef>,
    graph: BTreeMap<SchemaRef, CanonicalSchemalet>,
}

impl Converter {
    pub fn new(graph: BTreeMap<SchemaRef, CanonicalSchemalet>) -> Self {
        Self {
            typespace: Default::default(),
            graph,
        }
    }

    fn get<'a>(&'a self, id: &SchemaRef) -> &'a CanonicalSchemalet {
        self.graph.get(id).unwrap()
    }

    fn resolve<'a>(&'a self, mut id: &'a SchemaRef) -> &'a CanonicalSchemalet {
        loop {
            let schemalet = self.get(id);
            if let CanonicalSchemaletDetails::Reference(schema_ref) = &schemalet.details {
                id = schema_ref;
            } else {
                break schemalet;
            }
        }
    }

    pub fn resolve_and_get_stuff<'a>(&'a self, mut id: &'a SchemaRef) -> GottenStuff<'a> {
        let mut title = None;
        let mut description = None;
        loop {
            let schemalet = self.get(id);

            let CanonicalSchemaletDetails::Reference(next_id) = &schemalet.details else {
                return GottenStuff {
                    id,
                    schemalet,
                    description,
                    title,
                };
            };

            if let (None, Some(new_title)) = (&title, &schemalet.metadata.title) {
                title = Some(new_title.clone());
            }
            if let (None, Some(new_description)) = (&description, &schemalet.metadata.description) {
                description = Some(new_description.clone());
            }

            id = next_id;
        }
    }

    pub fn convert(&mut self, id: &SchemaRef) -> Type<SchemaRef> {
        let schemalet = self.get(id);
        println!(
            "converting {}",
            serde_json::to_string_pretty(schemalet).unwrap(),
        );
        let CanonicalSchemalet { metadata, details } = schemalet;

        match details {
            CanonicalSchemaletDetails::Anything => Type::JsonValue,
            CanonicalSchemaletDetails::Nothing => todo!(),
            CanonicalSchemaletDetails::Constant(_) => todo!(),
            CanonicalSchemaletDetails::Reference(schema_ref) => todo!(),
            CanonicalSchemaletDetails::Note(schema_ref) => todo!(),
            CanonicalSchemaletDetails::ExclusiveOneOf { subschemas, .. } => {
                self.convert_one_of(metadata, subschemas)
            }

            CanonicalSchemaletDetails::Value(SchemaletValue::Boolean) => Type::Boolean,
            CanonicalSchemaletDetails::Value(SchemaletValue::Array(array)) => {
                self.convert_array(metadata, array)
            }
            CanonicalSchemaletDetails::Value(SchemaletValue::Object(object)) => {
                self.convert_object(metadata, object)
            }
            CanonicalSchemaletDetails::Value(SchemaletValue::String { pattern, format }) => {
                self.convert_string(metadata, pattern.as_ref(), format.as_ref())
            }
            CanonicalSchemaletDetails::Value(SchemaletValue::Integer {
                minimum,
                exclusive_minimum,
            }) => {
                // TODO not handling this well ...
                Type::Float("i64".to_string())
            }
            CanonicalSchemaletDetails::Value(SchemaletValue::Number {
                minimum,
                exclusive_minimum,
            }) => {
                // TODO not handling this well ...
                Type::Float("f64".to_string())
            }
            CanonicalSchemaletDetails::Value(SchemaletValue::Null) => todo!(),
        }
    }

    fn convert_string(
        &self,
        metadata: &crate::schemalet::SchemaletMetadata,
        pattern: Option<&String>,
        format: Option<&String>,
    ) -> Type<SchemaRef> {
        match (pattern, format) {
            (_, _) => Type::String,
            // _ => panic!("{:?} {:?}", pattern, format),
        }
    }
}

pub struct GottenStuff<'a> {
    id: &'a SchemaRef,
    schemalet: &'a CanonicalSchemalet,
    description: Option<String>,
    title: Option<String>,
}
