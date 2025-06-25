use std::ops::Deref;

use crate::Bundle;

enum RefOrReal<'a, T> {
    Ref(&'a T),
    Real(T),
}

impl<T> Deref for RefOrReal<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            RefOrReal::Ref(value) => value,
            RefOrReal::Real(value) => value,
        }
    }
}

pub struct TypeSpaceSettings {}
pub struct TypeSpace<'a> {
    bundle: RefOrReal<'a, Bundle>,
}

impl TypeSpace<'_> {
    pub fn new(settings: TypeSpaceSettings) -> Self {
        Self {
            bundle: RefOrReal::Real(Bundle::default()),
        }
    }
}

impl<'a> TypeSpace<'a> {
    pub fn new_with_bundle(settings: TypeSpaceSettings, bundle: &'a Bundle) -> Self {
        Self {
            bundle: RefOrReal::Ref(bundle),
        }
    }
}

pub struct TypeId {}

pub enum Error {
    /// A schema could not be processed due to unrecognized constructions or
    /// ambiguous interpretation.
    InvalidSchema,
    /// The provided $id for a schema was malformed.
    InvalidSchemaId,
    /// The provided $id for a schema was inaccessible due to the configuration
    /// of the associated `Bundle` of schema data.
    InaccessibleSchemaId,
    /// The provided schema did not specify a value for `$schema`.
    NoSchemaSpec,
    /// The provided schema specified a value for `$schema` that does not
    /// correspond to a supported schema specification.
    UnknownSchemaSpec,
}

type Result<T> = std::result::Result<T, Error>;

impl<'a> TypeSpace<'a> {
    /// Add a new type (and any supporting types) by looking up the provided
    /// id in the associated `Bundle` of schema data. The applicable schema
    /// specification is determined by the value in the document named by the
    /// provided id; to override that value, use facilities of the `Bundle`.
    pub fn add_type_by_id(mut self, id: impl AsRef<str>) -> Result<TypeId> {
        todo!()
    }

    /// Add a new type (and any supporting types) from the provided schema. The
    /// schema must contain a property named `$schema` that is a string that
    /// identifies a known schema specification. To specify or override that
    /// value, insert the desired value of `$schema` into a
    /// `serde_json::Value`.
    pub fn add_type_by_schema(mut self, schema: &serde_json::Value) -> Result<TypeId> {
        todo!()
    }

    /// Add a new type (and any supporting types) from the provided schema and
    /// infer the appropriate schema specification based on the content. This
    /// first examines the value of the `$schema` property if it is present; if
    /// it is not, this interprets the data against known schema
    /// specifications, failing in the case of ambiguity.
    pub fn add_type_by_schema_infer_spec(mut self, schema: &serde_json::Value) -> Result<TypeId> {
        todo!()
    }
}
