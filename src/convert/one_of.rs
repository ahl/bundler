use crate::{
    convert::Converter,
    schemalet::{CanonicalSchemaletDetails, SchemaRef, SchemaletMetadata, SchemaletValue},
};

impl Converter {
    pub(crate) fn convert_one_of(&self, metadata: &SchemaletMetadata, subschemas: &[SchemaRef]) {
        let resolved_subschemas = subschemas
            .into_iter()
            .map(|schema_ref| self.get(schema_ref))
            .collect::<Vec<_>>();

        println!(
            "subschemas {}",
            serde_json::to_string_pretty(&resolved_subschemas).unwrap()
        );

        let proto_variants = subschemas
            .iter()
            .map(|variant_id| {
                let schemalet = self.get(variant_id);
                let kind = match &schemalet.details {
                    CanonicalSchemaletDetails::Constant(value) => ProtoVariantKind::Constant {
                        value: value.clone(),
                    },
                    CanonicalSchemaletDetails::Reference(schema_ref) => todo!(),
                    CanonicalSchemaletDetails::Value(SchemaletValue::Object(object)) => {
                        let solo_prop =
                            match (object.properties.len(), object.properties.iter().next()) {
                                (1, Some((prop_name, prop_id))) => {
                                    Some((prop_name.clone(), prop_id))
                                }
                                _ => None,
                            };
                        let const_value = object
                            .properties
                            .iter()
                            .filter_map(|(prop_name, prop_id)| {
                                let prop_schema = self.resolve(prop_id);
                                if let CanonicalSchemaletDetails::Constant(value) =
                                    &prop_schema.details
                                {
                                    Some((prop_name.clone(), value.clone()))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        ProtoVariantKind::Object {
                            solo_prop,
                            const_value,
                            property_count: object.properties.len(),
                        }
                    }
                    CanonicalSchemaletDetails::Value(_) => ProtoVariantKind::Other,
                    CanonicalSchemaletDetails::Anything => todo!(),
                    CanonicalSchemaletDetails::Nothing => todo!(),
                    CanonicalSchemaletDetails::ExclusiveOneOf { typ, subschemas } => todo!(),
                };
                ProtoVariant {
                    id: variant_id,
                    name: None,
                    description: None,
                    kind,
                }
            })
            .collect::<Vec<_>>();

        if let Some(ty) = self.maybe_externally_tagged_enum(metadata, &proto_variants) {
            ty
        }
        // ... adjacent and internal

        println!("{:#?}", proto_variants);
        todo!()
    }

    fn maybe_externally_tagged_enum(
        &self,
        metadata: &SchemaletMetadata,
        proto_variants: &[ProtoVariant],
    ) -> Option<()> {
        let xxx = proto_variants
            .iter()
            .map(|variant| match &variant.kind {
                ProtoVariantKind::Constant { value } => Some((value.as_str()?.to_string(), None)),
                ProtoVariantKind::Object {
                    solo_prop: Some((prop_name, prop_id)),
                    ..
                } => Some((prop_name.clone(), Some(*prop_id))),
                _ => None,
            })
            .collect::<Option<Vec<_>>>()?;
        todo!()
    }
}

#[derive(Debug)]
struct ProtoVariant<'a> {
    id: &'a SchemaRef,
    /// A name from a part of the schema.
    name: Option<String>,
    /// A comment from a part of the schema.
    description: Option<String>,

    kind: ProtoVariantKind<'a>,
}

#[derive(Debug)]
enum ProtoVariantKind<'a> {
    Other,
    /// Constant value; appropriate for a stock externally tagged enum or an
    /// with custom serde where each variant represents a fixed value.
    Constant {
        value: serde_json::Value,
    },
    /// An object with its solo property and constant values. The former is
    /// relevant for externally tagged enums; the latter for adjacently and
    /// internally tagged enums. These are represented together because we
    /// won't know the germane interpretation until we evaluate all the
    /// variants toegether.
    Object {
        solo_prop: Option<(String, &'a SchemaRef)>,
        property_count: usize,
        const_value: Vec<(String, serde_json::Value)>,
    },
}
