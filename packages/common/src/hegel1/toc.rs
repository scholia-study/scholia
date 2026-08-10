//! Authoritative TOC for Hegel's Phänomenologie des Geistes (1807).
//!
//! One entry per content node, in document order, transcribed from the
//! Deutsches Textarchiv TEI: the 53 `<div>` elements less the `contents`,
//! `imprint` and `advertisement` apparatus, leaving 50. Labels are the head
//! text in the 1807 orthography, long-s (`ſ`) included, trailing period
//! stripped; `slug` is the label put through `filenames::slugify`.
//!
//! Depths form a strict tree — every node's parent sits exactly one level up.
//! The book itself is the implicit depth-0 root, created by the importer.

/// The printed page a node starts on. The 1807 edition paginates the Vorrede
/// in Roman numerals and the body in Arabic, so the two cannot share one
/// integer: Roman pages keep their printed form, and the distinction also
/// decides whether the curated file's `page_1807` front-matter value is
/// quoted (Roman) or bare (Arabic).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Roman(&'static str),
    Arabic(u16),
}

impl Page {
    pub fn is_roman(&self) -> bool {
        matches!(self, Page::Roman(_))
    }
}

impl std::fmt::Display for Page {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Page::Roman(n) => f.write_str(n),
            Page::Arabic(n) => write!(f, "{n}"),
        }
    }
}

pub struct TocEntry {
    /// 1-based document position, also the `NNN` filename prefix.
    pub position: u16,
    pub depth: u16,
    /// The 1807 page the node starts on. `None` for the Vorrede, which opens
    /// before the first numbered `<pb>`.
    pub page: Option<Page>,
    pub slug: &'static str,
    pub label: &'static str,
    /// `true` where Scholia supplies a heading the 1807 print does not have —
    /// an editorial act a reader may need to know about. Only the Einleitung
    /// qualifies: its `<div n="2">` carries a literal empty `<head/>`.
    pub supplied_heading: bool,
}

