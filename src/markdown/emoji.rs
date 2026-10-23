//! GitHub-style emoji shortcodes (`:smile:`) for rendered output.
//!
//! Only the rendered view rewrites shortcodes; the markdown source keeps the
//! literal `:name:` text so editing round-trips are lossless.

/// Map of common GitHub emoji names to their glyphs.
///
/// Kept deliberately small — enough for everyday documents without pulling in
/// a multi-thousand-entry table.
const EMOJI: &[(&str, &str)] = &[
    ("smile", "😄"),
    ("smiley", "😃"),
    ("grin", "😁"),
    ("joy", "😂"),
    ("laughing", "😆"),
    ("wink", "😉"),
    ("blush", "😊"),
    ("heart_eyes", "😍"),
    ("thinking", "🤔"),
    ("neutral_face", "😐"),
    ("sunglasses", "😎"),
    ("cry", "😢"),
    ("sob", "😭"),
    ("angry", "😠"),
    ("rage", "😡"),
    ("scream", "😱"),
    ("sleeping", "😴"),
    ("shrug", "🤷"),
    ("thumbsup", "👍"),
    ("+1", "👍"),
    ("thumbsdown", "👎"),
    ("-1", "👎"),
    ("clap", "👏"),
    ("wave", "👋"),
    ("ok_hand", "👌"),
    ("pray", "🙏"),
    ("muscle", "💪"),
    ("eyes", "👀"),
    ("heart", "❤️"),
    ("broken_heart", "💔"),
    ("sparkling_heart", "💖"),
    ("star", "⭐"),
    ("star2", "🌟"),
    ("sparkles", "✨"),
    ("fire", "🔥"),
    ("zap", "⚡"),
    ("boom", "💥"),
    ("tada", "🎉"),
    ("rocket", "🚀"),
    ("bug", "🐛"),
    ("check", "✅"),
    ("white_check_mark", "✅"),
    ("heavy_check_mark", "✔️"),
    ("x", "❌"),
    ("warning", "⚠️"),
    ("bulb", "💡"),
    ("memo", "📝"),
    ("book", "📖"),
    ("books", "📚"),
    ("link", "🔗"),
    ("lock", "🔒"),
    ("unlock", "🔓"),
    ("key", "🔑"),
    ("gear", "⚙️"),
    ("wrench", "🔧"),
    ("hammer", "🔨"),
    ("package", "📦"),
    ("folder", "📁"),
    ("file_folder", "📁"),
    ("page_facing_up", "📄"),
    ("clipboard", "📋"),
    ("calendar", "📅"),
    ("clock1", "🕐"),
    ("hourglass", "⌛"),
    ("bell", "🔔"),
    ("gift", "🎁"),
    ("gem", "💎"),
    ("coffee", "☕"),
    ("beer", "🍺"),
    ("pizza", "🍕"),
    ("cake", "🎂"),
    ("apple", "🍎"),
    ("sunny", "☀️"),
    ("moon", "🌙"),
    ("cloud", "☁️"),
    ("umbrella", "☔"),
    ("snowflake", "❄️"),
    ("earth_asia", "🌏"),
    ("computer", "💻"),
    ("phone", "📱"),
    ("email", "✉️"),
    ("envelope", "✉️"),
    ("chart_with_upwards_trend", "📈"),
    ("bar_chart", "📊"),
    ("moneybag", "💰"),
    ("heavy_dollar_sign", "💲"),
    ("yen", "💴"),
    ("credit_card", "💳"),
    ("shopping_cart", "🛒"),
    ("truck", "🚚"),
    ("house", "🏠"),
    ("office", "🏢"),
    ("100", "💯"),
    ("question", "❓"),
    ("exclamation", "❗"),
    ("plus", "➕"),
    ("minus", "➖"),
    ("arrow_right", "➡️"),
    ("arrow_left", "⬅️"),
    ("arrow_up", "⬆️"),
    ("arrow_down", "⬇️"),
];

/// Replace `:name:` shortcodes with emoji glyphs.
///
/// Unknown shortcodes and stray colons are preserved verbatim.
pub fn replace_shortcodes(input: &str) -> String {
    if !input.contains(':') {
        return input.to_string();
    }

    let mut out = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(start) = rest.find(':') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find(':') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let name = &after[..end];
        let valid = !name.is_empty()
            && name.len() <= 32
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '+' || c == '-');

        match EMOJI.iter().find(|(key, _)| *key == name) {
            Some((_, glyph)) if valid => out.push_str(glyph),
            _ => {
                out.push(':');
                out.push_str(name);
                out.push(':');
            }
        }
        rest = &after[end + 1..];
    }

    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::replace_shortcodes;

    #[test]
    fn replaces_known_shortcodes() {
        assert_eq!(replace_shortcodes("hi :wave:"), "hi 👋");
        assert_eq!(replace_shortcodes(":rocket::star:"), "🚀⭐");
    }

    #[test]
    fn keeps_unknown_and_text() {
        assert_eq!(replace_shortcodes(":not_an_emoji:"), ":not_an_emoji:");
        assert_eq!(replace_shortcodes("10:30"), "10:30");
        assert_eq!(replace_shortcodes("no colons"), "no colons");
    }

    #[test]
    fn tolerates_unpaired_colon() {
        assert_eq!(replace_shortcodes("trailing :smile"), "trailing :smile");
    }
}
