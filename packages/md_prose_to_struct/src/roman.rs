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

/// Sort order for a block page marker whose value may be Arabic, Roman,
/// suffixed Arabic, or dotted volume.page.
///
/// Arabic keeps its face value, so corpora whose block system is entirely
/// Arabic (kant1, kant3) are unaffected. Roman maps below every Arabic page
/// while staying monotonic within its own series. A dotted volume.page
/// (hegel2's GW system) blocks by volume — sort is only consulted within a
/// sentence, where volumes never mix.
/// [`block_sort_order`] with the letter suffix treated as a SUB-PAGE address
/// rather than a duplicate-page disambiguator: `327a` → 3270, `327e` → 3274,
/// so Stephanus sections stay strictly ordered within their page. Opt-in per
/// corpus — hobbes1's `247b` is the second printing of page 247 and must keep
/// sorting as 247, next to it.
pub fn block_sort_order_subpage(value: &str) -> i32 {
    let digits = value.trim_end_matches(|c: char| c.is_ascii_lowercase());
    let letters = &value[digits.len()..];
    if !digits.is_empty()
        && letters.len() == 1
        && let Ok(page) = digits.parse::<i32>()
        && let Some(ch) = letters.chars().next()
    {
        return page * 10 + (ch as i32 - 'a' as i32);
    }
    // No single-letter suffix: keep the page's own value on the same scale.
    block_sort_order(value) * 10
}

pub fn block_sort_order(value: &str) -> i32 {
    value
        .parse::<i32>()
        .ok()
        .or_else(|| dotted_volume_page(value))
        .or_else(|| venue_volume_page(value))
        .or_else(|| suffixed_arabic(value))
        .or_else(|| roman_to_int(value).map(|n| ROMAN_SORT_FLOOR + n as i32))
        .unwrap_or(0)
}

/// "PSM 12:1" → 12_001: like the dotted form, blocked by volume. The venue is
/// dropped rather than ranked — sort is only consulted among the markers of a
/// single sentence, and no sentence spans two periodicals.
fn venue_volume_page(value: &str) -> Option<i32> {
    let (_venue, vol_page) = value.split_once(' ')?;
    let (vol, page) = vol_page.split_once(':')?;
    let vol: i32 = vol.parse().ok()?;
    let page: i32 = page.parse().ok()?;
    (page < 1000).then_some(vol * 1000 + page)
}

/// "21.68" → 21_068: volume-blocked so pages stay monotonic per volume.
fn dotted_volume_page(value: &str) -> Option<i32> {
    let (vol, page) = value.split_once('.')?;
    let vol: i32 = vol.parse().ok()?;
    let page: i32 = page.parse().ok()?;
    (page < 1000).then_some(vol * 1000 + page)
}

/// "247b" → 247: the twice-printed 1651 Leviathan pages carry a letter suffix
/// on their second occurrence; they sort by the printed number (document
/// order breaks the tie).
fn suffixed_arabic(value: &str) -> Option<i32> {
    let digits = value.trim_end_matches(|c: char| c.is_ascii_lowercase());
    if digits.len() < value.len() && !digits.is_empty() {
        digits.parse().ok()
    } else {
        None
    }
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
    fn venue_qualified_pages_sort_by_page_within_volume() {
        assert_eq!(block_sort_order("PSM 12:1"), 12_001);
        assert!(block_sort_order("PSM 12:1") < block_sort_order("PSM 12:15"));
        assert!(block_sort_order("Monist 2:533") < block_sort_order("Monist 3:1"));
    }

    #[test]
    fn venue_qualified_pages_ignore_the_venue() {
        // Sort is only consulted within one sentence, which never spans two
        // periodicals — so colliding volume numbers across venues are fine.
        assert_eq!(
            block_sort_order("JSP 2:103"),
            block_sort_order("Monist 2:103")
        );
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

#[cfg(test)]
mod subpage_tests {
    use super::*;

    #[test]
    fn stephanus_sections_stay_ordered_within_a_page() {
        let refs = ["327a", "327b", "327c", "327d", "327e", "328a"];
        let orders: Vec<i32> = refs.iter().map(|r| block_sort_order_subpage(r)).collect();
        assert_eq!(orders, vec![3270, 3271, 3272, 3273, 3274, 3280]);
        assert!(orders.windows(2).all(|w| w[0] < w[1]), "not monotonic");
    }

    #[test]
    fn whole_republic_range_is_monotonic() {
        let mut prev = i32::MIN;
        for page in 327..=621 {
            for letter in ["a", "b", "c", "d", "e"] {
                let o = block_sort_order_subpage(&format!("{page}{letter}"));
                assert!(o > prev, "{page}{letter} broke monotonicity");
                prev = o;
            }
        }
    }

    /// hobbes1's duplicate-page suffix must NOT move: it is the same printed
    /// page, and the opt-in flag is what keeps the two readings apart.
    #[test]
    fn duplicate_page_suffix_is_unaffected_by_the_default() {
        assert_eq!(block_sort_order("247b"), 247);
        assert_eq!(block_sort_order("247"), 247);
        assert_eq!(block_sort_order("248"), 248);
    }
}
