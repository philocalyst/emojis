// tests/smoke.rs

// Pull in everything we need.
use emojis::{self, emoji::Emoji, emoji::Group, SkinTone, UnicodeVersion};

#[test]
fn smoke_test_suite() {
    // 1) get_variation
    assert_eq!(emojis::get("☹"), emojis::get("☹️"));

    // 2) iter_only_default_skin_tones
    assert!(
        emojis::iter().all(|e| { matches!(e.skin_tone(), Some(SkinTone::Default) | None) }),
        "Found a non‐default skin tone in iter()"
    );
    assert!(
        emojis::iter()
            .filter(|e| matches!(e.skin_tone(), Some(SkinTone::Default)))
            .count()
            > 0,
        "Expected at least one default‐skin‐tone emoji"
    );

    // 3) unicode_version_partial_ord
    assert!(UnicodeVersion::new(13, 0) >= UnicodeVersion::new(12, 0));
    assert!(UnicodeVersion::new(12, 1) >= UnicodeVersion::new(12, 0));
    assert!(UnicodeVersion::new(12, 0) >= UnicodeVersion::new(12, 0));
    assert!(UnicodeVersion::new(12, 0) < UnicodeVersion::new(12, 1));
    assert!(UnicodeVersion::new(11, 0) < UnicodeVersion::new(12, 1));

    // 4) emoji_partial_eq_str  & emoji_display
    let grinning = emojis::get("😀").unwrap();
    assert_eq!(grinning, "😀");
    assert_eq!(grinning.to_string(), "😀");

    // 5) emoji_skin_tones
    let expected = [
        SkinTone::Default,
        SkinTone::Light,
        SkinTone::MediumLight,
        SkinTone::Medium,
        SkinTone::MediumDark,
        SkinTone::Dark,
        SkinTone::LightAndMediumLight,
        SkinTone::LightAndMedium,
        SkinTone::LightAndMediumDark,
        SkinTone::LightAndDark,
        SkinTone::MediumLightAndLight,
        SkinTone::MediumLightAndMedium,
        SkinTone::MediumLightAndMediumDark,
        SkinTone::MediumLightAndDark,
        SkinTone::MediumAndLight,
        SkinTone::MediumAndMediumLight,
        SkinTone::MediumAndMediumDark,
        SkinTone::MediumAndDark,
        SkinTone::MediumDarkAndLight,
        SkinTone::MediumDarkAndMediumLight,
        SkinTone::MediumDarkAndMedium,
        SkinTone::MediumDarkAndDark,
        SkinTone::DarkAndLight,
        SkinTone::DarkAndMediumLight,
        SkinTone::DarkAndMedium,
        SkinTone::DarkAndMediumDark,
    ];

    for e in emojis::iter() {
        if e.skin_tone().is_some() {
            let variants = e.skin_tones();
            assert!(
                variants.len() == 6 || variants.len() == 26,
                "{} had {} variants",
                e,
                variants.len()
            );
            for (v, &tone) in variants.iter().zip(expected.iter()) {
                assert_eq!(
                    v.skin_tone().unwrap(),
                    tone,
                    "variant list for {} was wrong",
                    e
                );
            }
        } else {
            assert!(e.skin_tones().is_empty());
        }
    }

    // 6) group_iter_and_emojis
    let by_group: Vec<&Emoji> = Group::iter().flat_map(|g| g.emojis()).collect();
    let flat: Vec<&Emoji> = emojis::iter().collect();
    assert_eq!(by_group, flat);
}
