use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

pub fn normalize(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .nfd()
        .flat_map(char::to_lowercase)
        .filter(|character| !is_combining_mark(*character))
        .collect()
}

pub fn valid_label(value: &str, max: usize) -> bool {
    value.chars().count() <= max
        && !value.chars().any(char::is_control)
        && !normalize(value).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_case_accents_and_word_spacing() {
        assert_eq!(normalize("  A\u{301}NIME   Clásico "), "anime clasico");
        assert!(!valid_label("   ", 200));
        assert!(!valid_label("a\nb", 200));
        assert!(!valid_label("abcd", 3));
    }
}
pub mod watch;