const TOC: &[TocEntry] = &[
    TocEntry {
        position: 1,
        depth: 1,
        page: None,
        slug: "vorrede",
        label: "Vorrede",
        supplied_heading: false,
    },
    TocEntry {
        position: 2,
        depth: 1,
        page: Some(Page::Roman("XCI")),
        slug: "erster_theil_wissenschaft_der_erfahrung_des_bewusstseyns",
        label: "Erster Theil. Wissenschaft der Erfahrung des Bewuſstseyns",
        supplied_heading: false,
    },
    TocEntry {
        position: 3,
        depth: 2,
        page: Some(Page::Roman("XCI")),
        slug: "einleitung",
        label: "Einleitung",
        supplied_heading: true,
    },
    TocEntry {
        position: 4,
        depth: 2,
        page: Some(Page::Arabic(22)),
        slug: "i_die_sinnliche_gewissheit_oder_das_diese_und_das_meynen",
        label: "I. Die sinnliche Gewiſsheit; oder das Diese und das Meynen",
        supplied_heading: false,
    },
    TocEntry {
        position: 5,
        depth: 2,
        page: Some(Page::Arabic(38)),
        slug: "ii_die_wahrnehmung_oder_das_ding_und_die_taeuschung",
        label: "II. Die Wahrnehmung; oder das Ding, und die Täuschung",
        supplied_heading: false,
    },
    TocEntry {
        position: 6,
        depth: 2,
        page: Some(Page::Arabic(59)),
        slug: "iii_krafft_und_verstand_erscheinung_und_uebersinnliche_welt",
        label: "III. Krafft und Verstand, Erscheinung und übersinnliche Welt",
        supplied_heading: false,
    },
    TocEntry {
        position: 7,
        depth: 2,
        page: Some(Page::Arabic(101)),
        slug: "iv_die_wahrheit_der_gewissheit_seiner_selbst",
        label: "IV. Die Wahrheit der Gewiſsheit seiner selbst",
        supplied_heading: false,
    },
    TocEntry {
        position: 8,
        depth: 3,
        page: Some(Page::Arabic(114)),
        slug: "a_selbststaendigkeit_und_unselbststaendigkeit_des_selbstbewusstseyns_herrschafft_und_knechtschafft",
        label: "A. Selbstständigkeit und Unselbstständigkeit des Selbstbewuſstseyns; Herrschafft und Knechtschafft",
        supplied_heading: false,
    },
    TocEntry {
        position: 9,
        depth: 3,
        page: Some(Page::Arabic(129)),
        slug: "b_freyheit_des_selbstbewusstseyns_stoicismus_skepticismus_und_das_unglueckliche_bewusstseyn",
        label: "B. Freyheit des Selbstbewuſstseyns; Stoicismus, Skepticismus, und das unglückliche Bewuſstseyn",
        supplied_heading: false,
    },
    TocEntry {
        position: 10,
        depth: 2,
        page: Some(Page::Arabic(162)),
        slug: "v_gewissheit_und_wahrheit_der_vernunft",
        label: "V. Gewiſsheit und Wahrheit der Vernunft",
        supplied_heading: false,
    },
    TocEntry {
        position: 11,
        depth: 3,
        page: Some(Page::Arabic(174)),
        slug: "a_beobachtende_vernunft",
        label: "A. Beobachtende Vernunft",
        supplied_heading: false,
    },
    TocEntry {
        position: 12,
        depth: 4,
        page: Some(Page::Arabic(177)),
        slug: "a_beobachtung_der_natur",
        label: "a. Beobachtung der Natur",
        supplied_heading: false,
    },
    TocEntry {
        position: 13,
        depth: 4,
        page: Some(Page::Arabic(234)),
        slug: "b_die_beobachtung_des_selbstbewusstseyns_in_seiner_reinheit_und_seiner_beziehung_auf_aeussre_wirklichkeit_logische_und_psychologische_gesetze",
        label: "b. Die Beobachtung des Selbstbewuſstseyns in seiner Reinheit und seiner Beziehung auf äuſsre Wirklichkeit; logische und psychologische Gesetze",
        supplied_heading: false,
    },
    TocEntry {
        position: 14,
        depth: 4,
        page: Some(Page::Arabic(243)),
        slug: "c_beobachtung_der_beziehung_des_selbstbewusstseyns_auf_seine_unmittelbare_wirklichkeit_physiognomik_und_schaedellehre",
        label: "c. Beobachtung der Beziehung des Selbstbewuſstseyns auf seine unmittelbare Wirklichkeit; Physiognomik und Schädellehre",
        supplied_heading: false,
    },
    TocEntry {
        position: 15,
        depth: 3,
        page: Some(Page::Arabic(286)),
        slug: "b_die_verwirklichung_des_vernuenftigen_selbstbewusstseyns_durch_sich_selbst",
        label: "B. Die Verwirklichung des vernünftigen Selbstbewuſstseyns durch sich selbst",
        supplied_heading: false,
    },
    TocEntry {
        position: 16,
        depth: 4,
        page: Some(Page::Arabic(298)),
        slug: "a_die_lust_und_die_nothwendigkeit",
        label: "a. Die Luſt und die Nothwendigkeit",
        supplied_heading: false,
    },
    TocEntry {
        position: 17,
        depth: 4,
        page: Some(Page::Arabic(305)),
        slug: "b_das_gesetz_des_herzens_und_der_wahnsinn_des_eigenduenkels",
        label: "b. Das Gesetz des Herzens, und der Wahnsinn des Eigendünkels",
        supplied_heading: false,
    },
    TocEntry {
        position: 18,
        depth: 4,
        page: Some(Page::Arabic(317)),
        slug: "c_die_tugend_und_der_weltlauff",
        label: "c. Die Tugend und der Weltlauff",
        supplied_heading: false,
    },
    TocEntry {
        position: 19,
        depth: 3,
        page: Some(Page::Arabic(330)),
        slug: "c_die_individualitaet_welche_sich_an_und_fuer_sich_selbst_reell_ist",
        label: "C. Die Individualität, welche sich an und für sich selbst reell ist",
        supplied_heading: false,
    },
    TocEntry {
        position: 20,
        depth: 4,
        page: Some(Page::Arabic(333)),
        slug: "a_das_geistige_thierreich_und_der_betrug_oder_die_sache_selbs",
        label: "a. Das geistige Thierreich und der Betrug, oder die Sache selbs’",
        supplied_heading: false,
    },
    TocEntry {
        position: 21,
        depth: 4,
        page: Some(Page::Arabic(358)),
        slug: "b_die_gesetzgebende_vernunfft",
        label: "b. Die gesetzgebende Vernunfft",
        supplied_heading: false,
    },
    TocEntry {
        position: 22,
        depth: 4,
        page: Some(Page::Arabic(365)),
        slug: "c_gesetzprueffende_vernunfft",
        label: "c. Gesetzprüffende Vernunfft",
        supplied_heading: false,
    },
    TocEntry {
        position: 23,
        depth: 2,
        page: Some(Page::Arabic(376)),
        slug: "vi_der_geist",
        label: "VI. Der Geist",
        supplied_heading: false,
    },
    TocEntry {
        position: 24,
        depth: 3,
        page: Some(Page::Arabic(382)),
        slug: "a_der_wahre_geist_die_sittlichkeit",
        label: "A. Der wahre Geist, die Sittlichkeit",
        supplied_heading: false,
    },
    TocEntry {
        position: 25,
        depth: 4,
        page: Some(Page::Arabic(383)),
        slug: "a_die_sittliche_welt_das_menschliche_und_goettliche_gesetz_der_mann_und_das_weib",
        label: "a. Die sittliche Welt, das menschliche und göttliche Gesetz, der Mann und das Weib",
        supplied_heading: false,
    },
    TocEntry {
        position: 26,
        depth: 4,
        page: Some(Page::Arabic(403)),
        slug: "b_die_sittliche_handlung_das_menschliche_und_goettliche_wissen_die_schuld_und_das_schicksal",
        label: "b. Die sittliche Handlung, das menschliche und göttliche Wissen, die Schuld und das Schicksal",
        supplied_heading: false,
    },
    TocEntry {
        position: 27,
        depth: 4,
        page: Some(Page::Arabic(422)),
        slug: "c_rechtszustand",
        label: "c. Rechtszustand",
        supplied_heading: false,
    },
    TocEntry {
        position: 28,
        depth: 3,
        page: Some(Page::Arabic(429)),
        slug: "b_der_sich_entfremdete_geist_die_bildung",
        label: "B. Der sich entfremdete Geist; die Bildung",
        supplied_heading: false,
    },
    TocEntry {
        position: 29,
        depth: 4,
        page: Some(Page::Arabic(434)),
        slug: "i_die_welt_des_sich_entfremdeten_geistes",
        label: "I. Die Welt des sich entfremdeten Geistes",
        supplied_heading: false,
    },
    TocEntry {
        position: 30,
        depth: 5,
        page: Some(Page::Arabic(435)),
        slug: "a_die_bildung_und_ihr_reich_der_wirklichkeit",
        label: "a. Die Bildung und ihr Reich der Wirklichkeit",
        supplied_heading: false,
    },
    TocEntry {
        position: 31,
        depth: 5,
        page: Some(Page::Arabic(474)),
        slug: "b_der_glauben_und_die_reine_einsicht",
        label: "b. Der Glauben und die reine Einsicht",
        supplied_heading: false,
    },
    TocEntry {
        position: 32,
        depth: 4,
        page: Some(Page::Arabic(486)),
        slug: "ii_die_aufklaerung",
        label: "II. Die Aufklärung",
        supplied_heading: false,
    },
    TocEntry {
        position: 33,
        depth: 5,
        page: Some(Page::Arabic(488)),
        slug: "a_der_kampf_der_aufklaerung_mit_dem_aberglauben",
        label: "a. Der Kampf der Aufklärung mit dem Aberglauben",
        supplied_heading: false,
    },
    TocEntry {
        position: 34,
        depth: 5,
        page: Some(Page::Arabic(522)),
        slug: "b_die_wahrheit_der_aufklaerung",
        label: "b. Die Wahrheit der Aufklärung",
        supplied_heading: false,
    },
    TocEntry {
        position: 35,
        depth: 4,
        page: Some(Page::Arabic(533)),
        slug: "iii_die_absolute_freyheit_und_der_schrecken",
        label: "III. Die abſolute Freyheit und der Schrecken",
        supplied_heading: false,
    },
    TocEntry {
        position: 36,
        depth: 3,
        page: Some(Page::Arabic(548)),
        slug: "c_der_seiner_selbst_gewisse_geist_die_moralitaet",
        label: "C. Der seiner selbst gewiſse Geist. Die Moralität",
        supplied_heading: false,
    },
    TocEntry {
        position: 37,
        depth: 4,
        page: Some(Page::Arabic(550)),
        slug: "a_die_moralische_weltanschauung",
        label: "a. Die moralische Weltanschauung",
        supplied_heading: false,
    },
    TocEntry {
        position: 38,
        depth: 4,
        page: Some(Page::Arabic(565)),
        slug: "b_die_verstellung",
        label: "b. Die Verstellung",
        supplied_heading: false,
    },
    TocEntry {
        position: 39,
        depth: 4,
        page: Some(Page::Arabic(581)),
        slug: "c_das_gewissen_die_schoene_seele_das_boese_und_seine_verzeyhung",
        label: "c. Das Gewiſſen, die ſchöne Seele, das Böſe und ſeine Verzeyhung",
        supplied_heading: false,
    },
    TocEntry {
        position: 40,
        depth: 2,
        page: Some(Page::Arabic(625)),
        slug: "vii_die_religion",
        label: "VII. Die Religion",
        supplied_heading: false,
    },
    TocEntry {
        position: 41,
        depth: 3,
        page: Some(Page::Arabic(637)),
        slug: "a_natuerliche_religion",
        label: "A. Natürliche Religion",
        supplied_heading: false,
    },
    TocEntry {
        position: 42,
        depth: 4,
        page: Some(Page::Arabic(640)),
        slug: "a_das_lichtwesen",
        label: "a, Das Lichtwesen",
        supplied_heading: false,
    },
    TocEntry {
        position: 43,
        depth: 4,
        page: Some(Page::Arabic(643)),
        slug: "b_die_pflanze_und_das_thier",
        label: "b. Die Pflanze und das Thier",
        supplied_heading: false,
    },
    TocEntry {
        position: 44,
        depth: 4,
        page: Some(Page::Arabic(645)),
        slug: "c_der_werkmeister",
        label: "c. Der Werkmeister",
        supplied_heading: false,
    },
    TocEntry {
        position: 45,
        depth: 3,
        page: Some(Page::Arabic(651)),
        slug: "b_die_kunst_religion",
        label: "B. Die Kunst-Religion",
        supplied_heading: false,
    },
    TocEntry {
        position: 46,
        depth: 4,
        page: Some(Page::Arabic(655)),
        slug: "a_das_abstracte_kunstwerk",
        label: "a. Das abstracte Kunstwerk",
        supplied_heading: false,
    },
    TocEntry {
        position: 47,
        depth: 4,
        page: Some(Page::Arabic(669)),
        slug: "b_das_lebendige_kunstwerk",
        label: "b. Das lebendige Kunſtwerk",
        supplied_heading: false,
    },
    TocEntry {
        position: 48,
        depth: 4,
        page: Some(Page::Arabic(676)),
        slug: "c_das_geistige_kunstwerk",
        label: "c. Das geiſtige Kunſtwerk",
        supplied_heading: false,
    },
    TocEntry {
        position: 49,
        depth: 3,
        page: Some(Page::Arabic(699)),
        slug: "c_die_offenbare_religion",
        label: "C. Die offenbare Religion",
        supplied_heading: false,
    },
    TocEntry {
        position: 50,
        depth: 2,
        page: Some(Page::Arabic(742)),
        slug: "viii_das_absolute_wissen",
        label: "VIII. Das abſolute Wiſſen",
        supplied_heading: false,
    },
];

