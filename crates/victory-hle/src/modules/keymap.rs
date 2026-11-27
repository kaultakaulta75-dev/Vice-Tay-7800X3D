//! Which key a stand-in gamepad control uses, whatever the keyboard layout.
//!
//! The stand-in pad is written for a QWERTY keyboard (W/A/S/D move, Q/E bumpers...). On an AZERTY
//! or QWERTZ keyboard the same virtual keys sit elsewhere, so the controls ended up in odd places.
//! `VICTORY_KEYBOARD` picks the mapping:
//!
//! * `scancode` (default): by physical position. The key where QWERTY has W is the up key, so ZQSD
//!   works on AZERTY with no setting. Windows says which virtual key that position is on the
//!   active layout (`MapVirtualKey`).
//! * `qwerty`: the virtual keys as written.
//! * `azerty`: the letters moved to where an AZERTY keyboard has them (Z up, Q left, A bumper...).
//! * `qwertz`: Y and Z swapped.

/// The keyboard mapping of the stand-in pad.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    Scancode,
    Qwerty,
    Azerty,
    Qwertz,
}

impl Layout {
    /// Reads the value of `VICTORY_KEYBOARD`; anything unknown is the default (`scancode`).
    pub fn parse(text: &str) -> Layout {
        match text.trim().to_ascii_lowercase().as_str() {
            "qwerty" => Layout::Qwerty,
            "azerty" => Layout::Azerty,
            "qwertz" => Layout::Qwertz,
            _ => Layout::Scancode,
        }
    }
}

/// Set-1 scan codes of the letter keys, by the QWERTY letter printed at that position.
const QWERTY_SCANCODES: [(u8, u32); 26] = [
    (b'Q', 0x10),
    (b'W', 0x11),
    (b'E', 0x12),
    (b'R', 0x13),
    (b'T', 0x14),
    (b'Y', 0x15),
    (b'U', 0x16),
    (b'I', 0x17),
    (b'O', 0x18),
    (b'P', 0x19),
    (b'A', 0x1E),
    (b'S', 0x1F),
    (b'D', 0x20),
    (b'F', 0x21),
    (b'G', 0x22),
    (b'H', 0x23),
    (b'J', 0x24),
    (b'K', 0x25),
    (b'L', 0x26),
    (b'Z', 0x2C),
    (b'X', 0x2D),
    (b'C', 0x2E),
    (b'V', 0x2F),
    (b'B', 0x30),
    (b'N', 0x31),
    (b'M', 0x32),
];

/// The scan code of the key printed `letter` on a QWERTY keyboard.
pub fn scancode_of(letter: u8) -> Option<u32> {
    QWERTY_SCANCODES
        .iter()
        .find(|(l, _)| *l == letter)
        .map(|(_, code)| *code)
}

/// The virtual key of the control written as QWERTY `logical` under `layout`. `from_scancode` asks
/// the system which virtual key a scan code is (`None` or 0 when it does not know). Keys that are
/// not letters (Enter, arrows...) are the same on every layout.
pub fn vk_for(layout: Layout, logical: u8, from_scancode: impl Fn(u32) -> Option<u8>) -> u8 {
    if !logical.is_ascii_uppercase() {
        return logical;
    }
    match layout {
        Layout::Qwerty => logical,
        Layout::Azerty => match logical {
            b'Q' => b'A',
            b'A' => b'Q',
            b'W' => b'Z',
            b'Z' => b'W',
            other => other,
        },
        Layout::Qwertz => match logical {
            b'Y' => b'Z',
            b'Z' => b'Y',
            other => other,
        },
        Layout::Scancode => scancode_of(logical)
            .and_then(from_scancode)
            .filter(|vk| *vk != 0)
            .unwrap_or(logical),
    }
}

/// The layout the process runs with. Tests always see QWERTY, so they do not depend on the
/// keyboard of the machine that runs them.
pub fn current_layout() -> Layout {
    #[cfg(test)]
    {
        Layout::Qwerty
    }
    #[cfg(not(test))]
    {
        static LAYOUT: std::sync::OnceLock<Layout> = std::sync::OnceLock::new();
        *LAYOUT.get_or_init(|| {
            let layout = std::env::var("VICTORY_KEYBOARD")
                .map(|v| Layout::parse(&v))
                .unwrap_or(Layout::Scancode);
            tracing::info!(?layout, "keyboard mapping of the stand-in pad");
            layout
        })
    }
}

/// The virtual key to look for, now, for the control written as QWERTY `logical`.
pub fn resolve(logical: u8) -> u8 {
    vk_for(current_layout(), logical, system_vk)
}

