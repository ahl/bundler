use std::collections::BTreeMap;

use crate::{
    schemalet::{CanonicalSchemalet, SchemaRef, Schemalet},
    typespace::TypespaceBuilder,
    Bundle,
};

pub struct Typify {
    bundle: Bundle,
    normalizer: Normalizer,
    typespace: TypespaceBuilder,
}

#[derive(Default)]
struct Normalizer {
    raw: BTreeMap<SchemaRef, Schemalet>,
    canonical: BTreeMap<SchemaRef, CanonicalSchemalet>,
}
#[derive(Debug)]
pub enum Error {
    X,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct TypeId(SchemaRef);

impl Typify {
    pub fn new_with_bundle(bundle: Bundle) -> Self {
        Self {
            bundle,
            normalizer: Default::default(),
            typespace: Default::default(),
        }
    }

    /// Add a new type (and any supporting types) by looking up the provided
    /// id in the associated `Bundle` of schema data. The applicable schema
    /// specification is determined by the value in the document named by the
    /// provided id; to override that value, use facilities of the `Bundle`.
    pub fn add_type_by_id(&mut self, id: impl AsRef<str>) -> Result<TypeId> {
        self.normalizer.add(&self.bundle, id).unwrap();

        todo!()
    }
}

impl Normalizer {
    fn add(&self, bundle: &Bundle, id: impl AsRef<str>) -> Result<()> {
        todo!()
    }
}