/// Flattened rows for `md_prose_to_struct::corpus` — reviewed labels; the
/// page is the printed 1807 page as a string (Roman in the Vorrede, Arabic
/// in the body), `None` for the Vorrede itself.
pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    TOC.iter()
        .enumerate()
        .map(|(i, e)| (i, e.page.map(|p| p.to_string()), e.depth, e.label, None))
        .collect()
}

pub fn entries() -> &'static [TocEntry] {
    TOC
}

pub fn toc_len() -> usize {
    TOC.len()
}

/// Index of the entry that parents `index` — the nearest preceding entry
/// exactly one level up. `None` for depth-1 nodes, which hang off the book
/// root. This is the same backward scan the importer uses to build ltree
/// paths.
pub fn parent_of(index: usize) -> Option<usize> {
    let target = TOC[index].depth.checked_sub(1)?;
    if target == 0 {
        return None;
    }
    TOC[..index].iter().rposition(|e| e.depth == target)
}

#[cfg(test)]
mod tests {
    use super::super::filenames::slugify;
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_entry_count() {
        assert_eq!(TOC.len(), 50);
        assert_eq!(TOC[0].label, "Vorrede");
        assert_eq!(TOC[49].label, "VIII. Das abſolute Wiſſen");
    }

    #[test]
    fn test_positions_contiguous() {
        for (i, e) in TOC.iter().enumerate() {
            assert_eq!(e.position as usize, i + 1, "position gap at index {i}");
        }
    }

