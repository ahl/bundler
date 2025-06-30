use quote::format_ident;
use unicode_ident::is_xid_continue;

use crate::{
    convert::{Converter, GottenStuff},
    schemalet::{SchemaRef, SchemaletMetadata, SchemaletValueObject},
    typespace::{StructProperty, StructPropertySerde, StructPropertyState, Type, TypeStruct},
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
            ..
        } = object;

        assert!(additional_properties.is_none());

        let prop_names = properties
            .keys()
            .map(|prop_name| tmp_sanitize(prop_name))
            .collect::<Vec<_>>();

        let properties = properties
            .iter()
            .zip(prop_names)
            .map(|((prop_name, prop_id), new_prop_name)| {
                let GottenStuff {
                    id,
                    schemalet: _,
                    description,
                    title: _,
                } = self.resolve_and_get_stuff(prop_id);

                let rust_name = format_ident!("{new_prop_name}");
                let json_name = if *prop_name == new_prop_name {
                    StructPropertySerde::None
                } else {
                    StructPropertySerde::Rename(prop_name.clone())
                };
                StructProperty {
                    rust_name,
                    json_name,
                    // TODO need to figure this out
                    state: StructPropertyState::Optional,
                    // TODO maybe a helper to pull out descriptions for property meta?
                    description,
                    type_id: id.clone(),
                }
            })
            .collect();

        TypeStruct {
            description: metadata.description.clone(),
            default: None,
            properties,
            deny_unknown_fields: false,
        }
    }
}

fn tmp_sanitize(prop_name: &str) -> String {
    use heck::ToSnakeCase;

    let x = prop_name.replace(|ch| !is_xid_continue(ch), "-");

    let mut out = x.to_snake_case();

    if syn::parse_str::<syn::Ident>(&out).is_err() {
        out.push('_');
    }

    out
}
