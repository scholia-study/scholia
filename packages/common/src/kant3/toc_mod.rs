// ---------------------------------------------------------------------------
// Modernized-orthography TOC labels for Kritik der Urteilskraft
// ---------------------------------------------------------------------------
//
// The structural fields (aa_page, depth, slug_override) are reused verbatim from
// `toc`; only the labels are modernized (Urtheil→Urteil, Theil→Teil,
// Princip→Prinzip, transscendental→transzendental, -ction→-ktion, object→objekt,
// nothwendig→notwendig, eigenthümlich→eigentümlich, …). Greek-derived "th"
// (Mathematisch, Methodenlehre, Physikotheologie, Ethikotheologie) is preserved.
//
// This is the modernization standard for the §-level labels; the body
// modernization pass (`md_modernized/`) must follow the same choices so each
// file's front-matter label matches its entry here.

use super::toc;

/// Modernized labels, one per `toc` entry, in document order.
const MODERNIZED_LABELS: &[&str] = &[
    // Front matter + Einleitung
    "Vorrede",
    "Einleitung",
    "I. Von der Einteilung der Philosophie",
    "II. Vom Gebiete der Philosophie überhaupt",
    "III. Von der Kritik der Urteilskraft, als einem Verbindungsmittel der zwei Teile der Philosophie zu einem Ganzen",
    "IV. Von der Urteilskraft, als einem _a priori_ gesetzgebenden Vermögen",
    "V. Das Prinzip der formalen Zweckmäßigkeit der Natur ist ein transzendentales Prinzip der Urteilskraft",
    "VI. Von der Verbindung des Gefühls der Lust mit dem Begriffe der Zweckmäßigkeit der Natur",
    "VII. Von der ästhetischen Vorstellung der Zweckmäßigkeit der Natur",
    "VIII. Von der logischen Vorstellung der Zweckmäßigkeit der Natur",
    "IX. Von der Verknüpfung der Gesetzgebungen des Verstandes und der Vernunft durch die Urteilskraft",
    "Einteilung des ganzen Werks",
    // Erster Teil
    "Erster Teil. Kritik der ästhetischen Urteilskraft",
    "Erster Abschnitt. Analytik der ästhetischen Urteilskraft",
    "Erstes Buch. Analytik des Schönen",
    "1. Moment des Geschmacksurteils der Qualität nach",
    "§ 1. Das Geschmacksurteil ist ästhetisch",
    "§ 2. Das Wohlgefallen, welches das Geschmacksurteil bestimmt, ist ohne alles Interesse",
    "§ 3. Das Wohlgefallen am Angenehmen ist mit Interesse verbunden",
    "§ 4. Das Wohlgefallen am Guten ist mit Interesse verbunden",
    "§ 5. Vergleichung der drei spezifisch verschiedenen Arten des Wohlgefallens",
    "2. Moment des Geschmacksurteils, nämlich seiner Quantität nach",
    "§ 6. Das Schöne ist das, was ohne Begriff als Objekt eines allgemeinen Wohlgefallens vorgestellt wird",
    "§ 7. Vergleichung des Schönen mit dem Angenehmen und Guten durch obiges Merkmal",
    "§ 8. Die Allgemeinheit des Wohlgefallens wird in einem Geschmacksurteile nur als subjektiv vorgestellt",
    "§ 9. Untersuchung der Frage: ob im Geschmacksurteile das Gefühl der Lust vor der Beurteilung des Gegenstandes, oder diese vor jener vorhergehe",
    "3. Moment der Geschmacksurteile nach der Relation der Zwecke, welche in ihnen in Betrachtung gezogen wird",
    "§ 10. Von der Zweckmäßigkeit überhaupt",
    "§ 11. Das Geschmacksurteil hat nichts als die Form der Zweckmäßigkeit eines Gegenstandes (oder der Vorstellungsart desselben) zum Grunde",
    "§ 12. Das Geschmacksurteil beruht auf Gründen _a priori_",
    "§ 13. Das reine Geschmacksurteil ist von Reiz und Rührung unabhängig",
    "§ 14. Erläuterung durch Beispiele",
    "§ 15. Das Geschmacksurteil ist von dem Begriffe der Vollkommenheit gänzlich unabhängig",
    "§ 16. Das Geschmacksurteil, wodurch ein Gegenstand unter der Bedingung eines bestimmten Begriffs für schön erklärt wird, ist nicht rein",
    "§ 17. Vom Ideale der Schönheit",
    "4. Moment des Geschmacksurteils nach der Modalität des Wohlgefallens an dem Gegenstande",
    "§ 18. Was die Modalität eines Geschmacksurteils sei",
    "§ 19. Die subjektive Notwendigkeit, die wir dem Geschmacksurteile beilegen, ist bedingt",
    "§ 20. Die Bedingung der Notwendigkeit, die ein Geschmacksurteil vorgibt, ist die Idee eines Gemeinsinnes",
    "§ 21. Ob man mit Grunde einen Gemeinsinn voraussetzen könne",
    "§ 22. Die Notwendigkeit der allgemeinen Beistimmung, die in einem Geschmacksurteil gedacht wird, ist eine subjektive Notwendigkeit, die unter der Voraussetzung eines Gemeinsinnes als objektiv vorgestellt wird",
    "Allgemeine Anmerkung zum ersten Abschnitte der Analytik",
    "Zweites Buch. Analytik des Erhabenen",
    "§ 23. Übergang von dem Beurteilungsvermögen des Schönen zu dem des Erhabenen",
    "§ 24. Von der Einteilung einer Untersuchung des Gefühls des Erhabenen",
    "A. Vom Mathematisch-Erhabenen",
    "§ 25. Namenerklärung des Erhabenen",
    "§ 26. Von der Größenschätzung der Naturdinge, die zur Idee des Erhabenen erforderlich ist",
    "§ 27. Von der Qualität des Wohlgefallens in der Beurteilung des Erhabenen",
    "B. Vom Dynamisch-Erhabenen der Natur",
    "§ 28. Von der Natur als einer Macht",
    "§ 29. Von der Modalität des Urteils über das Erhabene der Natur",
    "Allgemeine Anmerkung zur Exposition der ästhetischen reflektierenden Urteile",
    "Deduktion der reinen ästhetischen Urteile",
    "§ 30. Die Deduktion der ästhetischen Urteile über die Gegenstände der Natur darf nicht auf das, was wir in dieser erhaben nennen, sondern nur auf das Schöne gerichtet werden",
    "§ 31. Von der Methode der Deduktion der Geschmacksurteile",
    "§ 32. Erste Eigentümlichkeit des Geschmacksurteils",
    "§ 33. Zweite Eigentümlichkeit des Geschmacksurteils",
    "§ 34. Es ist kein objektives Prinzip des Geschmacks möglich",
    "§ 35. Das Prinzip des Geschmacks ist das subjektive Prinzip der Urteilskraft überhaupt",
    "§ 36. Von der Aufgabe einer Deduktion der Geschmacksurteile",
    "§ 37. Was wird eigentlich in einem Geschmacksurteile von einem Gegenstande _a priori_ behauptet?",
    "§ 38. Deduktion der Geschmacksurteile",
    "§ 39. Von der Mitteilbarkeit einer Empfindung",
    "§ 40. Vom Geschmacke als einer Art von _sensus communis_",
    "§ 41. Vom empirischen Interesse am Schönen",
    "§ 42. Vom intellektuellen Interesse am Schönen",
    "§ 43. Von der Kunst überhaupt",
    "§ 44. Von der schönen Kunst",
    "§ 45. Schöne Kunst ist eine Kunst, sofern sie zugleich Natur zu sein scheint",
    "§ 46. Schöne Kunst ist Kunst des Genies",
    "§ 47. Erläuterung und Bestätigung obiger Erklärung vom Genie",
    "§ 48. Vom Verhältnisse des Genies zum Geschmack",
    "§ 49. Von den Vermögen des Gemüts, welche das Genie ausmachen",
    "§ 50. Von der Verbindung des Geschmacks mit Genie in Produkten der schönen Kunst",
    "§ 51. Von der Einteilung der schönen Künste",
    "§ 52. Von der Verbindung der schönen Künste in einem und demselben Produkte",
    "§ 53. Vergleichung des ästhetischen Werts der schönen Künste untereinander",
    "§ 54. Anmerkung",
    "Zweiter Abschnitt. Dialektik der ästhetischen Urteilskraft",
    "§ 55.",
    "§ 56. Vorstellung der Antinomie des Geschmacks",
    "§ 57. Auflösung der Antinomie des Geschmacks",
    "§ 58. Vom Idealismus der Zweckmäßigkeit der Natur sowohl als Kunst, als dem alleinigen Prinzip der ästhetischen Urteilskraft",
    "§ 59. Von der Schönheit als Symbol der Sittlichkeit",
    "§ 60. Anhang. Von der Methodenlehre des Geschmacks",
    // Zweiter Teil
    "Zweiter Teil. Kritik der teleologischen Urteilskraft",
    "§ 61. Von der objektiven Zweckmäßigkeit der Natur",
    "Erste Abteilung. Analytik der teleologischen Urteilskraft",
    "§ 62. Von der objektiven Zweckmäßigkeit, die bloß formal ist, zum Unterschiede von der materialen",
    "§ 63. Von der relativen Zweckmäßigkeit der Natur zum Unterschiede von der innern",
    "§ 64. Von dem eigentümlichen Charakter der Dinge als Naturzwecke",
    "§ 65. Dinge als Naturzwecke sind organisierte Wesen",
    "§ 66. Vom Prinzip der Beurteilung der innern Zweckmäßigkeit in organisierten Wesen",
    "§ 67. Vom Prinzip der teleologischen Beurteilung der Natur überhaupt als System der Zwecke",
    "§ 68. Von dem Prinzip der Teleologie als innerem Prinzip der Naturwissenschaft",
    "Zweite Abteilung. Dialektik der teleologischen Urteilskraft",
    "§ 69. Was eine Antinomie der Urteilskraft sei",
    "§ 70. Vorstellung dieser Antinomie",
    "§ 71. Vorbereitung zur Auflösung obiger Antinomie",
    "§ 72. Von den mancherlei Systemen über die Zweckmäßigkeit der Natur",
    "§ 73. Keines der obigen Systeme leistet das, was es vorgibt",
    "§ 74. Die Ursache der Unmöglichkeit, den Begriff einer Technik der Natur dogmatisch zu behandeln, ist die Unerklärlichkeit eines Naturzwecks",
    "§ 75. Der Begriff einer objektiven Zweckmäßigkeit der Natur ist ein kritisches Prinzip der Vernunft für die reflektierende Urteilskraft",
    "§ 76. Anmerkung",
    "§ 77. Von der Eigentümlichkeit des menschlichen Verstandes, wodurch uns der Begriff eines Naturzwecks möglich wird",
    "§ 78. Von der Vereinigung des Prinzips des allgemeinen Mechanismus der Materie mit dem teleologischen in der Technik der Natur",
    "Anhang. Methodenlehre der teleologischen Urteilskraft",
    "§ 79. Ob die Teleologie als zur Naturlehre gehörend abgehandelt werden müsse",
    "§ 80. Von der notwendigen Unterordnung des Prinzips des Mechanismus unter dem teleologischen in Erklärung eines Dinges als Naturzwecks",
    "§ 81. Von der Beigesellung des Mechanismus zum teleologischen Prinzip in der Erklärung eines Naturzwecks als Naturprodukts",
    "§ 82. Von dem teleologischen System in den äußern Verhältnissen organisierter Wesen",
    "§ 83. Von dem letzten Zwecke der Natur als eines teleologischen Systems",
    "§ 84. Von dem Endzwecke des Daseins einer Welt, d. i. der Schöpfung selbst",
    "§ 85. Von der Physikotheologie",
    "§ 86. Von der Ethikotheologie",
    "§ 87. Von dem moralischen Beweise des Daseins Gottes",
    "§ 88. Beschränkung der Gültigkeit des moralischen Beweises",
    "§ 89. Von dem Nutzen des moralischen Arguments",
    "§ 90. Von der Art des Fürwahrhaltens in einem teleologischen Beweise des Daseins Gottes",
    "§ 91. Von der Art des Fürwahrhaltens durch einen praktischen Glauben",
    "Allgemeine Anmerkung zur Teleologie",
];

