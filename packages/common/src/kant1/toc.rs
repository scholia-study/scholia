// ---------------------------------------------------------------------------
// Authoritative TOC for Kant's KrV B-edition (Akademie-Ausgabe Band III)
// ---------------------------------------------------------------------------
//
// Each entry: (aa_page, depth, label)
// Derived from the Akademie-Ausgabe Band III table of contents,
// cross-referenced with scholarly sources.

struct FlatEntry {
    aa_page: u16,
    depth: u16,
    label: &'static str,
    slug_override: Option<&'static str>,
}

const TOC: &[FlatEntry] = &[
    // -----------------------------------------------------------------------
    // Front matter
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 2,
        depth: 1,
        label: "Motto",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 3,
        depth: 1,
        label: "Zueignung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 7,
        depth: 1,
        label: "Vorrede zur zweiten Auflage",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // Einleitung
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 27,
        depth: 1,
        label: "Einleitung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 27,
        depth: 2,
        label: "I. Von dem Unterschiede der reinen und empirischen Erkenntniß",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 28,
        depth: 2,
        label: "II. Wir sind im Besitze gewisser Erkenntnisse _a priori_, und selbst der gemeine Verstand ist niemals ohne solche",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 30,
        depth: 2,
        label: "III. Die Philosophie bedarf einer Wissenschaft, welche die Möglichkeit, die Principien und den Umfang aller Erkenntnisse _a priori_ bestimme",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 33,
        depth: 2,
        label: "IV. Von dem Unterschiede analytischer und synthetischer Urtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 36,
        depth: 2,
        label: "V. In allen theoretischen Wissenschaften der Vernunft sind synthetische Urtheile _a priori_ als Principien enthalten",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 39,
        depth: 2,
        label: "VI. Allgemeine Aufgabe der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 42,
        depth: 2,
        label: "VII. Idee und Eintheilung einer besonderen Wissenschaft unter dem Namen einer Kritik der reinen Vernunft",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // I. Transscendentale Elementarlehre
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 49,
        depth: 1,
        label: "I. Transscendentale Elementarlehre",
        slug_override: None,
    },
    // -- Erster Theil: Transscendentale Ästhetik --
    FlatEntry {
        aa_page: 49,
        depth: 2,
        label: "Erster Theil. Die transscendentale Ästhetik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 49,
        depth: 3,
        label: "Einleitung",
        slug_override: Some("einleitung-aesthetik"),
    },
    FlatEntry {
        aa_page: 51,
        depth: 3,
        label: "1. Abschnitt. Von dem Raume",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 57,
        depth: 3,
        label: "2. Abschnitt. Von der Zeit",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 65,
        depth: 3,
        label: "Allgemeine Anmerkungen zur transscendentalen Ästhetik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 73,
        depth: 3,
        label: "Beschluß der transscendentalen Ästhetik",
        slug_override: None,
    },
    // -- Zweiter Theil: Transscendentale Logik --
    FlatEntry {
        aa_page: 74,
        depth: 2,
        label: "Zweiter Theil. Die transscendentale Logik",
        slug_override: None,
    },
    // Einleitung zur transscendentalen Logik
    FlatEntry {
        aa_page: 74,
        depth: 3,
        label: "Einleitung. Idee einer transscendentalen Logik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 74,
        depth: 4,
        label: "I. Von der Logik überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 77,
        depth: 4,
        label: "II. Von der transscendentalen Logik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 79,
        depth: 4,
        label: "III. Von der Eintheilung der allgemeinen Logik in Analytik und Dialektik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 81,
        depth: 4,
        label: "IV. Von der Eintheilung der transscendentalen Logik in die transscendentale Analytik und Dialektik",
        slug_override: None,
    },
    // -- Erste Abtheilung: Transscendentale Analytik --
    FlatEntry {
        aa_page: 83,
        depth: 3,
        label: "Erste Abtheilung. Die transscendentale Analytik",
        slug_override: None,
    },
    // Erstes Buch: Analytik der Begriffe
    FlatEntry {
        aa_page: 83,
        depth: 4,
        label: "Erstes Buch. Die Analytik der Begriffe",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 84,
        depth: 5,
        label: "1. Hauptstück. Von dem Leitfaden der Entdeckung aller reinen Verstandesbegriffe",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 85,
        depth: 6,
        label: "1. Abschnitt. Von dem logischen Verstandesgebrauche überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 86,
        depth: 6,
        label: "2. Abschnitt. Von der logischen Function des Verstandes in Urtheilen. \u{00A7}9",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 90,
        depth: 6,
        label: "3. Abschnitt. Von den reinen Verstandesbegriffen oder Kategorien. \u{00A7}10\u{2013}12",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 99,
        depth: 5,
        label: "2. Hauptstück. Von der Deduction der reinen Verstandesbegriffe",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 99,
        depth: 6,
        label: "1. Abschnitt. Von den Principien einer transscendentalen Deduction überhaupt. \u{00A7}13",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 104,
        depth: 6,
        label: "Übergang zur transscendentalen Deduction der Kategorien. \u{00A7}14",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 107,
        depth: 6,
        label: "2. Abschnitt. Transscendentale Deduction der reinen Verstandesbegriffe. \u{00A7}15\u{2013}27",
        slug_override: None,
    },
    // Zweites Buch: Analytik der Grundsätze
    FlatEntry {
        aa_page: 130,
        depth: 4,
        label: "Zweites Buch. Die Analytik der Grundsätze",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 131,
        depth: 5,
        label: "Einleitung. Von der transscendentalen Urtheilskraft überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 133,
        depth: 5,
        label: "1. Hauptstück. Von dem Schematismus der reinen Verstandesbegriffe",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 140,
        depth: 5,
        label: "2. Hauptstück. System aller Grundsätze des reinen Verstandes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 141,
        depth: 6,
        label: "1. Abschnitt. Von dem obersten Grundsatze aller analytischen Urtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 143,
        depth: 6,
        label: "2. Abschnitt. Von dem obersten Grundsatze aller synthetischen Urtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 146,
        depth: 6,
        label: "3. Abschnitt. Systematische Vorstellung aller synthetischen Grundsätze des reinen Verstandes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 148,
        depth: 7,
        label: "1. Axiome der Anschauung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 151,
        depth: 7,
        label: "2. Anticipationen der Wahrnehmung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 158,
        depth: 7,
        label: "3. Analogien der Erfahrung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 162,
        depth: 8,
        label: "Erste Analogie. Grundsatz der Beharrlichkeit der Substanz",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 166,
        depth: 8,
        label: "Zweite Analogie. Grundsatz der Zeitfolge nach dem Gesetze der Causalität",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 180,
        depth: 8,
        label: "Dritte Analogie. Grundsatz des Zugleichseins nach dem Gesetze der Wechselwirkung oder Gemeinschaft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 185,
        depth: 7,
        label: "4. Die Postulate des empirischen Denkens überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 198,
        depth: 7,
        label: "Allgemeine Anmerkung zum System der Grundsätze",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 202,
        depth: 5,
        label: "3. Hauptstück. Von dem Grunde der Unterscheidung aller Gegenstände überhaupt in Phaenomena und Noumena",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 214,
        depth: 5,
        label: "Anhang. Von der Amphibolie der Reflexionsbegriffe",
        slug_override: None,
    },
    // -- Zweite Abtheilung: Transscendentale Dialektik --
    FlatEntry {
        aa_page: 234,
        depth: 3,
        label: "Zweite Abtheilung. Die transscendentale Dialektik",
        slug_override: None,
    },
    // Einleitung zur Dialektik
    FlatEntry {
        aa_page: 234,
        depth: 4,
        label: "Einleitung",
        slug_override: Some("einleitung-dialektik"),
    },
    FlatEntry {
        aa_page: 234,
        depth: 5,
        label: "I. Vom transscendentalen Schein",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 237,
        depth: 5,
        label: "II. Von der reinen Vernunft als dem Sitze des transscendentalen Scheins",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 237,
        depth: 6,
        label: "A. Von der Vernunft überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 240,
        depth: 6,
        label: "B. Vom logischen Gebrauche der Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 241,
        depth: 6,
        label: "C. Von dem reinen Gebrauche der Vernunft",
        slug_override: None,
    },
    // Erstes Buch der Dialektik
    FlatEntry {
        aa_page: 244,
        depth: 4,
        label: "Erstes Buch. Von den Begriffen der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 245,
        depth: 5,
        label: "1. Abschnitt. Von den Ideen überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 250,
        depth: 5,
        label: "2. Abschnitt. Von den transscendentalen Ideen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 257,
        depth: 5,
        label: "3. Abschnitt. System der transscendentalen Ideen",
        slug_override: None,
    },
    // Zweites Buch der Dialektik
    FlatEntry {
        aa_page: 261,
        depth: 4,
        label: "Zweites Buch. Von den dialektischen Schlüssen der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 262,
        depth: 5,
        label: "1. Hauptstück. Von den Paralogismen der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 279,
        depth: 5,
        label: "Allgemeine Anmerkung, den Übergang von der rationalen Psychologie zur Kosmologie betreffend",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 281,
        depth: 5,
        label: "2. Hauptstück. Die Antinomie der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 283,
        depth: 6,
        label: "1. Abschnitt. System der kosmologischen Ideen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 290,
        depth: 6,
        label: "2. Abschnitt. Antithetik der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 294,
        depth: 7,
        label: "Erste Antinomie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 300,
        depth: 7,
        label: "Zweite Antinomie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 308,
        depth: 7,
        label: "Dritte Antinomie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 314,
        depth: 7,
        label: "Vierte Antinomie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 322,
        depth: 6,
        label: "3. Abschnitt. Von dem Interesse der Vernunft bei diesem ihrem Widerstreite",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 330,
        depth: 6,
        label: "4. Abschnitt. Von den transscendentalen Aufgaben der reinen Vernunft, in so fern sie schlechterdings müssen aufgelöset werden können",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 335,
        depth: 6,
        label: "5. Abschnitt. Sceptische Vorstellung der kosmologischen Fragen durch alle vier transscendentalen Ideen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 338,
        depth: 6,
        label: "6. Abschnitt. Der transscendentale Idealismus als der Schlüssel zu Auflösung der kosmologischen Dialektik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 342,
        depth: 6,
        label: "7. Abschnitt. Kritische Entscheidung des kosmologischen Streits der Vernunft mit sich selbst",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 348,
        depth: 6,
        label: "8. Abschnitt. Regulatives Princip der reinen Vernunft in Ansehung der kosmologischen Ideen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 353,
        depth: 6,
        label: "9. Abschnitt. Von dem empirischen Gebrauche des regulativen Princips der Vernunft in Ansehung aller kosmologischen Ideen",
        slug_override: None,
    },
    // Note: consider changing the depth of the comments after the numbered sections here?
    FlatEntry {
        aa_page: 354,
        depth: 7,
        label: "I. Auflösung der kosmologischen Idee von der Totalität der Zusammensetzung der Erscheinungen von einem Weltganzen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 357,
        depth: 7,
        label: "II. Auflösung der kosmologischen Idee von der Totalität der Teilung eines gegebenen Ganzen in der Anschauung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 360,
        depth: 8,
        label: "Schlußanmerkung zur Auflösung der mathematischtranszendentalen, und Vorerinnerung zur Auflösung der dynamisch-transzendentalen Ideen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 362,
        depth: 7,
        label: "III. Auflösung der kosmologischen Ideen von der Totalität der Ableitung der Weltbegebenheiten aus ihren Ursachen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 366,
        depth: 8,
        label: "Möglichkeit der Kausalität durch Freiheit, in Vereinigung mit dem allgemeinen Gesetze der Naturnotwendigkeit",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 368,
        depth: 8,
        label: "Erläuterung der kosmologischen Idee einer Freiheit in Verbindung mit der allgemeinen Naturnotwendigkeit",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 378,
        depth: 7,
        label: "IV. Auflösung der kosmologischen Idee von der Totalität der Abhängigkeit der Erscheinungen, ihrem Dasein nach überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 381,
        depth: 7,
        label: "Schlußanmerkung zur ganzen Antinomie der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 383,
        depth: 5,
        label: "3. Hauptstück. Das Ideal der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 383,
        depth: 6,
        label: "1. Abschnitt. Von dem Ideal überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 385,
        depth: 6,
        label: "2. Abschnitt. Von dem transscendentalen Ideal (Prototypon transscendentale)",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 392,
        depth: 6,
        label: "3. Abschnitt. Von den Beweisgründen der speculativen Vernunft auf das Dasein eines höchsten Wesens zu schließen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 397,
        depth: 6,
        label: "4. Abschnitt. Von der Unmöglichkeit eines ontologischen Beweises vom Dasein Gottes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 403,
        depth: 6,
        label: "5. Abschnitt. Von der Unmöglichkeit eines kosmologischen Beweises vom Dasein Gottes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 410,
        depth: 7,
        label: "Entdeckung und Erklärung des dialektischen Scheins in allen transzendentalen Beweisen vom Dasein eines notwendigen Wesens",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 413,
        depth: 6,
        label: "6. Abschnitt. Von der Unmöglichkeit des physikotheologischen Beweises",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 420,
        depth: 6,
        label: "7. Abschnitt. Kritik aller Theologie aus speculativen Principien der Vernunft",
        slug_override: None,
    },
    // Anhang zur transscendentalen Dialektik
    FlatEntry {
        aa_page: 426,
        depth: 6,
        label: "Anhang zur transscendentalen Dialektik",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 426,
        depth: 7,
        label: "Von dem regulativen Gebrauch der Ideen der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 442,
        depth: 7,
        label: "Von der Endabsicht der natürlichen Dialektik der menschlichen Vernunft",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // II. Transscendentale Methodenlehre
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 463,
        depth: 1,
        label: "II. Transscendentale Methodenlehre",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 465,
        depth: 2,
        label: "Einleitung",
        slug_override: Some("einleitung-methodenlehre"),
    },
    FlatEntry {
        aa_page: 466,
        depth: 2,
        label: "Erstes Hauptstück. Die Disciplin der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 468,
        depth: 3,
        label: "1. Abschnitt. Die Disciplin der reinen Vernunft im dogmatischen Gebrauche",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 484,
        depth: 3,
        label: "2. Abschnitt. Die Disciplin der reinen Vernunft in Ansehung ihres polemischen Gebrauchs",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 495,
        depth: 4,
        label: "Von der Unmöglichkeit einer skeptischen Befriedigung der mit sich selbst veruneinigten reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 502,
        depth: 3,
        label: "3. Abschnitt. Die Disciplin der reinen Vernunft in Ansehung der Hypothesen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 509,
        depth: 3,
        label: "4. Abschnitt. Die Disciplin der reinen Vernunft in Ansehung ihrer Beweise",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 517,
        depth: 2,
        label: "Zweites Hauptstück. Der Kanon der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 518,
        depth: 3,
        label: "1. Abschnitt. Von dem letzten Zwecke des reinen Gebrauchs unserer Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 522,
        depth: 3,
        label: "2. Abschnitt. Von dem Ideal des höchsten Guts als einem Bestimmungsgrunde des letzten Zwecks der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 531,
        depth: 3,
        label: "3. Abschnitt. Vom Meinen, Wissen und Glauben",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 538,
        depth: 2,
        label: "Drittes Hauptstück. Die Architektonik der reinen Vernunft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 550,
        depth: 2,
        label: "Viertes Hauptstück. Die Geschichte der reinen Vernunft",
        slug_override: None,
    },
];

/// Return the flat TOC entries in document order for content assignment.
/// Each entry: (index_in_flat_list, aa_page, depth, label, slug_override)
pub fn flat_toc_entries() -> Vec<(usize, u16, u16, &'static str, Option<&'static str>)> {
    TOC.iter()
        .enumerate()
        .map(|(i, e)| (i, e.aa_page, e.depth, e.label, e.slug_override))
        .collect()
}

/// Return the total number of TOC entries.
pub fn toc_len() -> usize {
    TOC.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_entry_count() {
        let flat = flat_toc_entries();
        assert_eq!(flat.len(), TOC.len());
        assert_eq!(flat[0].3, "Motto");
        assert_eq!(
            flat.last().unwrap().3,
            "Viertes Hauptstück. Die Geschichte der reinen Vernunft"
        );
    }

    #[test]
    fn test_toc_len() {
        assert_eq!(toc_len(), 113);
    }

    #[test]
    fn test_front_matter_entries() {
        let flat = flat_toc_entries();
        assert_eq!(flat[0], (0, 2, 1, "Motto", None));
        assert_eq!(flat[1], (1, 3, 1, "Zueignung", None));
        assert_eq!(flat[2], (2, 7, 1, "Vorrede zur zweiten Auflage", None));
    }
}
