use crate::typespace::JsonValue;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TypeStruct<Id> {
    pub name: String,
    pub rename: Option<String>,
    pub description: Option<String>,
    pub default: Option<JsonValue>,
    pub properties: Vec<StructProperty<Id>>,
    pub deny_unknown_fields: bool,
    //     pub schema: SchemaWrapper,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructProperty<Id> {
    pub name: String,
    pub rename: StructPropertyRename,
    pub state: StructPropertyState,
    pub description: Option<String>,
    pub type_id: Id,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StructPropertyRename {
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
