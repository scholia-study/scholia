//! Pass-1 mechanical rule table for the `md_modernized` layer.
//! A token pair — the 1807 surface form against CAB's
//! `@norm` — is *mechanical* when some subset of the spelling rules below
//! carries the ſ-folded surface exactly onto the norm. Everything else needs
//! an explicit ruling row in the checked-in decision table.
//!
//! CAB's norm is advisory, never the text: a pair is trusted only when both
//! CAB endorses it *and* a rule composition explains it. One systematic
//! exception: CAB predates the 1996 reform on ß (it writes `Bewußtsein` and
//! `ausser` alike), so any mechanically-explained result containing `ss` is
//! still routed to the decision table for a short-vowel/long-vowel ruling.

/// The long s is a glyph variant, not a spelling difference: folding it is
/// always safe and is the only change a `reject` ruling keeps.
pub fn fold_long_s(s: &str) -> String {
    s.replace('ſ', "s")
}

/// A spelling-archaism class of the 1807 print, in first-match-wins order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleClass {
    /// `iſt → ist` — nothing but the glyph fold.
    LongS,
    /// `ſeyn → sein`, `bey → bei`.
    EyEi,
    /// `That → Tat`, `nothwendig → notwendig`.
    ThT,
    /// `Krafft → Kraft`, `Herrschafft → Herrschaft`.
    FfF,
    /// `Subject → Subjekt`, `practiſch → praktiſch`.
    CK,
}

const SUBSTITUTIONS: [(RuleClass, &[(&str, &str)]); 4] = [
    (RuleClass::EyEi, &[("ey", "ei"), ("Ey", "Ei")]),
    (RuleClass::ThT, &[("th", "t"), ("Th", "T")]),
    (RuleClass::FfF, &[("ff", "f")]),
    (RuleClass::CK, &[("c", "k"), ("C", "K")]),
];

/// Where a surface/norm pair lands in the modernization pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classification {
    /// Surface and norm agree; the token is never touched.
    Identity,
    /// A rule subset explains the pair and the result carries no `ss`:
    /// trusted, emitted without a ruling row.
    Mechanical { result: String, class: RuleClass },
    /// A rule subset explains the pair but the result contains `ss`, which
    /// CAB cannot be trusted on (ß reform) — ruling row required.
    SsAudit { result: String },
    /// No rule subset explains the pair — ruling row required.
    Residual,
}

/// Classify a pair by the smallest rule subset (fewest rules, earliest class
/// first) that carries the folded surface onto the norm. Trying subsets
/// rather than one cumulative chain matters: `Affect → Affekt` needs `c→k`
/// while an interfering `ff→f` would strand it in the residual.
pub fn classify(surface: &str, norm: &str) -> Classification {
    if surface == norm {
        return Classification::Identity;
    }
    let folded = fold_long_s(surface);
    let explained = |result: String, class: RuleClass| {
        if result.contains("ss") {
            Classification::SsAudit { result }
        } else {
            Classification::Mechanical { result, class }
        }
    };
    if folded == norm {
        return explained(folded, RuleClass::LongS);
    }
    for size in 1..=SUBSTITUTIONS.len() {
        for mask in 1u8..16 {
            if mask.count_ones() as usize != size {
                continue;
            }
            let mut candidate = folded.clone();
            let mut first = None;
            for (bit, (class, subs)) in SUBSTITUTIONS.iter().enumerate() {
                if mask & (1 << bit) == 0 {
                    continue;
                }
                first.get_or_insert(*class);
                for (from, to) in *subs {
                    candidate = candidate.replace(from, to);
                }
            }
            if candidate == norm {
                return explained(candidate, first.expect("mask is non-empty"));
            }
        }
    }
    Classification::Residual
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mechanical(surface: &str, norm: &str) -> (String, RuleClass) {
        match classify(surface, norm) {
            Classification::Mechanical { result, class } => (result, class),
            other => panic!("{surface} → {norm} classified {other:?}"),
        }
    }

    #[test]
    fn glyph_fold_alone_is_the_long_s_class() {
        assert_eq!(
            mechanical("iſt", "ist"),
            ("ist".to_string(), RuleClass::LongS)
        );
    }

    #[test]
    fn ey_th_ff_ck_each_explain_their_class() {
        assert_eq!(
            mechanical("ſeyn", "sein"),
            ("sein".to_string(), RuleClass::EyEi)
        );
        assert_eq!(
            mechanical("That", "Tat"),
            ("Tat".to_string(), RuleClass::ThT)
        );
        assert_eq!(
            mechanical("Krafft", "Kraft"),
            ("Kraft".to_string(), RuleClass::FfF)
        );
        assert_eq!(
            mechanical("Subject", "Subjekt"),
            ("Subjekt".to_string(), RuleClass::CK)
        );
    }

    #[test]
    fn subsets_beat_a_cumulative_chain() {
        // ff→f applied blindly would give Afekt and strand the pair.
        assert_eq!(
            mechanical("Affect", "Affekt"),
            ("Affekt".to_string(), RuleClass::CK)
        );
    }

    #[test]
    fn compositions_still_explain() {
        assert_eq!(
            mechanical("Nothwendigkeit", "Notwendigkeit"),
            ("Notwendigkeit".to_string(), RuleClass::ThT)
        );
        assert_eq!(
            mechanical("ſeyender", "seiender"),
            ("seiender".to_string(), RuleClass::EyEi)
        );
    }

    #[test]
    fn ss_results_are_never_trusted() {
        assert_eq!(
            classify("daſs", "dass"),
            Classification::SsAudit {
                result: "dass".to_string()
            }
        );
        assert_eq!(
            classify("auſſer", "ausser"),
            Classification::SsAudit {
                result: "ausser".to_string()
            }
        );
    }

    #[test]
    fn cab_only_changes_are_residual() {
        // CAB's pre-reform ß: no rule writes ß, so the pair cannot mechanize.
        assert_eq!(
            classify("Bewuſstseyn", "Bewußtsein"),
            Classification::Residual
        );
        assert_eq!(classify("groſs", "groß"), Classification::Residual);
        // morphological rewrite
        assert_eq!(classify("andern", "anderen"), Classification::Residual);
    }

    #[test]
    fn identity_is_untouched() {
        assert_eq!(classify("Geist", "Geist"), Classification::Identity);
    }
}
