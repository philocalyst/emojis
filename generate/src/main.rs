mod github;
mod unicode;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::Result;

use crate::unicode::SkinTone;

fn write_group_enum<W: io::Write>(w: &mut W, unicode_data: &unicode::ParsedData) -> Result<()> {
    writeln!(w, "/// A category for an emoji.")?;
    writeln!(w, "///")?;
    writeln!(w, "/// Based on Unicode CLDR data.")?;
    writeln!(
        w,
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum, PartialOrd, Ord)]"
    )?;
    writeln!(
        w,
        r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
    )?;
    writeln!(w, "pub enum Group {{")?;
    for name in unicode_data.keys() {
        if name == "Component" {
            continue;
        }
        writeln!(w, "   {name},")?;
    }
    writeln!(w, "}}")?;
    Ok(())
}

fn write_skin_tone_data_struct<W: io::Write>(w: &mut W) -> Result<()> {
    writeln!(w, "/// Data about skin tone variants for an emoji.")?;
    writeln!(
        w,
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]"
    )?;
    writeln!(
        w,
        r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#,
    )?;
    writeln!(w, "pub struct SkinToneData {{")?;
    writeln!(
        w,
        "    /// Index of the first skin tone variant in the EMOJIS array"
    )?;
    writeln!(w, "    pub first: usize,")?;
    writeln!(w, "    /// Number of skin tone variants")?;
    writeln!(w, "    pub second: usize,")?;
    writeln!(w, "    /// The skin tone of this emoji")?;
    writeln!(w, "    pub tone: SkinTone,")?;
    writeln!(w, "}}")?;
    Ok(())
}

fn write_build_emojis_function<W: io::Write>(
    w: &mut W,
    unicode_data: &unicode::ParsedData,
    github_data: &github::ParsedData,
    unicode_map: &mut HashMap<String, String>,
    shortcode_map: &mut HashMap<String, String>,
) -> Result<()> {
    let mut i = 0;
    let mut default_skin_tone_index = 0;
    let mut skin_tone_count = 0;

    writeln!(w, "/// Build the static emoji data at program startup")?;
    writeln!(w, "fn build_emojis() -> Vec<Emoji> {{")?;
    writeln!(w, "    vec![")?;
    for (group, subgroups) in unicode_data {
        for subgroup in subgroups.values() {
            for emoji in subgroup {
                if matches!(emoji.skin_tone(), Some(SkinTone::Default)) {
                    default_skin_tone_index = i;
                    skin_tone_count = emoji.skin_tones();
                }
                write!(w, "        ")?;

                // Build the emoji struct
                let e = emoji.as_str();
                let name = emoji.name();
                let uv = emoji.unicode_version();
                write!(
                    w,
                    r#"Emoji {{
            emoji: "{e}".to_string(),
            name: "{name}".to_string(),
            unicode_version: {uv:?},
            group: Group::{group}"#,
                )?;

                match emoji.skin_tone() {
                    Some(tone) => write!(
                        w,
                        r#",
            skin_tone: Some(SkinToneData {{
                first: {default_skin_tone_index},
                second: {skin_tone_count},
                tone: SkinTone::{tone:?}
            }})"#,
                    )?,
                    None => write!(w, ",\n            skin_tone: None")?,
                }

                match &github_data.get(e) {
                    Some(github) => {
                        let aliases = github.aliases();
                        if aliases.is_empty() {
                            write!(w, ",\n            aliases: None")?;
                        } else {
                            write!(w, ",\n            aliases: Some(vec![")?;
                            for (i, alias) in aliases.iter().enumerate() {
                                write!(w, "\"{}\".to_string()", alias)?;
                                if i < aliases.len() - 1 {
                                    write!(w, ", ")?;
                                }
                            }
                            write!(w, "])")?;
                        }
                    }
                    None => write!(w, ",\n            aliases: None")?,
                }

                write!(w, "\n        }}")?;
                writeln!(w, ",")?;

                unicode_map.insert(emoji.as_str().to_owned(), i.to_string());
                for v in emoji.variations() {
                    assert!(unicode_map.insert(v.to_owned(), i.to_string()).is_none());
                }

                if let Some(github) = &github_data.get(emoji.as_str()) {
                    for alias in github.aliases() {
                        assert!(shortcode_map
                            .insert(alias.to_owned(), i.to_string())
                            .is_none());
                    }
                }
                i += 1;
            }
        }
    }
    writeln!(w, "    ]")?;
    writeln!(w, "}}")?;
    Ok(())
}

fn write_access_functions<W: io::Write>(w: &mut W) -> Result<()> {
    writeln!(w, "/// Get all available emojis")?;
    writeln!(w, "pub fn all_emojis() -> &'static [Emoji] {{")?;
    writeln!(w, "    &EMOJIS")?;
    writeln!(w, "}}\n")?;

    writeln!(w, "/// Get an emoji by its unicode representation")?;
    writeln!(
        w,
        "pub fn emoji_from_unicode(unicode: &str) -> Option<&'static Emoji> {{"
    )?;
    writeln!(w, "    unicode::MAP.get(unicode).map(|&idx| &EMOJIS[idx])")?;
    writeln!(w, "}}\n")?;

    writeln!(w, "/// Get an emoji by its shortcode")?;
    writeln!(
        w,
        "pub fn emoji_from_shortcode(shortcode: &str) -> Option<&'static Emoji> {{"
    )?;
    writeln!(
        w,
        "    shortcode::MAP.get(shortcode).map(|&idx| &EMOJIS[idx])"
    )?;
    writeln!(w, "}}\n")?;
    Ok(())
}

