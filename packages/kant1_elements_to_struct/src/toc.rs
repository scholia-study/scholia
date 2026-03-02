use crate::model::KantTocNode;

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
}

const TOC: &[FlatEntry] = &[
    // -----------------------------------------------------------------------
    // Front matter
    // -----------------------------------------------------------------------
    FlatEntry { aa_page: 1,   depth: 1, label: "Zueignung" },
    FlatEntry { aa_page: 3,   depth: 1, label: "Vorrede zur zweiten Auflage" },

    // -----------------------------------------------------------------------
    // Einleitung
    // -----------------------------------------------------------------------
    FlatEntry { aa_page: 27,  depth: 1, label: "Einleitung" },
    FlatEntry { aa_page: 27,  depth: 2, label: "I. Von dem Unterschiede der reinen und empirischen Erkenntniß" },
    FlatEntry { aa_page: 28,  depth: 2, label: "II. Wir sind im Besitze gewisser Erkenntnisse a priori, und selbst der gemeine Verstand ist niemals ohne solche" },
    FlatEntry { aa_page: 28,  depth: 2, label: "III. Die Philosophie bedarf einer Wissenschaft, welche die Möglichkeit, die Principien und den Umfang aller Erkenntnisse a priori bestimme" },
    FlatEntry { aa_page: 33,  depth: 2, label: "IV. Von dem Unterschiede analytischer und synthetischer Urtheile" },
    FlatEntry { aa_page: 36,  depth: 2, label: "V. In allen theoretischen Wissenschaften der Vernunft sind synthetische Urtheile a priori als Principien enthalten" },
    FlatEntry { aa_page: 39,  depth: 2, label: "VI. Allgemeine Aufgabe der reinen Vernunft" },
    FlatEntry { aa_page: 42,  depth: 2, label: "VII. Idee und Eintheilung einer besonderen Wissenschaft unter dem Namen einer Kritik der reinen Vernunft" },

    // -----------------------------------------------------------------------
    // I. Transscendentale Elementarlehre
    // -----------------------------------------------------------------------
    FlatEntry { aa_page: 49,  depth: 1, label: "I. Transscendentale Elementarlehre" },

    // -- Erster Theil: Transscendentale Ästhetik --
    FlatEntry { aa_page: 49,  depth: 2, label: "Erster Theil. Die transscendentale Ästhetik" },
    FlatEntry { aa_page: 49,  depth: 3, label: "\u{00A7}1" },
    FlatEntry { aa_page: 49,  depth: 3, label: "1. Abschnitt. Von dem Raume" },
    FlatEntry { aa_page: 51,  depth: 3, label: "2. Abschnitt. Von der Zeit" },
    FlatEntry { aa_page: 57,  depth: 3, label: "Allgemeine Anmerkungen zur transscendentalen Ästhetik" },
    FlatEntry { aa_page: 65,  depth: 3, label: "Beschluß der transscendentalen Ästhetik" },

    // -- Zweiter Theil: Transscendentale Logik --
    FlatEntry { aa_page: 73,  depth: 2, label: "Zweiter Theil. Die transscendentale Logik" },

    // Einleitung zur transscendentalen Logik
    FlatEntry { aa_page: 74,  depth: 3, label: "Einleitung. Idee einer transscendentalen Logik" },
    FlatEntry { aa_page: 74,  depth: 4, label: "I. Von der Logik überhaupt" },
    FlatEntry { aa_page: 77,  depth: 4, label: "II. Von der transscendentalen Logik" },
    FlatEntry { aa_page: 79,  depth: 4, label: "III. Von der Eintheilung der allgemeinen Logik in Analytik und Dialektik" },
    FlatEntry { aa_page: 81,  depth: 4, label: "IV. Von der Eintheilung der transscendentalen Logik in die transscendentale Analytik und Dialektik" },

    // -- Erste Abtheilung: Transscendentale Analytik --
    FlatEntry { aa_page: 83,  depth: 3, label: "Erste Abtheilung. Die transscendentale Analytik" },

    // Erstes Buch: Analytik der Begriffe
    FlatEntry { aa_page: 83,  depth: 4, label: "Erstes Buch. Die Analytik der Begriffe" },
    FlatEntry { aa_page: 83,  depth: 5, label: "1. Hauptstück. Von dem Leitfaden der Entdeckung aller reinen Verstandesbegriffe" },
    FlatEntry { aa_page: 84,  depth: 6, label: "1. Abschnitt. Von dem logischen Verstandesgebrauche überhaupt" },
    FlatEntry { aa_page: 85,  depth: 6, label: "2. Abschnitt. Von der logischen Function des Verstandes in Urtheilen. \u{00A7}9" },
    FlatEntry { aa_page: 86,  depth: 6, label: "3. Abschnitt. Von den reinen Verstandesbegriffen oder Kategorien. \u{00A7}10\u{2013}12" },
    FlatEntry { aa_page: 90,  depth: 5, label: "2. Hauptstück. Von der Deduction der reinen Verstandesbegriffe" },
    FlatEntry { aa_page: 99,  depth: 6, label: "1. Abschnitt. Von den Principien einer transscendentalen Deduction überhaupt. \u{00A7}13" },
    FlatEntry { aa_page: 99,  depth: 6, label: "Übergang zur transscendentalen Deduction der Kategorien. \u{00A7}14" },
    FlatEntry { aa_page: 104, depth: 6, label: "2. Abschnitt. Transscendentale Deduction der reinen Verstandesbegriffe. \u{00A7}15\u{2013}27" },

    // Zweites Buch: Analytik der Grundsätze
    FlatEntry { aa_page: 107, depth: 4, label: "Zweites Buch. Die Analytik der Grundsätze" },
    FlatEntry { aa_page: 130, depth: 5, label: "Einleitung. Von der transscendentalen Urtheilskraft überhaupt" },
    FlatEntry { aa_page: 131, depth: 5, label: "1. Hauptstück. Von dem Schematismus der reinen Verstandesbegriffe" },
    FlatEntry { aa_page: 133, depth: 5, label: "2. Hauptstück. System aller Grundsätze des reinen Verstandes" },
    FlatEntry { aa_page: 140, depth: 6, label: "1. Abschnitt. Von dem obersten Grundsatze aller analytischen Urtheile" },
    FlatEntry { aa_page: 141, depth: 6, label: "2. Abschnitt. Von dem obersten Grundsatze aller synthetischen Urtheile" },
    FlatEntry { aa_page: 143, depth: 6, label: "3. Abschnitt. Systematische Vorstellung aller synthetischen Grundsätze des reinen Verstandes" },
    FlatEntry { aa_page: 146, depth: 7, label: "1. Axiome der Anschauung" },
    FlatEntry { aa_page: 148, depth: 7, label: "2. Anticipationen der Wahrnehmung" },
    FlatEntry { aa_page: 151, depth: 7, label: "3. Analogien der Erfahrung" },
    FlatEntry { aa_page: 158, depth: 8, label: "Erste Analogie. Grundsatz der Beharrlichkeit der Substanz" },
    FlatEntry { aa_page: 162, depth: 8, label: "Zweite Analogie. Grundsatz der Zeitfolge nach dem Gesetze der Causalität" },
    FlatEntry { aa_page: 166, depth: 8, label: "Dritte Analogie. Grundsatz des Zugleichseins nach dem Gesetze der Wechselwirkung oder Gemeinschaft" },
    FlatEntry { aa_page: 183, depth: 7, label: "4. Die Postulate des empirischen Denkens überhaupt" },
    FlatEntry { aa_page: 185, depth: 6, label: "Allgemeine Anmerkung zum System der Grundsätze" },
    FlatEntry { aa_page: 198, depth: 5, label: "3. Hauptstück. Von dem Grunde der Unterscheidung aller Gegenstände überhaupt in Phaenomena und Noumena" },
    FlatEntry { aa_page: 202, depth: 5, label: "Anhang. Von der Amphibolie der Reflexionsbegriffe" },

    // -- Zweite Abtheilung: Transscendentale Dialektik --
    FlatEntry { aa_page: 234, depth: 3, label: "Zweite Abtheilung. Die transscendentale Dialektik" },

    // Einleitung zur Dialektik
    FlatEntry { aa_page: 234, depth: 4, label: "Einleitung" },
    FlatEntry { aa_page: 234, depth: 5, label: "I. Vom transscendentalen Schein" },
    FlatEntry { aa_page: 234, depth: 5, label: "II. Von der reinen Vernunft als dem Sitze des transscendentalen Scheins" },

    // Erstes Buch der Dialektik
    FlatEntry { aa_page: 241, depth: 4, label: "Erstes Buch. Von den Begriffen der reinen Vernunft" },
    FlatEntry { aa_page: 244, depth: 5, label: "1. Abschnitt. Von den Ideen überhaupt" },
    FlatEntry { aa_page: 245, depth: 5, label: "2. Abschnitt. Von den transscendentalen Ideen" },
    FlatEntry { aa_page: 250, depth: 5, label: "3. Abschnitt. System der transscendentalen Ideen" },

    // Zweites Buch der Dialektik
    FlatEntry { aa_page: 257, depth: 4, label: "Zweites Buch. Von den dialektischen Schlüssen der reinen Vernunft" },
    FlatEntry { aa_page: 261, depth: 5, label: "1. Hauptstück. Von den Paralogismen der reinen Vernunft" },
    FlatEntry { aa_page: 279, depth: 5, label: "2. Hauptstück. Die Antinomie der reinen Vernunft" },
    FlatEntry { aa_page: 281, depth: 6, label: "1. Abschnitt. System der kosmologischen Ideen" },
    FlatEntry { aa_page: 283, depth: 6, label: "2. Abschnitt. Antithetik der reinen Vernunft" },
    FlatEntry { aa_page: 314, depth: 6, label: "3. Abschnitt. Von dem Interesse der Vernunft bei diesem ihrem Widerstreite" },
    FlatEntry { aa_page: 322, depth: 6, label: "4. Abschnitt. Von den transscendentalen Aufgaben der reinen Vernunft, in so fern sie schlechterdings müssen aufgelöset werden können" },
    FlatEntry { aa_page: 330, depth: 6, label: "5. Abschnitt. Sceptische Vorstellung der kosmologischen Fragen durch alle vier transscendentalen Ideen" },
    FlatEntry { aa_page: 335, depth: 6, label: "6. Abschnitt. Der transscendentale Idealismus als der Schlüssel zu Auflösung der kosmologischen Dialektik" },
    FlatEntry { aa_page: 338, depth: 6, label: "7. Abschnitt. Kritische Entscheidung des kosmologischen Streits der Vernunft mit sich selbst" },
    FlatEntry { aa_page: 342, depth: 6, label: "8. Abschnitt. Regulatives Princip der reinen Vernunft in Ansehung der kosmologischen Ideen" },
    FlatEntry { aa_page: 348, depth: 6, label: "9. Abschnitt. Von dem empirischen Gebrauche des regulativen Princips der Vernunft in Ansehung aller kosmologischen Ideen" },
    FlatEntry { aa_page: 381, depth: 5, label: "3. Hauptstück. Das Ideal der reinen Vernunft" },
    FlatEntry { aa_page: 383, depth: 6, label: "1. Abschnitt. Von dem Ideal überhaupt" },
    FlatEntry { aa_page: 383, depth: 6, label: "2. Abschnitt. Von dem transscendentalen Ideal (Prototypon transscendentale)" },
    FlatEntry { aa_page: 385, depth: 6, label: "3. Abschnitt. Von den Beweisgründen der speculativen Vernunft auf das Dasein eines höchsten Wesens zu schließen" },
    FlatEntry { aa_page: 392, depth: 6, label: "4. Abschnitt. Von der Unmöglichkeit eines ontologischen Beweises vom Dasein Gottes" },
    FlatEntry { aa_page: 397, depth: 6, label: "5. Abschnitt. Von der Unmöglichkeit eines kosmologischen Beweises vom Dasein Gottes" },
    FlatEntry { aa_page: 410, depth: 6, label: "6. Abschnitt. Von der Unmöglichkeit des physikotheologischen Beweises" },
    FlatEntry { aa_page: 413, depth: 6, label: "7. Abschnitt. Kritik aller Theologie aus speculativen Principien der Vernunft" },

    // Anhang zur transscendentalen Dialektik
    FlatEntry { aa_page: 420, depth: 4, label: "Anhang zur transscendentalen Dialektik" },

    // -----------------------------------------------------------------------
    // II. Transscendentale Methodenlehre
    // -----------------------------------------------------------------------
    FlatEntry { aa_page: 442, depth: 1, label: "II. Transscendentale Methodenlehre" },

    FlatEntry { aa_page: 465, depth: 2, label: "Erstes Hauptstück. Die Disciplin der reinen Vernunft" },
    FlatEntry { aa_page: 466, depth: 3, label: "1. Abschnitt. Die Disciplin der reinen Vernunft im dogmatischen Gebrauche" },
    FlatEntry { aa_page: 468, depth: 3, label: "2. Abschnitt. Die Disciplin der reinen Vernunft in Ansehung ihres polemischen Gebrauchs" },
    FlatEntry { aa_page: 495, depth: 3, label: "3. Abschnitt. Die Disciplin der reinen Vernunft in Ansehung der Hypothesen" },
    FlatEntry { aa_page: 502, depth: 3, label: "4. Abschnitt. Die Disciplin der reinen Vernunft in Ansehung ihrer Beweise" },

    FlatEntry { aa_page: 517, depth: 2, label: "Zweites Hauptstück. Der Kanon der reinen Vernunft" },
    FlatEntry { aa_page: 517, depth: 3, label: "1. Abschnitt. Von dem letzten Zwecke des reinen Gebrauchs unserer Vernunft" },
    FlatEntry { aa_page: 522, depth: 3, label: "2. Abschnitt. Von dem Ideal des höchsten Guts als einem Bestimmungsgrunde des letzten Zwecks der reinen Vernunft" },
    FlatEntry { aa_page: 531, depth: 3, label: "3. Abschnitt. Vom Meinen, Wissen und Glauben" },

    FlatEntry { aa_page: 538, depth: 2, label: "Drittes Hauptstück. Die Architektonik der reinen Vernunft" },
    FlatEntry { aa_page: 550, depth: 2, label: "Viertes Hauptstück. Die Geschichte der reinen Vernunft" },
];

