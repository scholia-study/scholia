/// Convert a Roman numeral string to an integer.
/// Handles standard subtractive notation (I–DCCCXCIX+).
pub fn roman_to_int(s: &str) -> Option<u32> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let mut total: u32 = 0;
    let mut prev: u32 = 0;

    for ch in s.chars().rev() {
        let val = match ch {
            'I' | 'i' => 1,
            'V' | 'v' => 5,
            'X' | 'x' => 10,
            'L' | 'l' => 50,
            'C' | 'c' => 100,
            'D' | 'd' => 500,
            'M' | 'm' => 1000,
            _ => return None,
        };
        if val < prev {
            total -= val;
        } else {
            total += val;
        }
        prev = val;
    }

    if total == 0 { None } else { Some(total) }
}

/// Roman page numbers sort below this, Arabic at or above it, so one reference
/// system can paginate a preface in Roman and its body in Arabic without the
/// two series colliding (1807 Phänomenologie: Vorrede II–XCI, body 4–765).
const ROMAN_SORT_FLOOR: i32 = -10_000;

/// Sort order for a block page marker whose value may be Arabic or Roman.
///
/// Arabic keeps its face value, so corpora whose block system is entirely
/// Arabic (kant1, kant3) are unaffected. Roman maps below every Arabic page
/// while staying monotonic within its own series.
pub fn block_sort_order(value: &str) -> i32 {
    value
        .parse::<i32>()
        .ok()
        .or_else(|| roman_to_int(value).map(|n| ROMAN_SORT_FLOOR + n as i32))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arabic_block_pages_keep_their_face_value() {
        assert_eq!(block_sort_order("203"), 203);
        assert_eq!(block_sort_order("4"), 4);
    }

    #[test]
    fn roman_block_pages_sort_below_every_arabic_page() {
        assert!(block_sort_order("XCI") < block_sort_order("4"));
        assert!(block_sort_order("II") < block_sort_order("XCI"));
        assert_eq!(block_sort_order("II"), -9998);
    }

    #[test]
    fn an_unparseable_value_sorts_first_without_panicking() {
        assert_eq!(block_sort_order("?"), 0);
    }

    #[test]
    fn test_basic() {
        assert_eq!(roman_to_int("I"), Some(1));
        assert_eq!(roman_to_int("IV"), Some(4));
        assert_eq!(roman_to_int("VII"), Some(7));
        assert_eq!(roman_to_int("XIV"), Some(14));
        assert_eq!(roman_to_int("XLIV"), Some(44));
        assert_eq!(roman_to_int("DCCCLXXXIV"), Some(884));
    }

    #[test]
    fn test_invalid() {
        assert_eq!(roman_to_int(""), None);
        assert_eq!(roman_to_int("ABC"), None);
    }
}