    /// A strict tree: depth never jumps by more than one, so every node has a
    /// parent exactly one level up and the ltree path is well defined.
    #[test]
    fn test_depths_form_strict_tree() {
        assert_eq!(TOC[0].depth, 1, "first node must be a root child");
        for pair in TOC.windows(2) {
            assert!(
                pair[1].depth <= pair[0].depth + 1,
                "depth jumps from {} to {} at {:?}",
                pair[0].depth,
                pair[1].depth,
                pair[1].label
            );
        }
        for (i, e) in TOC.iter().enumerate() {
            if e.depth > 1 {
                let parent = parent_of(i).expect("no parent");
                assert_eq!(TOC[parent].depth, e.depth - 1);
            }
        }
    }

    #[test]
    fn test_slugs_unique() {
        let unique: HashSet<&str> = TOC.iter().map(|e| e.slug).collect();
        assert_eq!(unique.len(), TOC.len(), "duplicate slug in hegel1 TOC");
    }

    #[test]
    fn test_slugs_are_lowercase_ascii_underscore() {
        for e in TOC {
            assert!(!e.slug.is_empty(), "empty slug for {:?}", e.label);
            assert!(
                e.slug
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "slug {:?} is not lowercase-ascii-with-underscores",
                e.slug
            );
        }
    }

