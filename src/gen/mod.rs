use crate::{Emoji, SkinToneData, UnicodeVersion};
use bincode;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(
    Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, uniffi::Enum,
)]
pub enum Group {
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

// This struct MUST match the Emoji in lib.rs
#[derive(Serialize, Deserialize, Debug)]
struct SerializedEmoji {
    emoji: String,
    name: String,
    unicode_version: UnicodeVersion,
    group: Group,
    skin_tone: Option<SkinToneData>,
    aliases: Option<Vec<String>>,
}

// This struct MUST match the one in build.rs
#[derive(Serialize, Deserialize)]
struct EmojiData {
    emojis: Vec<SerializedEmoji>,
    unicode_map: HashMap<String, usize>,
    shortcode_map: HashMap<String, usize>,
}

// Include the pre-compiled emoji data using a static path
// This is a workaround since we can't use env!("OUT_DIR") at compile time
static EMOJI_DATA_BYTES: &[u8] = include_bytes!("../../emoji_data.bin");

// Lazily deserialize the emoji data
static EMOJI_DATA: Lazy<EmojiData> =
    Lazy::new(|| bincode::deserialize(EMOJI_DATA_BYTES).expect("Failed to deserialize emoji data"));

// Exported emoji list
pub static EMOJIS: Lazy<Vec<Emoji>> = Lazy::new(|| {
    EMOJI_DATA
        .emojis
        .iter()
        .map(|e| Emoji {
            emoji: e.emoji.clone(),
            name: e.name.clone(),
            unicode_version: e.unicode_version,
            group: e.group,
            skin_tone: e.skin_tone.clone(),
            aliases: e.aliases.clone(),
        })
        .collect()
});

// Map accessors for unicode and shortcode lookups
pub mod unicode {
    use super::*;

    pub fn get_emoji_index(unicode: &str) -> Option<usize> {
        EMOJI_DATA.unicode_map.get(unicode).copied()
    }
}

pub mod shortcode {
    use super::*;

    pub fn get_emoji_index(shortcode: &str) -> Option<usize> {
        EMOJI_DATA.shortcode_map.get(shortcode).copied()
    }
}