// ---------------------------------------------------------------------------
// Tree builder
// ---------------------------------------------------------------------------

/// Build the TOC tree from the flat list. Each entry becomes a KantTocNode.
/// Children are determined by depth: entries with depth > current entry's depth
/// that appear before the next entry at the same or lesser depth are children.
pub fn build_toc_tree() -> Vec<KantTocNode> {
    build_subtree(TOC, 1)
}

fn build_subtree(entries: &[FlatEntry], target_depth: u16) -> Vec<KantTocNode> {
    let mut nodes = Vec::new();
    let mut i = 0;

    while i < entries.len() {
        let entry = &entries[i];
        if entry.depth < target_depth {
            // We've moved up in the hierarchy — stop
            break;
        }
        if entry.depth > target_depth {
            // Skip — this is a child of a previous node, already consumed
            i += 1;
            continue;
        }

        // This entry is at the target depth. Collect its children.
        let child_start = i + 1;
        let child_end = find_section_end(entries, i);
        let children = if child_start < child_end {
            build_subtree(&entries[child_start..child_end], target_depth + 1)
        } else {
            Vec::new()
        };

        nodes.push(KantTocNode {
            label: entry.label.to_string(),
            aa_page: entry.aa_page,
            depth: entry.depth,
            children,
            content: Vec::new(),
        });

        i = child_end;
    }

    nodes
}

