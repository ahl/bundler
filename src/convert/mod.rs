mod object;
mod one_of;

use std::collections::BTreeMap;

use crate::{
    schemalet::{CanonicalSchemalet, CanonicalSchemaletDetails, SchemaRef, SchemaletValue},
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

    pub fn convert(&mut self, id: &SchemaRef) -> Type<SchemaRef> {
        let CanonicalSchemalet { metadata, details } = self.get(id);

        match details {
            CanonicalSchemaletDetails::Anything => todo!(),
            CanonicalSchemaletDetails::Nothing => todo!(),
            CanonicalSchemaletDetails::Constant(_) => todo!(),
            CanonicalSchemaletDetails::Reference(schema_ref) => todo!(),
            CanonicalSchemaletDetails::ExclusiveOneOf { subschemas, .. } => {
                self.convert_one_of(metadata, subschemas)
            }

            CanonicalSchemaletDetails::Value(SchemaletValue::Boolean) => todo!(),
            CanonicalSchemaletDetails::Value(SchemaletValue::Array {
                items,
                min_items,
                unique_items,
            }) => todo!(),
            CanonicalSchemaletDetails::Value(SchemaletValue::Object(object)) => {
                self.convert_object(metadata, object)
            }
            CanonicalSchemaletDetails::Value(SchemaletValue::String { pattern, format }) => todo!(),
            CanonicalSchemaletDetails::Value(SchemaletValue::Integer {
                minimum,
                exclusive_minimum,
            }) => todo!(),
            CanonicalSchemaletDetails::Value(SchemaletValue::Number {
                minimum,
                exclusive_minimum,
            }) => todo!(),
            CanonicalSchemaletDetails::Value(SchemaletValue::Null) => todo!(),
        }
    }
}
