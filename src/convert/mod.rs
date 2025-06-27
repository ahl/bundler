mod one_of;

use std::collections::BTreeMap;

use crate::{
    schemalet::{CanonicalSchemalet, CanonicalSchemaletDetails, SchemaRef},
    typespace::TypespaceBuilder,
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

    pub fn convert(&mut self, id: &SchemaRef) {
        let CanonicalSchemalet { metadata, details } = self.get(id);

        let ty = match details {
            crate::schemalet::CanonicalSchemaletDetails::Anything => todo!(),
            crate::schemalet::CanonicalSchemaletDetails::Nothing => todo!(),
            crate::schemalet::CanonicalSchemaletDetails::Constant(_) => todo!(),
            crate::schemalet::CanonicalSchemaletDetails::Reference(schema_ref) => todo!(),
            crate::schemalet::CanonicalSchemaletDetails::ExclusiveOneOf { subschemas, .. } => {
                self.convert_one_of(metadata, subschemas)
            }
            crate::schemalet::CanonicalSchemaletDetails::Value(schemalet_value) => todo!(),
        };
    }
}
