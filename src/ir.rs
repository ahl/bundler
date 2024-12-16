use std::collections::{BTreeMap, BTreeSet};

// What does a canonical schema look like? I think at the top level it should
// be a mutually exclusive oneOf (potentially of cardinality 1). Each variant
// of that array should be (basically) self-contained.

/// TODO need to figure out what this is; probably some doc + path
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SchemaRef {
    Where(String),
    Partial(String, &'static str),
}

#[derive(Debug)]
pub struct Schema {
    pub metadata: SchemaMetadata,
    pub details: SchemaDetails,
}

#[derive(Debug, Default)]
pub struct SchemaMetadata {
    pub id: Option<String>,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct SchemaDetailsObject {
    pub properties: BTreeMap<String, SchemaRef>,
    pub additional_properties: Option<SchemaRef>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SchemaDetailsArray {
    pub items: Option<SchemaRef>,
    pub min_items: Option<u64>,
    pub unique_items: bool,
}

#[derive(Debug, Clone)]
pub enum SchemaDetailsValue {
    Any,
    Object(SchemaDetailsObject),
    Array(SchemaDetailsArray),
    Boolean,
    String,
    Number,
    Integer,
}

// #[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum SchemaDetails {
    Nothing,
    Anything,
    Value(SchemaDetailsValue),
    AllOf(Vec<SchemaRef>),
    AnyOf(Vec<SchemaRef>),
    ExclusiveOneOf(Vec<SchemaRef>),
    DollarRef(String),
    DynamicRef(String),
    Constant(serde_json::Value),
}
#[derive(Clone, Debug)]
pub enum Type {
    Single(SimpleType),
    Array(BTreeSet<SimpleType>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimpleType {
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}

impl Schema {
    // pub fn list_subschema_refs(&self) -> Vec<String> {
    //     let mut out = Vec::new();
    //     self.list_subschema_refs_impl(&mut out);
    //     out
    // }

    // fn list_subschema_refs_impl(&self, out: &mut Vec<String>) {
    //     let SchemaDetails::Any {
    //         r#ref,
    //         all_of,
    //         any_of,
    //         one_of,
    //         ..
    //     } = &self.details
    //     else {
    //         panic!("this should only be called on raw schemas")
    //     };

    //     if let Some(r) = r#ref {
    //         out.push(r.clone());
    //     }

    //     all_of
    //         .iter()
    //         .chain(any_of)
    //         .chain(one_of)
    //         .flatten()
    //         .for_each(|subschema| match subschema {
    //             ObjectOrBool::Bool(_) => (),
    //             ObjectOrBool::Object(ss) => ss.list_subschema_refs_impl(out),
    //         });
    // }
}

// Random thoughts:
// - If we resolve to schemas and then it turns out to just be one or the other
//   we'll want the result to just turn into one or the other e.g. as the
//   unresolved reference
pub fn merge_two(a: &Schema, b: &Schema) -> Schema {
    let details = match (&a.details, &b.details) {
        // Nothing will come of nothing
        (SchemaDetails::Nothing, _) => SchemaDetails::Nothing,
        (_, SchemaDetails::Nothing) => SchemaDetails::Nothing,

        // Of one is permissive, use the other
        (SchemaDetails::Anything, other) => other.clone(),
        (other, SchemaDetails::Anything) => other.clone(),

        (
            SchemaDetails::Value(SchemaDetailsValue::Object(_aa)),
            SchemaDetails::Value(SchemaDetailsValue::Object(_bb)),
        ) => {
            // TODO this is tricky
            // Let's say we have two properties with the same name. We'll need
            // to--effectively--recursively merge those. So we either need to
            // actually do that merge recursively, or we need to produce some
            // intermediate IR whose value for the property is some new,
            // synthetic, IR that's an all_of[aaa, bbb]. That all seems ok, but
            // if the result of the deep merge is that we end up with one of
            // the input schemas (i.e. merging leaves us with precisely one
            // of the inputs), we'd like to figure that out and memoize that
            // work.
            //
            // We've taken apart schemas to forge the IRs; maybe it would be ok
            // to start to reassemble them as we process them into a canonical
            // form.
            //
            // I think I can punt on all of this for the bootstrapping process
            // as I don't think any of the properties in the bootstrap schema
            // actually conflict. The major challenge in here is going to be
            // the dynamic references (I think?).

            // TODO how to know if the result should be just a or b?
            // I don't think there's a better way to tell other than simply
            // performing the full merge and then checking at the end.

            todo!()
        }

        _ => todo!(),
    };
    todo!()
}

// pub fn merge_all(schemas: &[&Schema]) -> Schema {
//     match schemas {
//         [] => todo!(),
//         [solo] => todo!(),
//         [first, rest @ ..] => todo!(),
//     }
// }
