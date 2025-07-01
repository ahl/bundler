# Typify mk. 2

## Naming

There are several ways we might get a name for a type.

- User provided / indicated
- `title` field
- context from parent


## trait impls

What traits should a type implement? We should have a structured list that are
under consideration. For example `::serde::Serialize`, `::std::marker::Copy`,
and `::std::cmp::Ord`. We have some set by default. We let users opt out or
specify their own set. And they can add additional traits to derive.

For type replacements, users can (should?) specify which of the known (or
desired) traits the type implements.

Interesting idea: what if some traits are "as needed"? So for example, if we
have a schema like this:

```json
{
  "type": "array",
  "uniqueItems": true,
  "items": { "$ref": "#/$defs/Foo" }
}
```

We might say "`Foo` has to be `Ord` (or `Hash` + `Eq` depending on the set
type). But perhaps we *only* try to implement `Ord` for types that are used in
the context of a set. For trait propagation we would start with all the desired traits and then forward-propagate for implied-required traits and backward-poison for traits that cannot be satisfied (such as `Ord` for a `f64`).


## Extension notes

I've been thinking about the various ways we might want to annotate a schema.
I'm going to try to use this space to record them.

### `oneOf` → exclusive

For a coherent `enum` type, we would like all the variants to be mutually
exclusive in terms of serialization and deserialization. This is a constraint
beyond what Rust and `serde` enforce, but we believe it's an important one for
generative type modeling. In particular, consumers of generated types should
have exactly one way to express or consume a value. Imagine a given conceptual
value that could appear in more than one variant--this would be confusing and
hard to deal with!

The `oneOf` JSON schema construct validates if *exactly* one subschema
validates. This means that if the subschemas are *not* mutually exclusive then
a value at the intersection is actually not valid. As such, we need to generate
variants for each subschema that excludes all other subschemas. Unfortunately,
this isn't possible in the general case. Consider, for example, a schema like
this:

```json
{
	"oneOf": [
    {
      "type": "string",
      "format": "uuid"
    }, {
      "type": "string",
      "pattern": "^12345678(-1234)*-12345678$"
    }
  ]
}
```

Are these mutually exclusive? They are! I can tell that and one can imagine a
computer program that can tell that, but the general case with two regexes
is--as far as I can tell--intractably hard.

For that reason (and to accommodate imprecise schemas), we need a way to
indicate that all subschemas of a `oneOf` are, in fact, mutually exclusive. We
do this with a JSON schema extension `x-oneOfExclusive` in the same object as
the `oneOf`. It's valid values are `"known"` ("known to be mutually
exclusive"), `"override"` ("treat to be mutually exclusive even if
programmatically we can tell that they are not"), and `"unknown"` ("don't know
if they are mutually exclusive"--the default if no extension is present).

```json
{
	"oneOf": [
    {
      "type": "string",
      "format": "uuid"
    }, {
      "type": "string",
      "pattern": "^12345678(-1234)*-12345678$"
    }
  ],
  "x-oneOfExclusive": "known"
}
```

As with other extensions, we allow users to specify the default disposition.
For example, rather than patching each specific use of `oneOf` a user can
indicate that mutual-exclusivity is implicit for all such constructs.

