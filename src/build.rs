use std::collections::HashMap;
use std::fs;
use std::path::Path;

mod github;
mod unicode;

use serde::{Deserialize, Serialize};
use unicode::SkinTone;

use crate::unicode::UnicodeVersion;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Group {
    // Include all your group names here
    SmileysAndEmotion,
    PeopleAndBody,
    AnimalsAndNature,
    FoodAndDrink,
    TravelAndPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinToneData {
    pub first: usize,
    pub second: usize,
    pub tone: SkinTone,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Emoji {
    pub emoji: String,
    pub name: String,
    pub unicode_version: UnicodeVersion,
    pub group: Group,
    pub skin_tone: Option<SkinToneData>,
    pub aliases: Option<Vec<String>>,
}

// Serializable structure for all the data we need
#[derive(Serialize, Deserialize)]
struct EmojiData {
    emojis: Vec<Emoji>,
    unicode_map: HashMap<String, usize>,
    shortcode_map: HashMap<String, usize>,
}

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    // Parse emoji data
    let unicode_data = unicode::fetch_and_parse_emoji_data()?;
    let github_data = github::fetch_and_parse_emoji_data()?;

    let mut emojis = Vec::new();
    let mut unicode_map = HashMap::new();
    let mut shortcode_map = HashMap::new();

    let mut i = 0;
    let mut default_skin_tone_index = 0;
    let mut skin_tone_count = 0;

    // Process emoji data
    for (group, subgroups) in &unicode_data {
        for subgroup in subgroups.values() {
            for emoji in subgroup {
                if matches!(emoji.skin_tone(), Some(SkinTone::Default)) {
                    default_skin_tone_index = i;
                    skin_tone_count = emoji.skin_tones();
                }

                // Create Emoji struct
                let e = emoji.as_str();
                let name = emoji.name();
                let uv = emoji.unicode_version();

                let skin_tone = match emoji.skin_tone() {
                    Some(tone) => Some(SkinToneData {
                        first: default_skin_tone_index,
                        second: skin_tone_count,
                        tone,
                    }),
                    None => None,
                };

                let aliases = match &github_data.get(e) {
                    Some(github) => Some(github.aliases().iter().map(|s| s.to_string()).collect()),
                    None => None,
                };

                let emoji_struct = Emoji {
                    emoji: e.to_string(),
                    name: name.to_string(),
                    unicode_version: uv.clone(),
                    group: Group::from_str(group)?,
                    skin_tone,
                    aliases,
                };

                emojis.push(emoji_struct);

                // Build lookup maps
                unicode_map.insert(emoji.as_str().to_owned(), i);
                for v in emoji.variations() {
                    unicode_map.insert(v.to_owned(), i);
                }

                if let Some(github) = &github_data.get(emoji.as_str()) {
                    for alias in github.aliases() {
                        shortcode_map.insert(alias.to_owned(), i);
                    }
                }

                i += 1;
            }
        }
    }

    // Create the emoji data structure
    let emoji_data = EmojiData {
        emojis,
        unicode_map,
        shortcode_map,
    };

    // Serialize to bincode
    let bincode_data = bincode::serialize(&emoji_data)?;

    // Write to file in OUT_DIR
    let dest_path = Path::new("emoji_data.bin");
    fs::write(&dest_path, bincode_data)?;

    Ok(())
}

// Helper to convert string group to enum
impl Group {
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "SmileysAndEmotion" => Ok(Group::SmileysAndEmotion),
            "PeopleAndBody" => Ok(Group::PeopleAndBody),
            "AnimalsAndNature" => Ok(Group::AnimalsAndNature),
            "FoodAndDrink" => Ok(Group::FoodAndDrink),
            "TravelAndPlaces" => Ok(Group::TravelAndPlaces),
            "Activities" => Ok(Group::Activities),
            "Objects" => Ok(Group::Objects),
            "Symbols" => Ok(Group::Symbols),
            "Flags" => Ok(Group::Flags),
            _ => Err(anyhow::anyhow!("Unknown group: {}", s)),
        }
    }
}