fn write_phf_map<W: io::Write>(w: &mut W, map: HashMap<String, String>) -> Result<()> {
    write!(w, "pub static MAP: ::phf::Map<&'static str, usize> = ")?;
    let mut gen = phf_codegen::Map::new();
    for (key, value) in &map {
        gen.entry(key, &value);
    }
    writeln!(w, "{};", gen.build())?;
    Ok(())
}

const HEADER: &str = "// Code generated by `cargo run --package generate`. DO NOT EDIT.\n";

fn main() -> Result<()> {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "src", "gen"]
        .iter()
        .collect();

    let unicode_data = unicode::fetch_and_parse_emoji_data()?;
    let github_data = github::fetch_and_parse_emoji_data()?;
    let mut unicode_map = HashMap::new();
    let mut shortcode_map = HashMap::new();

    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir)?;

    let mut f = fs::File::create(dir.join("mod.rs"))?;
    writeln!(f, "{}", HEADER)?;
    writeln!(f, "#![cfg_attr(rustfmt, rustfmt::skip)]\n")?;
    writeln!(f, "pub mod shortcode;")?;
    writeln!(f, "pub mod unicode;\n")?;
    writeln!(f, "use once_cell::sync::Lazy;")?;
    writeln!(
        f,
        "use crate::{{Emoji, SkinTone, SkinToneData, UnicodeVersion, Group}};\n"
    )?;

    // Write the build_emojis function
    write_build_emojis_function(
        &mut f,
        &unicode_data,
        &github_data,
        &mut unicode_map,
        &mut shortcode_map,
    )?;
    writeln!(f)?;

    // Write static initialization
    writeln!(f, "/// Static instance of all emojis")?;
    writeln!(
        f,
        "pub static EMOJIS: Lazy<Vec<Emoji>> = Lazy::new(build_emojis);\n"
    )?;

    // Write access functions
    write_access_functions(&mut f)?;

    // Write emoji lookup tables
    let mut f = fs::File::create(dir.join("unicode.rs"))?;
    writeln!(f, "{}", HEADER)?;
    write_phf_map(&mut f, unicode_map)?;

    let mut f = fs::File::create(dir.join("shortcode.rs"))?;
    writeln!(f, "{}", HEADER)?;
    write_phf_map(&mut f, shortcode_map)?;

    // Create a main emoji.rs file that exports everything
    let mut f = fs::File::create(dir.join("../emoji.rs"))?;
    writeln!(f, "{}", HEADER)?;
    writeln!(f, "//! Emoji processing functionality.")?;
    writeln!(f, "mod gen;\n")?;
    writeln!(
        f,
        "pub use gen::{{all_emojis, emoji_from_unicode, emoji_from_shortcode}};\n"
    )?;

    // Write the Group enum
    writeln!(f, "/// A category for an emoji.")?;
    writeln!(f, "///")?;
    writeln!(f, "/// Based on Unicode CLDR data.")?;
    writeln!(
        f,
        "#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum, Eq, Hash, PartialOrd, Ord)]"
    )?;
    writeln!(
        f,
        r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#
    )?;
    writeln!(f, "pub enum Group {{")?;
    for name in unicode_data.keys() {
        if name == "Component" {
            continue;
        }
        writeln!(f, "   {name},")?;
    }
    writeln!(f, "}}\n")?;

    // Write SkinToneData struct
    writeln!(f, "/// Data about skin tone variants for an emoji.")?;
    writeln!(
        f,
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]"
    )?;
    writeln!(
        f,
        r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#
    )?;
    writeln!(f, "pub struct SkinToneData {{")?;
    writeln!(
        f,
        "    /// Index of the first skin tone variant in the EMOJIS array"
    )?;
    writeln!(f, "    pub first: usize,")?;
    writeln!(f, "    /// Number of skin tone variants")?;
    writeln!(f, "    pub second: usize,")?;
    writeln!(f, "    /// The skin tone of this emoji")?;
    writeln!(f, "    pub tone: SkinTone,")?;
    writeln!(f, "}}\n")?;

    // Write Emoji struct
    writeln!(f, "/// Data about an emoji.")?;
    writeln!(f, "#[derive(Debug, Clone, uniffi::Record)]")?;
    writeln!(
        f,
        r#"#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]"#
    )?;
    writeln!(f, "pub struct Emoji {{")?;
    writeln!(f, "    /// The emoji character.")?;
    writeln!(f, "    pub emoji: String,")?;
    writeln!(f, "    /// The CLDR name of the emoji.")?;
    writeln!(f, "    pub name: String,")?;
    writeln!(f, "    /// The Unicode version when this emoji was added.")?;
    writeln!(f, "    pub unicode_version: UnicodeVersion,")?;
    writeln!(f, "    /// The group this emoji belongs to.")?;
    writeln!(f, "    pub group: Group,")?;
    writeln!(f, "    /// Skin tone information about this emoji.")?;
    writeln!(f, "    pub skin_tone: Option<SkinToneData>,")?;
    writeln!(f, "    /// Shortcodes/aliases for this emoji.")?;
    writeln!(f, "    pub aliases: Option<Vec<String>>,")?;
    writeln!(f, "}}")?;

    Ok(())
}
