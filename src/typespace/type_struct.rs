use crate::typespace::JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeStruct<Id> {
    // TODO fancy naming?
    pub name: String,
    //     pub rename: Option<String>,
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub properties: Vec<StructProperty<Id>>,
    pub deny_unknown_fields: bool,
    //     pub schema: SchemaWrapper,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructProperty<Id> {
    pub rust_name: String,
    pub json_name: StructPropertySerialization,
    pub state: StructPropertyState,
    pub description: Option<String>,
    pub type_id: Id,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructPropertySerialization {
    Json(String),
    Flatten,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructPropertyState {
    Required,
    Optional,
    Default(JsonValue),
}
