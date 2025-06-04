//! ✨ Lookup emoji in *O(1)* time, access metadata and GitHub shortcodes,
//! iterate over all emoji.
//!
//! # Features
//!
//! - Lookup up emoji by Unicode value
//! - Lookup up emoji by GitHub shortcode ([gemoji] v4.1.0)
//! - Access emoji metadata: name, unicode version, group, skin tone, [gemoji] shortcodes
//! - Iterate over emojis in Unicode CLDR order
//! - Iterate over emojis in an emoji group, e.g. "Smileys & Emotion" or "Flags"
//! - Iterate over the skin tones for an emoji
//! - Select a specific skin tone for an emoji
//! - Uses [Unicode v16.0](https://unicode.org/emoji/charts-16.0/emoji-released.html) emoji specification
//!
//! [gemoji]: https://github.com/github/gemoji
//!
//! # Getting started
//!
//! First, add the `emojis` crate to your Cargo manifest.
//!
//! ```sh
//! cargo add emojis
//! ```
//!
//! Simply use the `get()` function to lookup emojis by Unicode value.
//! ```
//! let rocket = emojis::get("🚀").unwrap();
//! ```
//!
//! Or the `get_by_shortcode()` function to lookup emojis by [gemoji] shortcode.
//!
//! ```
//! let rocket = emojis::get_by_shortcode("rocket").unwrap();
//! ```
//!
//! These operations take *Ο(1)* time.
//!
//! # MSRV
//!
//! Currently the minimum supported Rust version is 1.60 due to the dependency
//! on [`phf`]. The policy of this crate is to only increase the MSRV in a
//! breaking release.
//!
//! # Breaking changes
//!
//! When [gemoji] or the Unicode version is upgraded this is not considered a
//! breaking change, instead you should make sure to use
//! [`unicode_version()`][Emoji::unicode_version] to filter out newer versions.
//!
//! # Examples
//!
//! See [examples/replace.rs] for an example that replaces `:gemoji:` names with
//! real emojis in text.
//!
//! ```sh
//! $ echo "launch :rocket:" | cargo run --example replace
//! launch 🚀
//! ```
//!
//! [`get()`][get] and [`get_by_shortcode()`][get_by_shortcode] return an
//! [`Emoji`] struct which contains various metadata regarding the emoji.
//! ```
//! let hand = emojis::get("🤌").unwrap();
//! assert_eq!(hand.as_str(), "\u{1f90c}");
//! assert_eq!(hand.raw_bytes(), &[0xf0, 0x9f, 0xa4, 0x8c]);
//! ```
//!
//! Use [`skin_tones()`][Emoji::skin_tones] to iterate over the skin tones of an
//! emoji.
//! ```
//! let raised_hands: emojis::Emoji = emojis::get("🙌🏼").unwrap();
//! let variants = raised_hands.skin_tones();
//! let iter = variants.iter().cloned();
//! let skin_tones: Vec<_> = iter.collect();
//! assert_eq!(skin_tones, ["🙌", "🙌🏻", "🙌🏼", "🙌🏽", "🙌🏾", "🙌🏿"]);
//! ```
//!
//! You can use the [`iter()`] function to iterate over all emojis. This only
//! includes the default skin tone versions.
//! ```
//! let faces: Vec<_> = emojis::iter().map(|e| e.as_str()).take(5).collect();
//! assert_eq!(faces, ["😀", "😃", "😄", "😁", "😆"]);
//! ```
//!
//!
//! Using the [`Group`] enum you can iterate over all emojis in a group.
//! ```
//! let fruit: Vec<_> = emojis::Group::FoodAndDrink.emojis().map(|e| e.as_str()).take(5).collect();
//! assert_eq!(fruit, ["🍇", "🍈", "🍉", "🍊", "🍋"]);
//! ```
//!
//! [examples/replace.rs]: https://github.com/rossmacarthur/emojis/blob/trunk/examples/replace.rs
//! [gemoji]: https://github.com/github/gemoji

pub mod common;
pub mod emoji;

