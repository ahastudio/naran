//! Colors. Naran is dark only.

use gpui::{Hsla, rgb};

/// A color from its red, green and blue bytes, as in `#16181d`.
pub fn color([red, green, blue]: [u8; 3]) -> Hsla {
    rgb(u32::from_be_bytes([0, red, green, blue])).into()
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub background: Hsla,
    pub sidebar: Hsla,
    pub surface: Hsla,
    pub hover: Hsla,
    pub selected: Hsla,
    pub border: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub accent: Hsla,
    pub problem: Hsla,
    pub attention: Hsla,
    pub calm: Hsla,
    pub added: Hsla,
    pub removed: Hsla,
}

impl Theme {
    /// Three depths: the group column is the darkest, the repository column
    /// sits on the base, and the detail column and cards are one step up.
    pub fn dark() -> Self {
        Self {
            background: color([0x16, 0x18, 0x1d]),
            sidebar: color([0x11, 0x13, 0x17]),
            surface: color([0x1d, 0x20, 0x27]),
            hover: color([0x24, 0x28, 0x32]),
            selected: color([0x26, 0x34, 0x4f]),
            border: color([0x26, 0x2a, 0x33]),
            text: color([0xe3, 0xe6, 0xec]),
            muted: color([0x86, 0x8c, 0x9a]),
            accent: color([0x7a, 0xa2, 0xf7]),
            problem: color([0xf7, 0x76, 0x8e]),
            attention: color([0xe0, 0xaf, 0x68]),
            calm: color([0x9e, 0xce, 0x6a]),
            added: color([0x9e, 0xce, 0x6a]),
            removed: color([0xf7, 0x76, 0x8e]),
        }
    }
}
