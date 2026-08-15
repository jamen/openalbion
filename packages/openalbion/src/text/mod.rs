//! Font rasterization and text layout — the CPU half of AGENTS.md §13.
//!
//! Same shape as [`crate::scene`], and for the same reason (§11.1): real input in, plain
//! renderer input out. A `TextureImage` of coverage and a list of screen-space quads is all the
//! renderer ever sees; it has never heard of a glyph outline, and `ab_glyph` is not in its
//! dependency list. That is also what makes every value here testable with no GPU and no Fable
//! install (§6.9).
//!
//! **Monospace, by scope** (§13). Layout is a pen that advances by one cell per character —
//! there is no shaping, no kerning and no bidi, because the console does not need them and
//! inventing them now would be a mechanism with nothing to check it against.

pub mod font;
pub mod layout;

pub use self::font::Font;
pub use self::layout::{layout_line, layout_lines};
