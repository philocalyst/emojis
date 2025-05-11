import emojisFFI
import Foundation // Keep this if needed, usually for Foundation types

// --- Assume EmojisBindings.swift is compiled alongside this ---
// (It contains the definitions for Emoji, UnicodeVersion, etc.)

@main // <--- ADD THIS ATTRIBUTE
struct EmojiTestRunner { // <--- Wrap your code in a struct (or class)

    static func main() { // <--- Put your executable code in static func main()
        // --- Your existing code goes here ---

        // 1. Create a UnicodeVersion instance
        let unicodeVersion13 = UnicodeVersion(major: 13, minor: 0)
        let unicodeVersion14 = UnicodeVersion(major: 14, minor: 0)

                let mediumSkinToneData = SkinToneData(
            first: 0,       // Placeholder - Use a value valid for UInt16 (e.g., 0)
            second: 0,      // Placeholder - actual value depends on emoji
            tone: .medium
        )

        print(skinTones(emoji: get(s: "👍")!))


        // 3. Create some Emoji instances manually
        let grinningFace = Emoji(
            emoji: "😀",
            name: "grinning face",
            unicodeVersion: unicodeVersion13,
            group: .smileysAndEmotion,
            skinTone: nil,
            aliases: ["happy", "smiley", ":)"]
        )

        let thumbsUpMedium = Emoji(
            emoji: "👍🏽",
            name: "thumbs up: medium skin tone",
            unicodeVersion: unicodeVersion14,
            group: .peopleAndBody,
            skinTone: mediumSkinToneData,
            aliases: ["like", "approve", "yes"]
        )


        // 4. Use the created Emoji objects
        print("--- Emoji 1 ---")
        print("Character: \(grinningFace.emoji)")
        print("Name: \(grinningFace.name)")
        print("Group: \(grinningFace.group)")
        print(
            "Unicode Version: \(grinningFace.unicodeVersion.major).\(grinningFace.unicodeVersion.minor)"
        )
        if let aliases = grinningFace.aliases {
            print("Aliases: \(aliases.joined(separator: ", "))")
        }
        if grinningFace.skinTone == nil {
            print("Skin Tone: Not applicable")
        } else {
            // Added else for clarity if needed
            print("Skin Tone: \(grinningFace.skinTone!.tone)")
        }


        print("\n--- Emoji 2 ---")
        print("Character: \(thumbsUpMedium.emoji)")
        print("Name: \(thumbsUpMedium.name)")
        print("Group: \(thumbsUpMedium.group)")
        print(
            "Unicode Version: \(thumbsUpMedium.unicodeVersion.major).\(thumbsUpMedium.unicodeVersion.minor)"
        )
        if let aliases = thumbsUpMedium.aliases {
            print("Aliases: \(aliases.joined(separator: ", "))")
        }
        if let skinToneInfo = thumbsUpMedium.skinTone {
            print("Skin Tone: \(skinToneInfo.tone)")
        } else {
            print("Skin Tone: Not applicable")
        }

        // --- End of your existing code ---
    }
}
