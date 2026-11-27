//! Title textures written out as DDS files, and replaced from DDS files
//! (a texture pack), by the content of the title's memory.
//!
//! * `VICTORY_TEXTURE_DUMP=<folder>`: every title texture detiled is written
//!   there once, as `<name>_<width>x<height>_f<format>.dds` (`name`: the
//!   first 16 hex digits of the BLAKE3 of the texture's tiled memory; the
//!   format is the host's DXGI number). Only 2D textures of one slice whose
//!   format is in [`block`], up to `VICTORY_TEXTURE_DUMP_MAX` texels a side
//!   (2048). The folder must be outside the repository (asset policy).
//! * `VICTORY_TEXTURE_REPLACE=<folder>`: a texture whose `<name>*.dds` is
//!   there takes that file's texels in place of the title's, if the file has
//!   the same format, size and levels as the dump of it had. The files there
//!   are listed once, at the first texture.
//!
//! Both are off unless set, and cost nothing then. With one of them on,
//! every detiled texture's memory is hashed.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// One level of the staging buffer: the host format, its size in texels,
/// its row pitch and rows (of blocks) in the buffer, and where it begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Level {
    pub(crate) format: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depth: u32,
    pub(crate) pitch: usize,
    pub(crate) rows: usize,
    pub(crate) offset: usize,
}

/// A DXGI format's block: texels a side, and bytes. `None`: not written.
fn block(format: u32) -> Option<(u32, usize)> {
    Some(match format {
        70..=72 | 79..=81 => (4, 8),                 // BC1, BC4
        73..=78 | 82..=84 | 94..=99 => (4, 16),      // BC2, BC3, BC5, BC6H, BC7
        27..=32 | 87..=93 | 23..=26 | 33..=43 => (1, 4), // 32 bits a texel
        9..=14 => (1, 8),                            // R16G16B16A16
        48..=59 => (1, 2),                           // R8G8, R16
        60..=65 => (1, 1),                           // R8, A8
        _ => return None,
    })
}

/// The bytes of a row of `level` without the pitch's padding, and its rows.
fn tight(level: &Level) -> Option<(usize, usize)> {
    let (side, bytes) = block(level.format)?;
    let across = level.width.max(1).div_ceil(side) as usize;
    let down = level.height.max(1).div_ceil(side) as usize;
    (across * bytes <= level.pitch && down <= level.rows).then_some((across * bytes, down))
}

const HEADER: usize = 4 + 124 + 20;

/// A DDS file (DX10 header) of `levels`, the levels of one 2D texture from
/// the largest on, read from `data`. `None` if it is not such a texture.
pub(crate) fn dds(levels: &[Level], data: &[u8]) -> Option<Vec<u8>> {
    let first = levels.first()?;
    if levels.iter().any(|l| l.format != first.format || l.depth != 1) {
        return None;
    }
    let mut out = Vec::with_capacity(HEADER);
    let word = |out: &mut Vec<u8>, v: u32| out.extend_from_slice(&v.to_le_bytes());
    out.extend_from_slice(b"DDS ");
    word(&mut out, 124);
    // caps, height, width, pixel format, mip count, linear size
    word(&mut out, 0x1 | 0x2 | 0x4 | 0x1000 | 0x2_0000 | 0x8_0000);
    word(&mut out, first.height);
    word(&mut out, first.width);
    let (row, rows) = tight(first)?;
    word(&mut out, (row * rows) as u32);
    word(&mut out, 0);
    word(&mut out, levels.len() as u32);
    for _ in 0..11 {
        word(&mut out, 0);
    }
    word(&mut out, 32);
    word(&mut out, 0x4); // a four-character code
    out.extend_from_slice(b"DX10");
    for _ in 0..5 {
        word(&mut out, 0);
    }
    // texture, and with more than one level: complex, mipmap
    word(&mut out, if levels.len() > 1 { 0x1000 | 0x8 | 0x40_0000 } else { 0x1000 });
    for _ in 0..4 {
        word(&mut out, 0);
    }
    word(&mut out, first.format);
    word(&mut out, 3); // a 2D texture
    word(&mut out, 0);
    word(&mut out, 1);
    word(&mut out, 0);
    debug_assert_eq!(out.len(), HEADER);
    for level in levels {
        let (row, rows) = tight(level)?;
        for y in 0..rows {
            let at = level.offset + y * level.pitch;
            out.extend_from_slice(data.get(at..at + row)?);
        }
    }
    Some(out)
}

