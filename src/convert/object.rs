use crate::{
    convert::Converter,
    schemalet::{SchemaRef, SchemaletMetadata, SchemaletValueObject},
    typespace::{
        StructProperty, StructPropertySerialization, StructPropertyState, Type, TypeStruct,
    },
};

impl Converter {
    pub(crate) fn convert_object(
        &self,
        metadata: &SchemaletMetadata,
        object: &SchemaletValueObject,
    ) -> Type<SchemaRef> {
        Type::Struct(self.convert_object_to_struct(metadata, object))
    }

    pub(crate) fn convert_object_to_struct(
        &self,
        metadata: &SchemaletMetadata,
        object: &SchemaletValueObject,
    ) -> TypeStruct<SchemaRef> {
        let SchemaletValueObject {
            properties,
            additional_properties,
        } = object;

        let properties = properties
            .iter()
            .map(|(prop_name, prop_id)| {
                StructProperty {
                    // TODO this is wrong
                    rust_name: prop_name.clone(),
                    json_name: StructPropertySerialization::Json(prop_name.clone()),
                    // TODO need to figure this out
                    state: StructPropertyState::Optional,
                    // TODO maybe a helper to pull out descriptions for property meta?
                    description: None,
                    type_id: prop_id.clone(),
                }
            })
            .collect();

        TypeStruct {
            name: "xxx".to_string(),
            description: metadata.description.clone(),
            default: None,
            properties,
            // TODO
            deny_unknown_fields: false,
        }
    }
}
