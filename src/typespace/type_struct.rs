use syn::Ident;

use crate::typespace::{JsonValue, NameBuilder};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeStruct<Id> {
    pub name: NameBuilder<Id>,
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub properties: Vec<StructProperty<Id>>,
    pub deny_unknown_fields: bool,
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
    pub(crate) fn children(&self) -> Vec<Id> {
        self.properties
            .iter()
            .map(|StructProperty { type_id, .. }| type_id.clone())
            .collect()
    }

    pub(crate) fn children_with_context(&self) -> Vec<(Id, String)> {
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
    pub type_id: Id,
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