use core::cmp;
use core::convert;
use core::fmt;
use core::hash;

use crate::emoji::{Emoji, Group};

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// A Unicode version.
#[derive(
    Debug, Deserialize, Encode, Serialize, Clone, Copy, Decode, PartialEq, Eq, Hash, PartialOrd, Ord,
)]
pub struct UnicodeVersion {
    major: u32,
    minor: u32,
}

/// The skin tone of an emoji.
#[derive(Debug, Clone, Encode, Decode, Deserialize, Serialize, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum SkinTone {
    Default,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
    LightAndMediumLight,
    LightAndMedium,
    LightAndMediumDark,
    LightAndDark,
    MediumLightAndLight,
    MediumLightAndMedium,
    MediumLightAndMediumDark,
    MediumLightAndDark,
    MediumAndLight,
    MediumAndMediumLight,
    MediumAndMediumDark,
    MediumAndDark,
    MediumDarkAndLight,
    MediumDarkAndMediumLight,
    MediumDarkAndMedium,
    MediumDarkAndDark,
    DarkAndLight,
    DarkAndMediumLight,
    DarkAndMedium,
    DarkAndMediumDark,
}

impl UnicodeVersion {
    /// Construct a new version.
    #[inline]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    #[inline]
    pub const fn major(self) -> u32 {
        self.major
    }

    #[inline]
    pub const fn minor(self) -> u32 {
        self.minor
    }
}

#[inline]
pub fn skin_tones(emoji: &Emoji) -> Vec<Emoji> {
    let data = match &emoji.skin_tone {
        Some(d) => d,
        None => return vec![],
    };
    crate::common::EMOJIS
        .iter()
        .skip(data.first as usize)
        .take(data.second as usize)
        .cloned()
        .collect()
}

impl Emoji {
    pub fn skin_tones(&self) -> Vec<Emoji> {
        let data = match &self.skin_tone {
            Some(d) => d,
            None => return vec![],
        };
        crate::common::EMOJIS
            .iter()
            .skip(data.first as usize)
            .take(data.second as usize)
            .cloned()
            .collect()
    }

    /// Returns the first GitHub shortcode for this emoji.
    ///
    /// Most emojis only have zero or one shortcode but for a few there are
    /// multiple. Use the [`shortcodes()`][Emoji::shortcodes] method to return
    /// all the shortcodes. See [gemoji] for more information.
    ///
    /// For emojis that have zero shortcodes this will return `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let thinking = emojis::get("🤔").unwrap();
    /// assert_eq!(thinking.shortcode().unwrap(), "thinking");
    /// ```
    ///
    /// [gemoji]: https://github.com/github/gemoji
    #[inline]
    pub fn shortcode(self) -> Option<String> {
        self.aliases.and_then(|aliases| aliases.first().cloned())
    }
}

// 4) Any purely‐Rust helpers go here, not in the #[uniffi::export] impl:
impl Emoji {
    /// Borrow the raw &str (used only inside Rust)
    pub fn as_str(&self) -> &str {
        &self.emoji
    }

    /// Borrow the raw bytes (used only inside Rust)
    pub fn raw_bytes(&self) -> &[u8] {
        self.emoji.as_bytes()
    }

    pub fn skin_tone(&self) -> Option<SkinTone> {
        self.skin_tone.as_ref().map(|d| d.tone)
    }

    /// Iterator over variants with skin_tone (Rust only)
    pub fn skin_tones_iter(&self) -> Option<impl Iterator<Item = &Emoji> + Clone> {
        let d = self.skin_tone.as_ref()?;
        let slice: &[Emoji] = &*crate::common::EMOJIS;
        Some(slice.iter().skip(d.first as usize).take(d.second as usize))
    }
}

impl cmp::PartialEq<Emoji> for Emoji {
    #[inline]
    fn eq(&self, other: &Emoji) -> bool {
        self.emoji == other.emoji
    }
}

impl cmp::PartialEq<str> for Emoji {
    #[inline]
    fn eq(&self, s: &str) -> bool {
        self.as_str() == s
    }
}

