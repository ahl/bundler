// TODO
// an array of types will turn into a oneOf
/// Canonical form
pub enum Canonical {
    // TODO reminder that these are going to be mutually exclusive
    ExclusiveOneOf(Vec<Canonical>),
    Object,
    Integer,
    Float,
}
