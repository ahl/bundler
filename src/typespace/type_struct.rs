use syn::Ident;

use crate::{
    namespace::Name,
    typespace::{InternalId, JsonValue, NameBuilder},
};

#[derive(Debug, Clone)]
pub struct TypeStruct<Id> {
    pub name: NameBuilder<InternalId<Id>>,
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub properties: Vec<StructProperty<Id>>,
    pub deny_unknown_fields: bool,

    pub built: Option<TypeStructBuilt<Id>>,
}

#[derive(Debug, Clone)]
struct TypeStructBuilt<Id> {
    name: Name<Id>,
}

// pub struct TypeStructBuilder<Id> {
//     pub description: Option<String>,
//     pub default: Option<JsonValue>,
//     pub properties: Vec<TypeStructPropertyBuilder<Id>>,
//     pub deny_unknown_fields: bool,
// }
// pub struct TypeStructPropertyBuilder<Id> {
//     pub rust_name: String,
//     pub json_name: Struct
// }

impl<Id> TypeStruct<Id>
where
    Id: Clone,
{
    pub fn new(
        name: NameBuilder<Id>,
        description: Option<String>,
        default: Option<JsonValue>,
        properties: Vec<StructProperty<Id>>,
        deny_unknown_fields: bool,
    ) -> Self {
        Self {
            name: name.into(),
            description,
            default,
            properties,
            deny_unknown_fields,
            built: None,
        }
    }
    pub(crate) fn children(&self) -> Vec<InternalId<Id>> {
        self.properties
            .iter()
            .map(|StructProperty { type_id, .. }| type_id.clone())
            .collect()
    }

    pub(crate) fn children_with_context(&self) -> Vec<(InternalId<Id>, String)> {
        self.properties
            .iter()
            .map(
                |StructProperty {
                     rust_name, type_id, ..
                 }| (type_id.clone(), format!("{rust_name}")),
            )
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructProperty<Id> {
    pub rust_name: Ident,
    pub json_name: StructPropertySerde,
    pub state: StructPropertyState,
    pub description: Option<String>,
    pub type_id: InternalId<Id>,
}

impl<Id> StructProperty<Id> {
    pub fn new(
        rust_name: Ident,
        json_name: StructPropertySerde,
        state: StructPropertyState,
        description: Option<String>,
        type_id: Id,
    ) -> Self {
        Self {
            rust_name,
            json_name,
            state,
            description,
            type_id: type_id.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructPropertySerde {
    None,
    Rename(String),
    Flatten,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructPropertyState {
    Required,
    Optional,
    Default(JsonValue),
}
