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

#[cfg(test)]
mod tests {
    use super::*;

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
