// Shared card identities used by the game and standalone layout preview.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Kind { Number(u8, u8), Wild, Skip }
#[derive(Clone, Copy, Debug)]
pub(crate) struct Card { pub(crate) id: u32, pub(crate) kind: Kind }
