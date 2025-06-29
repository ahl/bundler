use syn::Ident;

use crate::typespace::JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeStruct<Id> {
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub properties: Vec<StructProperty<Id>>,
    pub deny_unknown_fields: bool,
}
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
