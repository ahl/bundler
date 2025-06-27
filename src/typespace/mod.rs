mod type_common;
mod type_enum;
mod type_struct;

pub use type_common::*;
pub use type_enum::*;
pub use type_struct::*;

use std::collections::BTreeMap;

// 6/25/2025
// I think I need a builder form e.g. of an enum or struct and then the
// finalized form which probably is basically what typify shows today in its
// public interface.

pub struct TypespaceBuilder<Id> {
    types: BTreeMap<Id, Type<Id>>,
}

impl<Id> Default for TypespaceBuilder<Id> {
    fn default() -> Self {
        Self {
            types: Default::default(),
        }
    }
}

pub struct Typespace<Id> {
    types: BTreeMap<Id, Type<Id>>,
}

impl<Id> TypespaceBuilder<Id> {
    pub fn insert(&mut self, id: Id, typ: Type<Id>)
    where
        Id: Ord,
    {
        // TODO do some validation of types, e.g. that variant names are
        // unique.
        self.types.insert(id, typ);
    }

    pub fn finalize(self) -> Result<Typespace<Id>, ()> {
        let Self { types } = self;

        // TODO Make sure that all referenced schemas are present.
        // TODO break cycles
        // TODO resolve names
        // TODO propagate trait impls

        todo!()
    }
}

pub enum Type<Id> {
    Enum(TypeEnum<Id>),
    Struct(TypeStruct<Id>),
}
