/// The generic schema; all specific schemas are responsible for converting
/// themselves into this structure. We want this conversion to be as simple as
/// possible, saving any actual manipulation for the conversion from
/// generic::Schema to ir::Schema.
pub struct Schema {}