impl cmp::PartialEq<&str> for Emoji {
    #[inline]
    fn eq(&self, s: &&str) -> bool {
        self.as_str() == *s
    }
}

impl cmp::Eq for Emoji {}

impl hash::Hash for Emoji {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.emoji.hash(state);
    }
}

impl convert::AsRef<str> for Emoji {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl convert::AsRef<[u8]> for Emoji {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.emoji.as_bytes()
    }
}

impl fmt::Display for Emoji {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Group {
    /// Returns an iterator over all groups.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut iter = emojis::Group::iter();
    /// assert_eq!(iter.next().unwrap(), emojis::Group::SmileysAndEmotion);
    /// assert_eq!(iter.next().unwrap(), emojis::Group::PeopleAndBody);
    /// ```
    #[inline]
    pub fn iter() -> impl Iterator<Item = Group> + Clone {
        [
            Self::SmileysAndEmotion,
            Self::PeopleAndBody,
            Self::AnimalsAndNature,
            Self::FoodAndDrink,
            Self::TravelAndPlaces,
            Self::Activities,
            Self::Objects,
            Self::Symbols,
            Self::Flags,
        ]
        .iter()
        .copied()
    }

    /// Returns an iterator over all emojis in this group.
    ///
    /// # Examples
    ///
    /// ```
    /// let flags: Vec<_> = emojis::Group::Flags.emojis().map(|e| e.as_str()).take(5).collect();
    /// assert_eq!(flags, ["🏁", "🚩", "🎌", "🏴", "🏳️"]);
    /// ```
    #[inline]
    pub fn emojis(&self) -> impl Iterator<Item = &'static Emoji> {
        let group = *self;
        iter()
            .skip_while(move |emoji| emoji.group != group)
            .take_while(move |emoji| emoji.group == group)
    }
}

/// Returns an iterator over all emojis.
///
/// - Ordered by Unicode CLDR data.
/// - Excludes non-default skin tones.
///
/// # Examples
///
/// ```
/// let faces: Vec<_> = emojis::iter().map(|e| e.as_str()).take(5).collect();
/// assert_eq!(faces, ["😀", "😃", "😄", "😁", "😆"]);
/// ```
#[inline]
pub fn iter() -> impl Iterator<Item = &'static Emoji> + Clone {
    crate::common::EMOJIS
        .iter()
        .filter(|emoji| matches!(emoji.skin_tone(), Some(SkinTone::Default) | None))
}

/// Lookup an emoji by Unicode value.
///
/// This take *Ο(1)* time.
///
/// # Note
///
/// If passed a minimally qualified or unqualified emoji this will return the
/// emoji struct containing the fully qualified version.
///
/// # Examples
///
/// In the ordinary case.
///
/// ```
/// let emoji = "🚀";
/// let rocket = emojis::get(emoji).unwrap();
/// assert!(rocket.as_str() == emoji);
/// assert_eq!(rocket.shortcode().unwrap(), "rocket");
/// ```
///
/// For a minimally qualified or unqualified emoji.
///
/// ```
/// let unqualified = "\u{1f43f}";
/// let fully_qualified = "\u{1f43f}\u{fe0f}";
/// let chipmunk = emojis::get(unqualified).unwrap();
/// assert_eq!(chipmunk.as_str(), fully_qualified);
/// assert_eq!(chipmunk.shortcode().unwrap(), "chipmunk");
/// ```
#[inline]
pub fn get(s: &str) -> Option<Emoji> {
    crate::common::unicode::MAP
        .get(s)
        .map(|&i| crate::common::EMOJIS[i].clone())
}

/// Lookup an emoji by GitHub shortcode.
///
/// This take *Ο(1)* time.
///
/// # Examples
///
/// ```
/// let rocket = emojis::get_by_shortcode("rocket").unwrap();
/// assert_eq!(rocket, "🚀");
/// ```
#[inline]
pub fn get_by_shortcode(s: &str) -> Option<&'static Emoji> {
    crate::common::shortcode::MAP
        .get(s)
        .map(|&i| &crate::common::EMOJIS[i])
}