#[cfg(all(windows, not(test)))]
fn system_vk(scancode: u32) -> Option<u8> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{MAPVK_VSC_TO_VK, MapVirtualKeyW};
    // SAFETY: a plain query of the active keyboard layout.
    let vk = unsafe { MapVirtualKeyW(scancode, MAPVK_VSC_TO_VK) };
    u8::try_from(vk).ok()
}

#[cfg(any(not(windows), test))]
fn system_vk(_scancode: u32) -> Option<u8> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What an AZERTY system answers for the scan codes of the keys moved by that layout.
    fn azerty_system(scancode: u32) -> Option<u8> {
        match scancode {
            0x10 => Some(b'A'),
            0x11 => Some(b'Z'),
            0x1E => Some(b'Q'),
            0x2C => Some(b'W'),
            other => QWERTY_SCANCODES
                .iter()
                .find(|(_, c)| *c == other)
                .map(|(l, _)| *l),
        }
    }

    #[test]
    fn the_names_of_the_layouts_are_read() {
        assert_eq!(Layout::parse("AZERTY"), Layout::Azerty);
        assert_eq!(Layout::parse(" qwertz "), Layout::Qwertz);
        assert_eq!(Layout::parse("qwerty"), Layout::Qwerty);
        assert_eq!(Layout::parse("scancode"), Layout::Scancode);
        assert_eq!(Layout::parse("dvorak?"), Layout::Scancode);
        assert_eq!(Layout::parse(""), Layout::Scancode);
    }

    #[test]
    fn scancodes_are_unique_and_known_for_every_letter() {
        let mut codes: Vec<u32> = QWERTY_SCANCODES.iter().map(|(_, c)| *c).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 26);
        assert_eq!(scancode_of(b'W'), Some(0x11));
        assert_eq!(scancode_of(b'1'), None);
    }

    #[test]
    fn qwerty_keeps_the_keys_as_written() {
        for letter in b'A'..=b'Z' {
            assert_eq!(vk_for(Layout::Qwerty, letter, |_| None), letter);
        }
    }

    #[test]
    fn azerty_moves_zqsd_and_the_bumper() {
        let none = |_| None;
        assert_eq!(vk_for(Layout::Azerty, b'W', none), b'Z');
        assert_eq!(vk_for(Layout::Azerty, b'A', none), b'Q');
        assert_eq!(vk_for(Layout::Azerty, b'S', none), b'S');
        assert_eq!(vk_for(Layout::Azerty, b'D', none), b'D');
        assert_eq!(vk_for(Layout::Azerty, b'Q', none), b'A');
        assert_eq!(vk_for(Layout::Azerty, b'Z', none), b'W');
    }

    #[test]
    fn qwertz_swaps_y_and_z() {
        let none = |_| None;
        assert_eq!(vk_for(Layout::Qwertz, b'Y', none), b'Z');
        assert_eq!(vk_for(Layout::Qwertz, b'Z', none), b'Y');
        assert_eq!(vk_for(Layout::Qwertz, b'W', none), b'W');
    }

    #[test]
    fn scancode_mode_follows_the_active_layout() {
        assert_eq!(vk_for(Layout::Scancode, b'W', azerty_system), b'Z');
        assert_eq!(vk_for(Layout::Scancode, b'A', azerty_system), b'Q');
        assert_eq!(vk_for(Layout::Scancode, b'S', azerty_system), b'S');
        assert_eq!(vk_for(Layout::Scancode, b'D', azerty_system), b'D');
        // On a QWERTY system it is the identity.
        let qwerty = |code: u32| {
            QWERTY_SCANCODES
                .iter()
                .find(|(_, c)| *c == code)
                .map(|(l, _)| *l)
        };
        for letter in b'A'..=b'Z' {
            assert_eq!(vk_for(Layout::Scancode, letter, qwerty), letter);
        }
    }

    #[test]
    fn scancode_mode_falls_back_when_the_system_does_not_know() {
        assert_eq!(vk_for(Layout::Scancode, b'W', |_| None), b'W');
        assert_eq!(vk_for(Layout::Scancode, b'W', |_| Some(0)), b'W');
    }

    #[test]
    fn keys_that_are_not_letters_never_move() {
        for vk in [0x0D_u8, 0x08, 0x1B, 0x09, 0x25, 0x26, 0x27, 0x28, b'1'] {
            for layout in [Layout::Scancode, Layout::Qwerty, Layout::Azerty, Layout::Qwertz] {
                assert_eq!(vk_for(layout, vk, |_| Some(b'Z')), vk);
            }
        }
    }
}
