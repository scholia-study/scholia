//! Modernized-orthography TOC labels for hegel3, one per `toc` entry, in
//! document order — the hegel1 `toc_mod` pattern. The labels are the
//! converter's own modernized head renderings (rulings-table driven), so a
//! file's front-matter label in `md_modernized/` matches its entry here
//! byte for byte.

/// Modernized labels, one per `toc` entry, in document order.
pub const MODERNIZED_LABELS: &[&str] = &[
    "Vorrede",
    "Einleitung",
    "Logik",
    "Über die allgemeine Einteilung derselben",
    "Erstes Buch. Das Sein",
    "Womit muß der Anfang der Wissenschaft gemacht werden?",
    "Allgemeine Einteilung des Seins",
    "Erster Abschnitt. Bestimmtheit",
    "Erstes Kapitel. Sein",
    "A",
    "B. Nichts",
    "C. Werden",
    "Einheit des Seins und Nichts",
    "Anmerkung 1",
    "Anmerkung 2",
    "Anmerkung 3",
    "Anmerkung 4",
    "2. Momente des Werdens",
    "3. Aufheben des Werdens",
    "Anmerkung",
    "Zweites Kapitel. Das Dasein",
    "A. Dasein als solches",
    "1. Dasein überhaupt",
    "2. Realität",
    "a) Anderssein",
    "b) Sein-für-Anderes und Ansichsein",
    "c) Realität",
    "Anmerkung",
    "3. Etwas",
    "B. Bestimmtheit",
    "1. Grenze",
    "2. Bestimmtheit",
    "a.) Bestimmung",
    "b.) Beschaffenheit",
    "c.) Qualität",
    "Anmerkung",
    "3. Veränderung",
    "a) Veränderung der Beschaffenheit",
    "b.) Sollen und Schranke",
    "Anmerkung",
    "c.) Negation",
    "Anmerkung",
    "C. (Qualitative) Unendlichkeit",
    "1. Endlichkeit und Unendlichkeit",
    "2. Wechselbestimmung des Endlichen und Unendlichen",
    "3. Rückkehr der Unendlichkeit in sich",
    "Anmerkung",
    "Drittes Kapitel. Das Fürsichsein",
    "A. Fürsichsein als solches",
    "1. Fürsichsein überhaupt",
    "2. Die Momente des Fürsichseins",
    "a.) das Moment seines Ansichseins,",
    "b.) Für eines sein",
    "Anmerkung",
    "c.) Idealität",
    "3. Werden des Eins",
    "B. Das Eins",
    "1. Das Eins und das Leere",
    "Anmerkung",
    "2. Viele Eins",
    "Anmerkung",
    "3. Gegenseitige Repulsion",
    "C. Attraktion",
    "1. Ein Eins",
    "2. Gleichgewicht der Attraktion und Repulsion",
    "Anmerkung",
    "3. Übergang zur Quantität",
    "Zweiter Abschnitt. Größe",
    "Anmerkung",
    "Erstes Kapitel. Die Quantität",
    "A. Die reine Quantität",
    "Anmerkung 1",
    "Anmerkung 2",
    "B. Kontinuierliche und discrete Größe",
    "Anmerkung",
    "C. Begrenzung der Quantität",
    "Zweites Kapitel. Quantum",
    "A. Die Zahl",
    "Anmerkung 1",
    "Anmerkung 2",
    "B. Extensives und intensives Quantum",
    "1. Unterschied derselben",
    "2. Identität der extensiven und intensiven Größe",
    "Anmerkung",
    "3. Veränderung des Quantums",
    "C. Quantitative Unendlichkeit",
    "1. Begriff derselben",
    "2. Der unendliche Progreß",
    "Anmerkung 1",
    "Anmerkung 2",
    "3. Unendlichkeit des Quantums",
    "Anmerkung",
    "Drittes Kapitel. Das quantitative Verhältnis",
    "A. Das direkte Verhältnis",
    "B. Das umgekehrte Verhältnis",
    "C. Potenzenverhältnis",
    "Anmerkung",
    "Dritter Abschnitt. Das Maß",
    "Erstes Kapitel. Die spezifische Quantität",
    "A. Das spezifische Quantum",
    "B. Die Regel",
    "1. Die qualitative und quantitative Größen-Bestimmtheit",
    "2. Qualität und Quantum",
    "3. Unterscheidung beider Seiten als Qualitäten",
    "Anmerkung",
    "C. Verhältnis von Qualitäten",
    "Zweites Kapitel. Verhältnis selbstständiger Masse",
    "A. Das Verhältnis selbstständiger Masse",
    "1. Neutralität",
    "2. Spezifikation der Neutralität",
    "3. Wahlverwandtschaft",
    "Anmerkung",
    "B. Knotenlinie von Maßverhältnissen",
    "Anmerkung",
    "C. Das Maßlose",
    "Drittes Kapitel. Das Werden des Wesens",
    "A. Die Indifferenz",
    "B. Das Selbstständige als umgekehrtes Verhältnis seiner Faktoren",
    "Anmerkung",
    "C. Hervorgehen des Wesens",
];

pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    super::toc::entries()
        .iter()
        .enumerate()
        .map(|(i, e)| {
            (
                i,
                e.page.map(str::to_string),
                e.depth,
                MODERNIZED_LABELS[i],
                Some(e.slug),
            )
        })
        .collect()
}

pub fn label(index: usize) -> &'static str {
    MODERNIZED_LABELS[index]
}

#[cfg(test)]
mod tests {
    #[test]
    fn one_label_per_toc_entry() {
        assert_eq!(super::MODERNIZED_LABELS.len(), super::super::toc::toc_len());
    }

    #[test]
    fn no_pre_reform_spellings_survive() {
        for l in super::MODERNIZED_LABELS {
            for bad in ["Seyn", "seyn", "Theil", "theilung", "ſ", "Maaß"] {
                assert!(!l.contains(bad), "unmodernized {bad:?} in {l:?}");
            }
        }
    }
}