/// Puts the texels of `file` (as [`dds`] writes one) into `data`. Nothing
/// is written unless the file is of the same format, size and levels.
pub(crate) fn apply(file: &[u8], levels: &[Level], data: &mut [u8]) -> bool {
    let word = |at: usize| file.get(at..at + 4).map(|b| u32::from_le_bytes(b.try_into().unwrap()));
    let Some(first) = levels.first() else { return false };
    let same = file.len() > HEADER
        && &file[..4] == b"DDS "
        && &file[84..88] == b"DX10"
        && word(12) == Some(first.height)
        && word(16) == Some(first.width)
        && word(28).map(|m| m.max(1)) == Some(levels.len() as u32)
        && word(128) == Some(first.format)
        && levels.iter().all(|l| l.format == first.format && l.depth == 1);
    if !same {
        return false;
    }
    // Checked whole before a byte is written.
    let mut spans = Vec::new();
    let mut from = HEADER;
    for level in levels {
        let Some((row, rows)) = tight(level) else { return false };
        for y in 0..rows {
            let at = level.offset + y * level.pitch;
            if from + row > file.len() || at + row > data.len() {
                return false;
            }
            spans.push((from, at, row));
            from += row;
        }
    }
    for (from, at, row) in spans {
        data[at..at + row].copy_from_slice(&file[from..from + row]);
    }
    true
}

fn folder(variable: &str) -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os(variable).filter(|v| !v.is_empty())?);
    if let Err(e) = victory_core::ensure_outside_repo(&path) {
        tracing::warn!(variable, error = %e, "texture folder inside the repository: left out");
        return None;
    }
    if let Err(e) = std::fs::create_dir_all(&path) {
        tracing::warn!(variable, path = %path.display(), error = %e, "texture folder cannot be made");
        return None;
    }
    Some(path)
}

fn dump_to() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        let dir = folder("VICTORY_TEXTURE_DUMP");
        if let Some(dir) = &dir {
            tracing::info!(path = %dir.display(), "title textures are written out (VICTORY_TEXTURE_DUMP)");
        }
        dir
    })
    .as_deref()
}

/// The replacement files, by name (the 16 hex digits their file name begins with).
fn replacements() -> &'static HashMap<String, PathBuf> {
    static FILES: OnceLock<HashMap<String, PathBuf>> = OnceLock::new();
    FILES.get_or_init(|| {
        let mut files = HashMap::new();
        if let Some(dir) = folder("VICTORY_TEXTURE_REPLACE") {
            for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if name.ends_with(".dds") && name.len() >= 16 && name.is_char_boundary(16) {
                    files.insert(name[..16].to_owned(), path);
                }
            }
            tracing::info!(path = %dir.display(), files = files.len(), "title textures are replaced (VICTORY_TEXTURE_REPLACE)");
        }
        files
    })
}

/// Whether either switch is set: the caller gathers levels only then.
pub(crate) fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| dump_to().is_some() || !replacements().is_empty())
}

fn most() -> u32 {
    static MOST: OnceLock<u32> = OnceLock::new();
    *MOST.get_or_init(|| {
        std::env::var("VICTORY_TEXTURE_DUMP_MAX").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(2048)
    })
}

