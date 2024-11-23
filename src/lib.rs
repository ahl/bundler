use std::collections::BTreeMap;

use append_map::AppendMap;
use url::Url;

/// How this works
///
/// 1. Read a file into a serde_json::Value
/// 2. Process anchor and dynamic anchors and the like
///    This builds up some sort of map from names to absolute paths
///    - do we want to look for references to other files here as well?
/// 3. Add known desired types into a queue
///    This might include just the root schema or also any defs
///    - Do we want to read in referenced other files before this step? I think
///      probably because we might want to add all those defs or referenced
///      schemas as well. On the other hand, random undiscovered refs could
///      pull in some new file so I wouldn't be assured that I had them all.
///    - Queue entries require a fully qualified path; they may also require
///      some additional context? Depends on how we handle dynamic refs.
type _XXX = ();

mod append_map;
mod bool_or;
mod bootstrap;
mod ir;
mod loader;

pub use loader::*;

/// TODO writing the description of this in the hope that it will help me find
/// the edges of what it is.
///
/// A Bundle is a collection of documents that have been indexed for reference
/// lookups. I think it will likely involve some sort of internal mutability,
/// more or less representing itself as a cache of documents. A bundle supports
/// heterogeneous schemas meaning that we keep the documents as untyped blob
/// data (serde_json::Value).
///
/// Operations
/// ----------
///
/// Load a document into the bundle. Each document is comprised of its blob
/// of data, caches values necessary for non-path ($anchor / $dynamicAnchor)
/// lookups, and an indication of the schema for the document (concretely,
/// either a JSON Schema draft or OpenAPI spec version). This last field is
/// necessary in order to properly interpret the results of a reference lookup.
/// Adding a "root" document (or maybe any document) needs to produce a Context
/// that lets us determine where we are in order to properly evaluate reference
/// lookups.
///
/// Lookup a reference. Given a context, a reference string, and a reference
/// type (lexical or dynamic), and a type T: Deserialize, return the
/// appropriate value, deserialized as the given type T. This would assume that
/// the caller knew the appropriate type T i.e. which $schema to assume. But we
/// could allow for multi-version, cross-schema references just returning a
/// blob along with the $schema value for the containing document.
///
pub struct Bundle {
    documents: AppendMap<String, Document>,
    loader: Box<dyn Loader>,
}

impl Default for Bundle {
    fn default() -> Self {
        Self {
            documents: Default::default(),
            loader: Box::new(loader::NullLoader),
        }
    }
}

#[derive(Debug)]
pub struct LoadError(pub String);

pub trait Loader {
    /// Return the canonical contents for the given URL.
    fn load(&self, url: Url) -> Result<String, LoadError>;
}

pub struct Document {
    pub id: String,
    pub content: serde_json::Value,
    pub schema: String,
    pub anchors: BTreeMap<String, String>,
    pub dyn_anchors: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Error;

pub struct Resolved<'a> {
    pub context: Context,
    pub value: &'a serde_json::Value,
    pub schema: &'a str,
}

impl Bundle {
    // TODO playing with the interface

    pub fn new(loader: impl Loader + 'static) -> Self {
        Self {
            documents: Default::default(),
            loader: Box::new(loader),
        }
    }

    /// Add explicit content (i.e. with no file lookup or web download).
    pub fn add_content(&mut self, content: impl AsRef<str>) -> Result<Context, Error> {
        // Turn the text into a JSON blob
        let value: serde_json::Value = serde_json::from_str(content.as_ref()).map_err(|_| Error)?;

        // Figure out the schema
        let schema = value.get("$schema");

        let document = match schema.and_then(serde_json::Value::as_str) {
            // TODO If there's no schema defined, we'll need to figure out some
            // sort of fallback position. We'll want some settings that let us
            // say things like "ignore $schema and use this" or "if there's no
            // schema, try whatever" or "if there's no schema only use this"
            None => todo!(),

            Some("https://json-schema.org/draft/2020-12/schema") => {
                bootstrap::Schema::make_document(value)
            }
            _ => todo!(),
        }?;

        let context = Context {
            id: document.id.clone(),
            dyn_anchors: document.dyn_anchors.clone(),
        };

        println!("adding {}", &document.id);
        self.documents.insert(document.id.clone(), document);

        Ok(context)
    }

    fn xxx_url(base: &str, reference: &str) -> (Url, String) {
        let base_url = url::Url::parse(base).unwrap();
        let mut ref_url = base_url.join(reference).unwrap();
        let fragment = ref_url.fragment().unwrap_or_default().to_string();
        ref_url.set_fragment(None);
        (ref_url, fragment)
    }