/// Flat TOC entries with modernized labels, reusing `toc`'s structural fields.
/// Each entry: (index, aa_page, depth, modernized_label, slug_override).
pub fn flat_toc_entries() -> Vec<(usize, u16, u16, &'static str, Option<&'static str>)> {
    toc::flat_toc_entries()
        .into_iter()
        .map(|(idx, aa_page, depth, _label, slug_override)| {
            (idx, aa_page, depth, MODERNIZED_LABELS[idx], slug_override)
        })
        .collect()
}

/// Return the total number of TOC entries.
pub fn toc_len() -> usize {
    MODERNIZED_LABELS.len()
}

#[cfg(test)]
mod tests {
    use super::super::filenames::slugify;
    use super::*;

    #[test]
    fn test_label_count_matches_toc() {
        assert_eq!(MODERNIZED_LABELS.len(), toc::toc_len());
    }

    #[test]
    fn test_structural_parity_with_toc() {
        // Only labels may differ; aa_page/depth/slug_override must match toc.
        for ((i, aa, d, _, s), (i2, aa2, d2, _, s2)) in
            flat_toc_entries().into_iter().zip(toc::flat_toc_entries())
        {
            assert_eq!((i, aa, d, s), (i2, aa2, d2, s2));
        }
    }

    #[test]
    fn test_labels_modernized() {
        let mod_entries = flat_toc_entries();
        assert_eq!(
            mod_entries[12].3,
            "Erster Teil. Kritik der ästhetischen Urteilskraft"
        );
        assert_eq!(
            mod_entries[16].3,
            "§ 1. Das Geschmacksurteil ist ästhetisch"
        );
        // No "Theil"/"Urtheil"/"Princip" leakage in modernized labels.
        for (_, _, _, label, _) in &mod_entries {
            assert!(!label.contains("Theil"), "stale Theil in {label:?}");
            assert!(!label.contains("Urtheil"), "stale Urtheil in {label:?}");
            assert!(!label.contains("Princip"), "stale Princip in {label:?}");
        }
    }

    /// Modernized sibling slugs must also be unique (node slug/path use these).
    #[test]
    fn test_modernized_sibling_slugs_unique() {
        use std::collections::HashMap;
        let entries = flat_toc_entries();
        let parent_of = |i: usize| -> Option<usize> {
            let target = entries[i].2.checked_sub(1)?;
            if target == 0 {
                return None;
            }
            for j in (0..i).rev() {
                if entries[j].2 == target {
                    return Some(j);
                }
                if entries[j].2 < target {
                    break;
                }
            }
            None
        };
        let mut seen: HashMap<(Option<usize>, String), usize> = HashMap::new();
        for (i, (_, _, _, label, slug_override)) in entries.iter().enumerate() {
            let slug = slug_override
                .map(|s| s.to_string())
                .unwrap_or_else(|| slugify(label));
            if let Some(prev) = seen.insert((parent_of(i), slug.clone()), i) {
                panic!("modernized sibling slug collision: {prev} and {i} ({slug:?})");
            }
        }
    }
}