/// Find the index (in the full slice) of the first entry after `start_idx` that has
/// depth <= entries[start_idx].depth. This marks the end of the section.
fn find_section_end(entries: &[FlatEntry], start_idx: usize) -> usize {
    let base_depth = entries[start_idx].depth;
    for j in (start_idx + 1)..entries.len() {
        if entries[j].depth <= base_depth {
            return j;
        }
    }
    entries.len()
}

/// Return the flat TOC entries in document order for content assignment.
/// Each entry: (index_in_flat_list, aa_page, depth, label)
pub fn flat_toc_entries() -> Vec<(usize, u16, u16, &'static str)> {
    TOC.iter()
        .enumerate()
        .map(|(i, e)| (i, e.aa_page, e.depth, e.label))
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
    fn test_tree_structure() {
        let tree = build_toc_tree();
        // Top-level nodes: Zueignung, Vorrede, Einleitung, I. Elementarlehre, II. Methodenlehre
        assert_eq!(tree.len(), 5);
        assert_eq!(tree[0].label, "Zueignung");
        assert_eq!(tree[1].label, "Vorrede zur zweiten Auflage");
        assert_eq!(tree[2].label, "Einleitung");
        assert_eq!(tree[3].label, "I. Transscendentale Elementarlehre");
        assert_eq!(tree[4].label, "II. Transscendentale Methodenlehre");
    }

    #[test]
    fn test_einleitung_children() {
        let tree = build_toc_tree();
        let einleitung = &tree[2];
        assert_eq!(einleitung.children.len(), 7); // I through VII
    }

    #[test]
    fn test_elementarlehre_structure() {
        let tree = build_toc_tree();
        let elem = &tree[3]; // I. Transscendentale Elementarlehre
        assert_eq!(elem.children.len(), 2); // Erster Theil (Ästhetik), Zweiter Theil (Logik)

        let aesthetik = &elem.children[0];
        assert_eq!(aesthetik.label, "Erster Theil. Die transscendentale Ästhetik");
        assert_eq!(aesthetik.children.len(), 5); // §1, 1.Abschn, 2.Abschn, Anmerkungen, Beschluß

        let logik = &elem.children[1];
        assert_eq!(logik.label, "Zweiter Theil. Die transscendentale Logik");
        // Children: Einleitung, Erste Abtheilung (Analytik), Zweite Abtheilung (Dialektik)
        assert_eq!(logik.children.len(), 3);
    }

    #[test]
    fn test_methodenlehre_structure() {
        let tree = build_toc_tree();
        let meth = &tree[4]; // II. Transscendentale Methodenlehre
        assert_eq!(meth.children.len(), 4); // 4 Hauptstücke
    }

    #[test]
    fn test_flat_entry_count() {
        let flat = flat_toc_entries();
        assert_eq!(flat.len(), TOC.len());
        // First entry is Zueignung
        assert_eq!(flat[0].3, "Zueignung");
        // Last entry is Geschichte der reinen Vernunft
        assert_eq!(flat.last().unwrap().3, "Viertes Hauptstück. Die Geschichte der reinen Vernunft");
    }

    #[test]
    fn test_analogien_depth() {
        let tree = build_toc_tree();
        // Navigate: Elementarlehre > Logik > Analytik > Zweites Buch > 2.Hauptstück > 3.Abschnitt > 3.Analogien
        let logik = &tree[3].children[1];
        let analytik = &logik.children[1]; // Erste Abtheilung
        let zweites_buch = &analytik.children[1]; // Zweites Buch
        let hauptstueck_2 = &zweites_buch.children[2]; // 2. Hauptstück
        let abschnitt_3 = &hauptstueck_2.children[2]; // 3. Abschnitt
        let analogien = &abschnitt_3.children[2]; // 3. Analogien
        assert_eq!(analogien.children.len(), 3); // Erste, Zweite, Dritte Analogie
        assert_eq!(analogien.children[0].depth, 8);
    }
}
