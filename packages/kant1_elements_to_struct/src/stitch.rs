use crate::model::InputLine;

// ---------------------------------------------------------------------------
// Line stitching within a paragraph element
// ---------------------------------------------------------------------------

/// A B-page reference anchored at a character offset in stitched text.
#[derive(Debug, Clone)]
pub struct BPageAnchor {
    pub b_page: String,
    pub char_offset: usize,
}

/// Stitch lines into a single paragraph string, handling hyphenated words.
///
/// Rules:
/// - If a line ends with `-`, join with the next line's first word (remove hyphen, no space).
/// - Otherwise join lines with a space.
/// - Record b_page_ref positions as character offsets in the output.
pub fn stitch_lines(lines: &[InputLine]) -> (String, Vec<BPageAnchor>) {
    let mut result = String::new();
    let mut anchors = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let text = line.text.trim();
        if text.is_empty() {
            continue;
        }

        // Join with previous line first
        if i > 0 && !result.is_empty() {
            if result.ends_with('-') {
                // Hyphenated word: remove the hyphen and join directly
                result.pop();
            } else {
                // Normal line break: join with space
                result.push(' ');
            }
        }

        // Record b_page_ref at the position where this line's text starts
        if let Some(ref b_ref) = line.b_page_ref {
            anchors.push(BPageAnchor {
                b_page: b_ref.clone(),
                char_offset: result.len(),
            });
        }

        result.push_str(text);
    }

    (result, anchors)
}

/// Stitch lines from an element's b_page_refs list (element-level refs that
/// weren't associated with individual lines).
pub fn collect_element_b_refs(
    element_b_refs: &[String],
    line_anchors: &[BPageAnchor],
) -> Vec<String> {
    let mut refs: Vec<String> = line_anchors.iter().map(|a| a.b_page.clone()).collect();
    for r in element_b_refs {
        if !refs.contains(r) {
            refs.push(r.clone());
        }
    }
    refs
}

// ---------------------------------------------------------------------------
// Cross-page hyphenation stitching
// ---------------------------------------------------------------------------

/// If the last line of the last element on a page ends with `-`,
/// join with the first word of the next page's first element.
///
/// Returns a list of (page_idx, joined_word) for logging purposes.
pub fn stitch_across_pages(
    pages: &mut [crate::model::InputPage],
) -> Vec<(usize, String)> {
    let mut joins = Vec::new();

    for i in 0..pages.len().saturating_sub(1) {
        // Get the last line's text from the last element on this page
        let ends_with_hyphen = pages[i]
            .elements
            .last()
            .and_then(|e| e.lines.last())
            .is_some_and(|l| l.text.trim().ends_with('-'));

        if !ends_with_hyphen {
            continue;
        }

        // Get the first word from the next page's first element's first line
        let first_word = match pages[i + 1]
            .elements
            .first()
            .and_then(|e| e.lines.first())
        {
            Some(line) => {
                let text = line.text.trim();
                text.split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_string()
            }
            None => continue,
        };

        if first_word.is_empty() {
            continue;
        }

        // Modify the last line of the current page: remove hyphen, append first word
        let last_elem = pages[i].elements.last_mut().unwrap();
        let last_line = last_elem.lines.last_mut().unwrap();
        let mut text = last_line.text.trim_end().to_string();
        if text.ends_with('-') {
            text.pop();
        }
        text.push_str(&first_word);
        let joined = text.clone();
        last_line.text = text;

        // Also update the element-level text
        last_elem.text = last_elem
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Remove the first word from the next page's first line
        let next_elem = &mut pages[i + 1].elements[0];
        let next_line = &mut next_elem.lines[0];
        let trimmed = next_line.text.trim();
        next_line.text = trimmed
            .splitn(2, char::is_whitespace)
            .nth(1)
            .unwrap_or("")
            .trim_start()
            .to_string();

        // Update element-level text
        next_elem.text = next_elem
            .lines
            .iter()
            .map(|l| l.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        joins.push((pages[i].page_index, joined));
    }

    joins
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::InputLine;

    fn line(text: &str, b_ref: Option<&str>) -> InputLine {
        InputLine {
            text: text.to_string(),
            line_number: None,
            b_page_ref: b_ref.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_simple_stitch() {
        let lines = vec![
            line("Erste Zeile", None),
            line("zweite Zeile.", None),
        ];
        let (text, anchors) = stitch_lines(&lines);
        assert_eq!(text, "Erste Zeile zweite Zeile.");
        assert!(anchors.is_empty());
    }

    #[test]
    fn test_hyphen_stitch() {
        let lines = vec![
            line("anthropolo-", None),
            line("gische Bedeutung.", None),
        ];
        let (text, anchors) = stitch_lines(&lines);
        assert_eq!(text, "anthropologische Bedeutung.");
        assert!(anchors.is_empty());
    }

    #[test]
    fn test_b_page_ref_anchor() {
        let lines = vec![
            line("Erster Teil des", None),
            line("Satzes hier.", Some("XIV")),
        ];
        let (text, anchors) = stitch_lines(&lines);
        assert_eq!(text, "Erster Teil des Satzes hier.");
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].b_page, "XIV");
        assert_eq!(anchors[0].char_offset, 16); // position of "Satzes"
    }

    #[test]
    fn test_hyphen_with_b_ref() {
        let lines = vec![
            line("Vernunft-", None),
            line("erkenntniß ist wichtig.", Some("42")),
        ];
        let (text, anchors) = stitch_lines(&lines);
        assert_eq!(text, "Vernunfterkenntniß ist wichtig.");
        assert_eq!(anchors.len(), 1);
        assert_eq!(anchors[0].b_page, "42");
    }
}