    /// The slugs are stored rather than derived, so pin them to the rule that
    /// produced them — a hand-edited label must not silently keep a stale slug.
    #[test]
    fn test_slugs_match_slugified_labels() {
        for e in TOC {
            assert_eq!(slugify(e.label), e.slug, "slug drift for {:?}", e.label);
        }
    }

    /// Roman pages belong to the Vorrede region, Arabic to the body, and the
    /// Vorrede itself opens before the first numbered page break.
    #[test]
    fn test_page_numbering_regions() {
        assert!(TOC[0].page.is_none());
        assert_eq!(TOC[1].page, Some(Page::Roman("XCI")));
        assert_eq!(TOC[2].page, Some(Page::Roman("XCI")));
        for e in &TOC[3..] {
            assert!(
                matches!(e.page, Some(Page::Arabic(_))),
                "body node {:?} must have an Arabic page",
                e.label
            );
        }
    }

    #[test]
    fn test_body_pages_ascend() {
        let arabic: Vec<u16> = TOC
            .iter()
            .filter_map(|e| match e.page {
                Some(Page::Arabic(n)) => Some(n),
                _ => None,
            })
            .collect();
        assert!(
            arabic.windows(2).all(|w| w[0] < w[1]),
            "body pages are not strictly ascending"
        );
    }

    #[test]
    fn test_einleitung_is_the_only_supplied_heading() {
        let supplied: Vec<&str> = TOC
            .iter()
            .filter(|e| e.supplied_heading)
            .map(|e| e.label)
            .collect();
        assert_eq!(supplied, vec!["Einleitung"]);
    }
}