/// After a texture was detiled from `tiled` into `data` (its staging
/// buffer, laid out as `levels`): replaced, or written out.
pub(crate) fn detiled(tiled: &[u8], levels: &[Level], data: &mut [u8]) {
    let Some(first) = levels.first() else { return };
    let name = blake3::hash(tiled).to_hex()[..16].to_owned();
    if let Some(path) = replacements().get(&name) {
        match std::fs::read(path) {
            Ok(file) if apply(&file, levels, data) => {
                static SAID: Mutex<Option<HashSet<String>>> = Mutex::new(None);
                let mut said = SAID.lock().unwrap_or_else(|e| e.into_inner());
                if said.get_or_insert_with(HashSet::new).insert(name.clone()) {
                    tracing::info!(name, "a title texture replaced");
                }
                return;
            }
            Ok(_) => tracing::warn!(name, width = first.width, height = first.height, format = first.format,
                levels = levels.len(), "a replacement texture of another format, size or level count is left out"),
            Err(e) => tracing::warn!(name, error = %e, "a replacement texture cannot be read"),
        }
    }
    let Some(dir) = dump_to() else { return };
    if first.width > most() || first.height > most() {
        return;
    }
    static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
    if !SEEN.lock().unwrap_or_else(|e| e.into_inner()).get_or_insert_with(HashSet::new).insert(name.clone()) {
        return;
    }
    let Some(file) = dds(levels, data) else { return };
    let path = dir.join(format!("{name}_{}x{}_f{}.dds", first.width, first.height, first.format));
    if let Err(e) = std::fs::write(&path, file) {
        tracing::warn!(path = %path.display(), error = %e, "a texture is not written out");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels() -> ([Level; 2], Vec<u8>) {
        // BC3 8x8 with a 4x4 level under it, pitch 256.
        let levels = [
            Level { format: 77, width: 8, height: 8, depth: 1, pitch: 256, rows: 2, offset: 0 },
            Level { format: 77, width: 4, height: 4, depth: 1, pitch: 256, rows: 1, offset: 512 },
        ];
        let data: Vec<u8> = (0..768u32).map(|i| (i * 7 % 251) as u8).collect();
        (levels, data)
    }

    #[test]
    fn a_dump_holds_the_rows_without_their_padding() {
        let (levels, data) = levels();
        let file = dds(&levels, &data).unwrap();
        assert_eq!(file.len(), HEADER + 32 + 32 + 16);
        assert_eq!(&file[..4], b"DDS ");
        assert_eq!(&file[HEADER..HEADER + 32], &data[..32]);
        assert_eq!(&file[HEADER + 32..HEADER + 64], &data[256..288]);
        assert_eq!(&file[HEADER + 64..], &data[512..528]);
    }

    #[test]
    fn a_dump_put_back_gives_the_same_texels_and_leaves_the_padding() {
        let (levels, data) = levels();
        let file = dds(&levels, &data).unwrap();
        let mut other = vec![0xEEu8; data.len()];
        assert!(apply(&file, &levels, &mut other));
        assert_eq!(&other[..32], &data[..32]);
        assert_eq!(&other[256..288], &data[256..288]);
        assert_eq!(&other[512..528], &data[512..528]);
        assert!(other[32..256].iter().all(|&b| b == 0xEE));
        assert_eq!(dds(&levels, &other).unwrap(), file);
    }

    #[test]
    fn a_file_of_another_shape_changes_nothing() {
        let (levels, data) = levels();
        let file = dds(&levels, &data).unwrap();
        let mut other = vec![0u8; data.len()];
        let mut wider = levels;
        wider[0].width = 12;
        assert!(!apply(&file, &wider, &mut other));
        let mut format = levels;
        format[0].format = 98;
        format[1].format = 98;
        assert!(!apply(&file, &format, &mut other));
        assert!(!apply(&file, &levels[..1], &mut other));
        assert!(!apply(&file[..file.len() - 1], &levels, &mut other));
        assert!(!apply(b"not a file", &levels, &mut other));
        assert!(other.iter().all(|&b| b == 0));
    }

    #[test]
    fn formats_not_known_and_volumes_are_not_written() {
        let (mut levels, data) = levels();
        levels[1].depth = 2;
        assert_eq!(dds(&levels, &data), None);
        let odd = [Level { format: 1000, width: 4, height: 4, depth: 1, pitch: 256, rows: 4, offset: 0 }];
        assert_eq!(dds(&odd, &data), None);
        let rgba = [Level { format: 28, width: 3, height: 2, depth: 1, pitch: 256, rows: 2, offset: 0 }];
        assert_eq!(dds(&rgba, &data).unwrap().len(), HEADER + 24);
    }
}
