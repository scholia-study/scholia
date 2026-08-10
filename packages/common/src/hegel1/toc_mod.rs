//! Modernized-orthography TOC labels for the Phänomenologie des Geistes,
//! one per `toc` entry, in document order — the kant3 `toc_mod` pattern.
//!
//! Structural fields (position, depth, page, slug, supplied_heading) are
//! `toc`'s; only the label text re-spells, through the same rule table and
//! decision table as the body (`ey→ei`, `th→t`, `Krafft→Kraft`,
//! `Bewuſstseyn→Bewusstsein`, post-1996 ß/ss), so a file's front-matter
//! label in `md_modernized/` matches its entry here byte for byte. Rulings
//! carry over: elisions stay (`äußre`), and the `a,` of "a, Das Lichtwesen"
//! is the 1807 print's own comma, kept in both layers.

/// Modernized labels, one per `toc` entry, in document order.
pub const MODERNIZED_LABELS: &[&str] = &[
    "Vorrede",
    "Erster Teil. Wissenschaft der Erfahrung des Bewusstseins",
    "Einleitung",
    "I. Die sinnliche Gewissheit; oder das Diese und das Meinen",
    "II. Die Wahrnehmung; oder das Ding, und die Täuschung",
    "III. Kraft und Verstand, Erscheinung und übersinnliche Welt",
    "IV. Die Wahrheit der Gewissheit seiner selbst",
    "A. Selbstständigkeit und Unselbstständigkeit des Selbstbewusstseins; Herrschaft und Knechtschaft",
    "B. Freiheit des Selbstbewusstseins; Stoizismus, Skeptizismus, und das unglückliche Bewusstsein",
    "V. Gewissheit und Wahrheit der Vernunft",
    "A. Beobachtende Vernunft",
    "a. Beobachtung der Natur",
    "b. Die Beobachtung des Selbstbewusstseins in seiner Reinheit und seiner Beziehung auf äußre Wirklichkeit; logische und psychologische Gesetze",
    "c. Beobachtung der Beziehung des Selbstbewusstseins auf seine unmittelbare Wirklichkeit; Physiognomik und Schädellehre",
    "B. Die Verwirklichung des vernünftigen Selbstbewusstseins durch sich selbst",
    "a. Die Lust und die Notwendigkeit",
    "b. Das Gesetz des Herzens, und der Wahnsinn des Eigendünkels",
    "c. Die Tugend und der Weltlauf",
    "C. Die Individualität, welche sich an und für sich selbst reell ist",
    "a. Das geistige Tierreich und der Betrug, oder die Sache selbst",
    "b. Die gesetzgebende Vernunft",
    "c. Gesetzprüfende Vernunft",
    "VI. Der Geist",
    "A. Der wahre Geist, die Sittlichkeit",
    "a. Die sittliche Welt, das menschliche und göttliche Gesetz, der Mann und das Weib",
    "b. Die sittliche Handlung, das menschliche und göttliche Wissen, die Schuld und das Schicksal",
    "c. Rechtszustand",
    "B. Der sich entfremdete Geist; die Bildung",
    "I. Die Welt des sich entfremdeten Geistes",
    "a. Die Bildung und ihr Reich der Wirklichkeit",
    "b. Der Glauben und die reine Einsicht",
    "II. Die Aufklärung",
    "a. Der Kampf der Aufklärung mit dem Aberglauben",
    "b. Die Wahrheit der Aufklärung",
    "III. Die absolute Freiheit und der Schrecken",
    "C. Der seiner selbst gewisse Geist. Die Moralität",
    "a. Die moralische Weltanschauung",
    "b. Die Verstellung",
    "c. Das Gewissen, die schöne Seele, das Böse und seine Verzeihung",
    "VII. Die Religion",
    "A. Natürliche Religion",
    "a, Das Lichtwesen",
    "b. Die Pflanze und das Tier",
    "c. Der Werkmeister",
    "B. Die Kunst-Religion",
    "a. Das abstrakte Kunstwerk",
    "b. Das lebendige Kunstwerk",
    "c. Das geistige Kunstwerk",
    "C. Die offenbare Religion",
    "VIII. Das absolute Wissen",
];

/// Flattened rows for `md_prose_to_struct::corpus` — structural fields from
/// `toc`, labels from this table.
pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    super::toc::entries()
        .iter()
        .enumerate()
        .map(|(i, e)| {
            (
                i,
                e.page.map(|p| p.to_string()),
                e.depth,
                MODERNIZED_LABELS[i],
                None,
            )
        })
        .collect()
}

/// The modernized label for a `toc` entry, by 0-based document index.
pub fn label(index: usize) -> &'static str {
    MODERNIZED_LABELS[index]
}

#[cfg(test)]
mod tests {
    use super::super::toc;
    use super::*;

    #[test]
    fn one_label_per_toc_entry() {
        assert_eq!(MODERNIZED_LABELS.len(), toc::entries().len());
    }

    #[test]
    fn no_archaic_spelling_survives() {
        for l in MODERNIZED_LABELS {
            assert!(!l.contains('ſ'), "long-s in {l}");
            assert!(!l.contains("ey"), "ey in {l}");
            assert!(!l.contains("fft"), "fft in {l}");
        }
    }

    #[test]
    fn supplied_heading_label_is_shared() {
        let einleitung = toc::entries()
            .iter()
            .position(|e| e.supplied_heading)
            .expect("the Einleitung is flagged");
        assert_eq!(MODERNIZED_LABELS[einleitung], "Einleitung");
    }
}
