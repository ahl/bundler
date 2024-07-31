/// Canonical form
/// TODO list attributes
pub enum Canonical {
    OneOf(Vec<Canonical>),
    Object,
    Integer,
    Float,
}