    /// Resolve a reference within the scope of the given context.
    pub fn resolve(
        &self,
        context: &Context,
        reference: impl AsRef<str>,
    ) -> Result<Resolved, Error> {
        let (id, fragment) = Self::xxx_url(&context.id, reference.as_ref());

        println!("resolving {} as {} {}", reference.as_ref(), id, fragment);

        let doc = if let Some(doc) = self.documents.get(id.as_str()) {
            doc
        } else {
            // TODO this is the interesting case. We need to somehow load up
            // the document and get it properly indexed.
            //
            // I think I want to have some kind of plug-in architecture where
            // we can choose between options such as mapping $id -> local files
            // and actually fetching files over the web (maybe with some
            // allow-list or cache or something?).
            //
            // We also need something pluggable dependent on the schema. ...
            // although, we're basically talking about two kinds of thing: JSON
            // Schema (in all its flavors) and OpenAPI (in its several
            // flavors). Could we just ... do those two things? Let's start
            // with that and not worry about plug-ins. At *least* let's start
            // with JSON Schema stuff built-in and then think about how to
            // handle OpenAPI.

            let contents = self.loader.load(id.clone()).unwrap();

            let doc = self.load_document(id.as_str(), &contents);

            doc
        };

        let value = &doc.content;

        let value = if fragment.starts_with('/') || fragment.is_empty() {
            value.pointer(&fragment)
        } else {
            let path = doc.anchors.get(&fragment).unwrap();
            // .ok_or(Error)?;
            value.pointer(path)
        }
        .unwrap();
        // .ok_or(Error)?;

        // The dynamic anchors of the incoming context *intentionally*
        // overwrite those of the document.
        let mut dyn_anchors = doc.dyn_anchors.clone();
        for (k, v) in &context.dyn_anchors {
            dyn_anchors.insert(k.clone(), v.clone());
        }

        let new_context = Context {
            id: id.to_string(),
            dyn_anchors,
        };

        let resolved = Resolved {
            context: new_context,
            value,
            schema: doc.schema.as_str(),
        };

        Ok(resolved)
    }

    pub fn load_document<'a>(&'a self, id: &str, contents: &str) -> &'a Document {
        let content: serde_json::Value =
            serde_json::from_str(contents).expect("couldn't parse into a Value");

        // We need to deduce the schema type from the document. In the case of
        // JSON Schema it might be easy if we see a `$schema` property. For
        // OpenAPI we could look at the `openapi` field. And if we don't find
        // those... I guess we'll just have to figure out something...
        // TODO later

        // TODO If there's no schema defined, we'll need to figure out some
        // sort of fallback position. We'll want some settings that let us
        // say things like "ignore $schema and use this" or "if there's no
        // schema, try whatever" or "if there's no schema only use this"

        let schema = if let Some(schema) = content.get("$schema") {
            schema
                .as_str()
                .expect("we should handle a non-string better")
                .to_string()
        } else {
            todo!("not sure of the schema type");
        };

        // TODO this is wrong; I only want to make this once I have the schema
        let mut document = Document {
            id: id.to_string(),
            content,
            anchors: Default::default(),
            dyn_anchors: Default::default(),
            schema: schema.clone(),
        };

        match schema.as_ref() {
            "https://json-schema.org/draft/2020-12/schema" => {
                bootstrap::Schema::populate_document(&mut document);
            }
            _ => todo!(),
        }

        self.documents.insert(id.to_string(), document)
    }
}

#[derive(Debug, Clone)]
pub struct Context {
    id: String,
    dyn_anchors: BTreeMap<String, String>,
}

pub fn to_generic(bundle: &Bundle, context: Context, value: &serde_json::Value, schema: &str) {
    match schema {
        "https://json-schema.org/draft/2020-12/schema" => {
            bootstrap::Schema::to_generic(bundle, context, value);
        }
        _ => todo!(),
    }
}

// TODO should this be fallible? Probably! What if it's a $schema I don't know?
// What if the serde fails?
pub fn to_ir(value: &serde_json::Value, schema: &str) -> ir::Schema {
    match schema {
        "https://json-schema.org/draft/2020-12/schema" => bootstrap::Schema::to_ir(value),
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use crate::Bundle;

    ///
    /// ideas
    /// 1. read in the top-level schema as RAW
    ///     RAW is schema draft specific
    /// 2. convert from RAW -> GENERIC
    ///     GENERIC get from all the various raw forms into some common form
    ///
    /// 3. GENERIC -> INTERNAL
    ///     INTERNAL, simpler and we manipulate it
    ///
    /// 4. Successive passes over INTERNAL to make CANONICAL
    ///     should be in a simple form
    ///     only constructions we like
    ///
    ///
    /// questions
    /// 1. when do we want to deal with dyn refs? I think **after** the
    ///    conversion into GENERIC
    #[test]
    fn xxx() {
        let id = "https://json-schema.org/draft/2020-12/schema";
        let contents = include_str!("../json-2020-12/schema");

        let bundle = Bundle::default();

        let _doc = bundle.load_document(id, contents);

        panic!();
    }
}
