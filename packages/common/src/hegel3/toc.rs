//! Authoritative TOC for hegel3 — Wissenschaft der Logik, Bd. 1,1:
//! *Die objektive Logik: Das Seyn* (Nürnberg 1812), the first-edition
//! Doctrine of Being that the 1832 second edition (hegel2) superseded.
//!
//! One entry per content `<div>` of the DTA TEI, in document order, as the
//! converter derives them (`hegel1_tei_to_md --page-key page_1812
//! --promote-head "Erstes Buch. Das Seyn"` — the DTA nests the Buch inside
//! the Logik division; the print's own contents list it at top level, so it
//! is promoted to a top-level sibling); labels keep the 1812 orthography
//! (long-s, combining umlauts as composed here).
//!
//! Depths form a strict tree — every node's parent sits exactly one level up.
//! The book itself is the implicit depth-0 root, created by the importer.

pub struct TocEntry {
    /// 1-based document position, also the `NNN` filename prefix.
    pub position: u16,
    pub depth: u16,
    /// The 1812 page the node starts on — Roman in the front matter (the
    /// DTA's supplied brackets stripped), Arabic in the body.
    pub page: Option<&'static str>,
    pub slug: &'static str,
    pub label: &'static str,
}

const TOC: &[TocEntry] = &[
    TocEntry {
        position: 1,
        depth: 1,
        page: Some("III"),
        slug: "vorrede",
        label: "Vorrede",
    },
    TocEntry {
        position: 2,
        depth: 1,
        page: Some("I"),
        slug: "einleitung",
        label: "Einleitung",
    },
    TocEntry {
        position: 3,
        depth: 1,
        page: Some("1"),
        slug: "logik",
        label: "Logik",
    },
    TocEntry {
        position: 4,
        depth: 2,
        page: Some("1"),
        slug: "ueber_die_allgemeine_eintheilung_derselben",
        label: "Ueber die allgemeine Eintheilung derſelben",
    },
    TocEntry {
        position: 5,
        depth: 1,
        page: Some("6"),
        slug: "erstes_buch_das_seyn",
        label: "Erſtes Buch. Das Seyn",
    },
    TocEntry {
        position: 6,
        depth: 2,
        page: Some("6"),
        slug: "womit_muss_der_anfang_der_wissenschaft_gemacht_werden",
        label: "Womit muß der Anfang der Wiſſenſchaft gemacht werden?",
    },
    TocEntry {
        position: 7,
        depth: 2,
        page: Some("19"),
        slug: "allgemeine_eintheilung_des_seyns",
        label: "Allgemeine Eintheilung des Seyns",
    },
    TocEntry {
        position: 8,
        depth: 2,
        page: Some("21"),
        slug: "erster_abschnitt_bestimmtheit",
        label: "Erſter Abſchnitt. Beſtimmtheit",
    },
    TocEntry {
        position: 9,
        depth: 3,
        page: Some("22"),
        slug: "erstes_kapitel_seyn",
        label: "Erſtes Kapitel. Seyn",
    },
    TocEntry {
        position: 10,
        depth: 4,
        page: Some("22"),
        slug: "a_seyn",
        // Body head prints bare "A."; label follows the Inhaltsanzeige, and the
        // slug follows the label — as it does for B and C.
        label: "A. Seyn",
    },
    TocEntry {
        position: 11,
        depth: 4,
        page: Some("22"),
        slug: "b_nichts",
        label: "B. Nichts",
    },
    TocEntry {
        position: 12,
        depth: 4,
        page: Some("23"),
        slug: "c_werden",
        label: "C. Werden",
    },
    TocEntry {
        position: 13,
        depth: 5,
        page: Some("23"),
        slug: "einheit_des_seyns_und_nichts",
        label: "Einheit des Seyns und Nichts",
    },
    TocEntry {
        position: 14,
        depth: 6,
        page: Some("23"),
        slug: "anmerkung_1",
        label: "Anmerkung 1",
    },
    TocEntry {
        position: 15,
        depth: 6,
        page: Some("33"),
        slug: "anmerkung_2",
        label: "Anmerkung 2",
    },
    TocEntry {
        position: 16,
        depth: 6,
        page: Some("38"),
        slug: "anmerkung_3",
        label: "Anmerkung 3",
    },
    TocEntry {
        position: 17,
        depth: 6,
        page: Some("40"),
        slug: "anmerkung_4",
        label: "Anmerkung 4",
    },
    TocEntry {
        position: 18,
        depth: 5,
        page: Some("43"),
        slug: "2_momente_des_werdens",
        label: "2. Momente des Werdens",
    },
    TocEntry {
        position: 19,
        depth: 5,
        page: Some("44"),
        slug: "3_aufheben_des_werdens",
        label: "3. Aufheben des Werdens",
    },
    TocEntry {
        position: 20,
        depth: 6,
        page: Some("45"),
        slug: "anmerkung",
        label: "Anmerkung",
    },
    TocEntry {
        position: 21,
        depth: 3,
        page: Some("47"),
        slug: "zweytes_kapitel_das_daseyn",
        label: "Zweytes Kapitel. Das Daſeyn",
    },
    TocEntry {
        position: 22,
        depth: 4,
        page: Some("47"),
        slug: "a_daseyn_als_solches",
        label: "A. Daſeyn als ſolches",
    },
    TocEntry {
        position: 23,
        depth: 5,
        page: Some("47"),
        slug: "1_daseyn_ueberhaupt",
        label: "1. Daſeyn uͤberhaupt",
    },
    TocEntry {
        position: 24,
        depth: 5,
        page: Some("48"),
        slug: "2_realitaet",
        label: "2. Realitaͤt",
    },
    TocEntry {
        position: 25,
        depth: 6,
        page: Some("49"),
        slug: "a_andersseyn",
        label: "a) Andersſeyn",
    },
    TocEntry {
        position: 26,
        depth: 6,
        page: Some("51"),
        slug: "b_seyn_fuer_anderes_und_ansichseyn",
        label: "b) Seyn-fuͤr-Anderes und Anſichſeyn",
    },
    TocEntry {
        position: 27,
        depth: 6,
        page: Some("53"),
        slug: "c_realitaet",
        label: "c) Realitaͤt",
    },
    TocEntry {
        position: 28,
        depth: 7,
        page: Some("54"),
        slug: "anmerkung_5",
        label: "Anmerkung",
    },
    TocEntry {
        position: 29,
        depth: 5,
        page: Some("57"),
        slug: "3_etwas",
        label: "3. Etwas",
    },
    TocEntry {
        position: 30,
        depth: 4,
        page: Some("60"),
        slug: "b_bestimmtheit",
        label: "B. Beſtimmtheit",
    },
    TocEntry {
        position: 31,
        depth: 5,
        page: Some("60"),
        slug: "1_grenze",
        label: "1. Grenze",
    },
    TocEntry {
        position: 32,
        depth: 5,
        page: Some("65"),
        slug: "2_bestimmtheit",
        label: "2. Beſtimmtheit",
    },
    TocEntry {
        position: 33,
        depth: 6,
        page: Some("66"),
        slug: "a_bestimmung",
        label: "a.) Beſtimmung",
    },
    TocEntry {
        position: 34,
        depth: 6,
        page: Some("66"),
        slug: "b_beschaffenheit",
        label: "b.) Beſchaffenheit",
    },
    TocEntry {
        position: 35,
        depth: 6,
        page: Some("67"),
        slug: "c_qualitaet",
        label: "c.) Qualitaͤt",
    },
    TocEntry {
        position: 36,
        depth: 7,
        page: Some("68"),
        slug: "anmerkung_6",
        label: "Anmerkung",
    },
    TocEntry {
        position: 37,
        depth: 5,
        page: Some("69"),
        slug: "3_veraenderung",
        label: "3. Veraͤnderung",
    },
    TocEntry {
        position: 38,
        depth: 6,
        page: Some("70"),
        slug: "a_veraenderung_der_beschaffenheit",
        label: "a) Veraͤnderung der Beſchaffenheit",
    },
    TocEntry {
        position: 39,
        depth: 6,
        page: Some("71"),
        slug: "b_sollen_und_schranke",
        label: "b.) Sollen und Schranke",
    },
    TocEntry {
        position: 40,
        depth: 7,
        page: Some("74"),
        slug: "anmerkung_7",
        label: "Anmerkung",
    },
    TocEntry {
        position: 41,
        depth: 6,
        page: Some("75"),
        slug: "c_negation",
        label: "c.) Negation",
    },
    TocEntry {
        position: 42,
        depth: 7,
        page: Some("75"),
        slug: "anmerkung_8",
        label: "Anmerkung",
    },
    TocEntry {
        position: 43,
        depth: 4,
        page: Some("79"),
        slug: "c_qualitative_unendlichkeit",
        label: "C. (Qualitative) Unendlichkeit",
    },
    TocEntry {
        position: 44,
        depth: 5,
        page: Some("79"),
        slug: "1_endlichkeit_und_unendlichkeit",
        label: "1. Endlichkeit und Unendlichkeit",
    },
    TocEntry {
        position: 45,
        depth: 5,
        page: Some("81"),
        slug: "2_wechselbestimmung_des_endlichen_und_unendlichen",
        label: "2. Wechſelbeſtimmung des Endlichen und Unendlichen",
    },
    TocEntry {
        position: 46,
        depth: 5,
        page: Some("85"),
        slug: "3_rueckkehr_der_unendlichkeit_in_sich",
        label: "3. Ruͤckkehr der Unendlichkeit in ſich",
    },
    TocEntry {
        position: 47,
        depth: 6,
        page: Some("87"),
        slug: "anmerkung_9",
        label: "Anmerkung",
    },
    TocEntry {
        position: 48,
        depth: 3,
        page: Some("91"),
        slug: "drittes_kapitel_das_fuersichseyn",
        label: "Drittes Kapitel. Das Fuͤrſichſeyn",
    },
    TocEntry {
        position: 49,
        depth: 4,
        page: Some("92"),
        slug: "a_fuersichseyn_als_solches",
        label: "A. Fuͤrſichſeyn als ſolches",
    },
    TocEntry {
        position: 50,
        depth: 5,
        page: Some("92"),
        slug: "1_fuersichseyn_ueberhaupt",
        label: "1. Fuͤrſichſeyn uͤberhaupt",
    },
    TocEntry {
        position: 51,
        depth: 5,
        page: Some("92"),
        slug: "2_die_momente_des_fuersichseyns",
        label: "2. Die Momente des Fuͤrſichſeyns",
    },
    TocEntry {
        position: 52,
        depth: 6,
        page: Some("93"),
        slug: "a_das_moment_seines_ansichseyns",
        label: "a.) das Moment ſeines Anſichſeyns,",
    },
    TocEntry {
        position: 53,
        depth: 6,
        page: Some("93"),
        slug: "b_fuer_eines_seyn",
        label: "b.) Fuͤr eines ſeyn",
    },
    TocEntry {
        position: 54,
        depth: 7,
        page: Some("94"),
        slug: "anmerkung_10",
        label: "Anmerkung",
    },
    TocEntry {
        position: 55,
        depth: 6,
        page: Some("95"),
        slug: "c_idealitaet",
        label: "c.) Idealitaͤt",
    },
    TocEntry {
        position: 56,
        depth: 5,
        page: Some("99"),
        slug: "3_werden_des_eins",
        label: "3. Werden des Eins",
    },
    TocEntry {
        position: 57,
        depth: 4,
        page: Some("101"),
        slug: "b_das_eins",
        label: "B. Das Eins",
    },
    TocEntry {
        position: 58,
        depth: 5,
        page: Some("101"),
        slug: "1_das_eins_und_das_leere",
        label: "1. Das Eins und das Leere",
    },
    TocEntry {
        position: 59,
        depth: 6,
        page: Some("103"),
        slug: "anmerkung_11",
        label: "Anmerkung",
    },
    TocEntry {
        position: 60,
        depth: 5,
        page: Some("104"),
        slug: "2_viele_eins",
        label: "2. Viele Eins",
    },
    TocEntry {
        position: 61,
        depth: 6,
        page: Some("107"),
        slug: "anmerkung_12",
        label: "Anmerkung",
    },
    TocEntry {
        position: 62,
        depth: 5,
        page: Some("108"),
        slug: "3_gegenseitige_repulsion",
        label: "3. Gegenſeitige Repulſion",
    },
    TocEntry {
        position: 63,
        depth: 4,
        page: Some("112"),
        slug: "c_attraktion",
        label: "C. Attraktion",
    },
    TocEntry {
        position: 64,
        depth: 5,
        page: Some("113"),
        slug: "1_ein_eins",
        label: "1. Ein Eins",
    },
    TocEntry {
        position: 65,
        depth: 5,
        page: Some("114"),
        slug: "2_gleichgewicht_der_attraction_und_repulsion",
        label: "2. Gleichgewicht der Attraction und Repulſion",
    },
    TocEntry {
        position: 66,
        depth: 6,
        page: Some("119"),
        slug: "anmerkung_13",
        label: "Anmerkung",
    },
    TocEntry {
        position: 67,
        depth: 5,
        page: Some("128"),
        slug: "3_uebergang_zur_quantitaet",
        label: "3. Uebergang zur Quantitaͤt",
    },
    TocEntry {
        position: 68,
        depth: 2,
        page: Some("130"),
        slug: "zweyter_abschnitt_groesse",
        label: "Zweyter Abſchnitt. Groͤße",
    },
    TocEntry {
        position: 69,
        depth: 3,
        page: Some("132"),
        slug: "anmerkung_14",
        label: "Anmerkung",
    },
    TocEntry {
        position: 70,
        depth: 3,
        page: Some("134"),
        slug: "erstes_kapitel_die_quantitaet",
        label: "Erſtes Kapitel. Die Quantitaͤt",
    },
    TocEntry {
        position: 71,
        depth: 4,
        page: Some("134"),
        slug: "a_die_reine_quantitaet",
        label: "A. Die reine Quantitaͤt",
    },
    TocEntry {
        position: 72,
        depth: 5,
        page: Some("136"),
        slug: "anmerkung_1_2",
        label: "Anmerkung 1",
    },
    TocEntry {
        position: 73,
        depth: 5,
        page: Some("138"),
        slug: "anmerkung_2_2",
        label: "Anmerkung 2",
    },
    TocEntry {
        position: 74,
        depth: 4,
        page: Some("151"),
        slug: "b_continuirliche_und_discrete_groesse",
        label: "B. Continuirliche und diſcrete Groͤße",
    },
    TocEntry {
        position: 75,
        depth: 5,
        page: Some("152"),
        slug: "anmerkung_15",
        label: "Anmerkung",
    },
    TocEntry {
        position: 76,
        depth: 4,
        page: Some("154"),
        slug: "c_begrenzung_der_quantitaet",
        label: "C. Begrenzung der Quantitaͤt",
    },
    TocEntry {
        position: 77,
        depth: 3,
        page: Some("156"),
        slug: "zweytes_kapitel_quantum",
        label: "Zweytes Kapitel. Quantum",
    },
    TocEntry {
        position: 78,
        depth: 4,
        page: Some("157"),
        slug: "a_die_zahl",
        label: "A. Die Zahl",
    },
    TocEntry {
        position: 79,
        depth: 5,
        page: Some("162"),
        slug: "anmerkung_1_3",
        label: "Anmerkung 1",
    },
    TocEntry {
        position: 80,
        depth: 5,
        page: Some("163"),
        slug: "anmerkung_2_3",
        label: "Anmerkung 2",
    },
    TocEntry {
        position: 81,
        depth: 4,
        page: Some("169"),
        slug: "b_extensives_und_intensives_quantum",
        label: "B. Extenſives und intenſives Quantum",
    },
    TocEntry {
        position: 82,
        depth: 5,
        page: Some("169"),
        slug: "1_unterschied_derselben",
        label: "1. Unterſchied derſelben",
    },
    TocEntry {
        position: 83,
        depth: 5,
        page: Some("174"),
        slug: "2_identitaet_der_extensiven_und_intensiven_groesse",
        label: "2. Identitaͤt der extenſiven und intenſiven Groͤße",
    },
    TocEntry {
        position: 84,
        depth: 6,
        page: Some("176"),
        slug: "anmerkung_16",
        label: "Anmerkung",
    },
    TocEntry {
        position: 85,
        depth: 5,
        page: Some("179"),
        slug: "3_veraenderung_des_quantums",
        label: "3. Veraͤnderung des Quantums",
    },
    TocEntry {
        position: 86,
        depth: 4,
        page: Some("182"),
        slug: "c_quantitative_unendlichkeit",
        label: "C. Quantitative Unendlichkeit",
    },
    TocEntry {
        position: 87,
        depth: 5,
        page: Some("182"),
        slug: "1_begriff_derselben",
        label: "1. Begriff derſelben",
    },
    TocEntry {
        position: 88,
        depth: 5,
        page: Some("183"),
        slug: "2_der_unendliche_progress",
        label: "2. Der unendliche Progreß",
    },
    TocEntry {
        position: 89,
        depth: 6,
        page: Some("187"),
        slug: "anmerkung_1_4",
        label: "Anmerkung 1",
    },
    TocEntry {
        position: 90,
        depth: 6,
        page: Some("194"),
        slug: "anmerkung_2_4",
        label: "Anmerkung 2",
    },
    TocEntry {
        position: 91,
        depth: 5,
        page: Some("200"),
        slug: "3_unendlichkeit_des_quantums",
        label: "3. Unendlichkeit des Quantums",
    },
    TocEntry {
        position: 92,
        depth: 6,
        page: Some("206"),
        slug: "anmerkung_17",
        label: "Anmerkung",
    },
    TocEntry {
        position: 93,
        depth: 3,
        page: Some("248"),
        slug: "drittes_kapitel_das_quantitative_verhaeltniss",
        label: "Drittes Kapitel. Das quantitative Verhaͤltniß",
    },
    TocEntry {
        position: 94,
        depth: 4,
        page: Some("249"),
        slug: "a_das_directe_verhaeltniss",
        label: "A. Das directe Verhaͤltniß",
    },
    TocEntry {
        position: 95,
        depth: 4,
        page: Some("253"),
        slug: "b_das_umgekehrte_verhaeltniss",
        label: "B. Das umgekehrte Verhaͤltniß",
    },
    TocEntry {
        position: 96,
        depth: 4,
        page: Some("258"),
        slug: "c_potenzenverhaeltniss",
        label: "C. Potenzenverhaͤltniß",
    },
    TocEntry {
        position: 97,
        depth: 5,
        page: Some("261"),
        slug: "anmerkung_18",
        label: "Anmerkung",
    },
    TocEntry {
        position: 98,
        depth: 2,
        page: Some("264"),
        slug: "dritter_abschnitt_das_maass",
        label: "Dritter Abſchnitt. Das Maaß",
    },
    TocEntry {
        position: 99,
        depth: 3,
        page: Some("268"),
        slug: "erstes_kapitel_die_specifische_quantitaet",
        label: "Erſtes Kapitel. Die ſpecifiſche Quantitaͤt",
    },
    TocEntry {
        position: 100,
        depth: 4,
        page: Some("268"),
        slug: "a_das_specifische_quantum",
        label: "A. Das ſpecifiſche Quantum",
    },
    TocEntry {
        position: 101,
        depth: 4,
        page: Some("271"),
        slug: "b_die_regel",
        label: "B. Die Regel",
    },
    TocEntry {
        position: 102,
        depth: 5,
        page: Some("271"),
        slug: "1_die_qualitative_und_quantitative_groessen_bestimmtheit",
        label: "1. Die qualitative und quantitative Groͤßen-Beſtimmtheit",
    },
    TocEntry {
        position: 103,
        depth: 5,
        page: Some("274"),
        slug: "2_qualitaet_und_quantum",
        label: "2. Qualitaͤt und Quantum",
    },
    TocEntry {
        position: 104,
        depth: 5,
        page: Some("278"),
        slug: "3_unterscheidung_beyder_seiten_als_qualitaeten",
        label: "3. Unterſcheidung beyder Seiten als Qualitaͤten",
    },
    TocEntry {
        position: 105,
        depth: 6,
        page: Some("281"),
        slug: "anmerkung_19",
        label: "Anmerkung",
    },
    TocEntry {
        position: 106,
        depth: 4,
        page: Some("284"),
        slug: "c_verhaeltniss_von_qualitaeten",
        label: "C. Verhaͤltniß von Qualitaͤten",
    },
    TocEntry {
        position: 107,
        depth: 3,
        page: Some("289"),
        slug: "zweytes_kapitel_verhaeltniss_selbststaendiger_maasse",
        label: "Zweytes Kapitel. Verhaͤltniß ſelbſtſtaͤndiger Maaße",
    },
    TocEntry {
        position: 108,
        depth: 4,
        page: Some("291"),
        slug: "a_das_verhaeltniss_selbststaendiger_maasse",
        label: "A. Das Verhaͤltniß ſelbſtſtaͤndiger Maaße",
    },
    TocEntry {
        position: 109,
        depth: 5,
        page: Some("291"),
        slug: "1_neutralitaet",
        label: "1. Neutralitaͤt",
    },
    TocEntry {
        position: 110,
        depth: 5,
        page: Some("293"),
        slug: "2_specification_der_neutralitaet",
        label: "2. Specification der Neutralitaͤt",
    },
    TocEntry {
        position: 111,
        depth: 5,
        page: Some("298"),
        slug: "3_wahlverwandtschaft",
        label: "3. Wahlverwandtſchaft",
    },
    TocEntry {
        position: 112,
        depth: 6,
        page: Some("301"),
        slug: "anmerkung_20",
        label: "Anmerkung",
    },
    TocEntry {
        position: 113,
        depth: 4,
        page: Some("307"),
        slug: "b_knotenlinie_von_maassverhaeltnissen",
        label: "B. Knotenlinie von Maaßverhaͤltniſſen",
    },
    TocEntry {
        position: 114,
        depth: 5,
        page: Some("311"),
        slug: "anmerkung_21",
        label: "Anmerkung",
    },
    TocEntry {
        position: 115,
        depth: 4,
        page: Some("315"),
        slug: "c_das_maasslose",
        label: "C. Das Maaßloſe",
    },
    TocEntry {
        position: 116,
        depth: 3,
        page: Some("321"),
        slug: "drittes_kapitel_das_werden_des_wesens",
        label: "Drittes Kapitel. Das Werden des Weſens",
    },
    TocEntry {
        position: 117,
        depth: 4,
        page: Some("321"),
        slug: "a_die_indifferenz",
        label: "A. Die Indifferenz",
    },
    TocEntry {
        position: 118,
        depth: 4,
        page: Some("323"),
        slug: "b_das_selbststaendige_als_umgekehrtes_verhaeltniss_seiner_factoren",
        label: "B. Das Selbſtſtaͤndige als umgekehrtes Verhaͤltniß ſeiner Factoren",
    },
    TocEntry {
        position: 119,
        depth: 5,
        page: Some("328"),
        slug: "anmerkung_22",
        label: "Anmerkung",
    },
    TocEntry {
        position: 120,
        depth: 4,
        page: Some("331"),
        slug: "c_hervorgehen_des_wesens",
        label: "C. Hervorgehen des Weſens",
    },
];

pub fn entries() -> &'static [TocEntry] {
    TOC
}

pub fn toc_len() -> usize {
    TOC.len()
}

pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    TOC.iter()
        .enumerate()
        .map(|(i, e)| {
            (
                i,
                e.page.map(str::to_string),
                e.depth,
                e.label,
                Some(e.slug),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn positions_are_contiguous() {
        for (i, e) in TOC.iter().enumerate() {
            assert_eq!(e.position as usize, i + 1);
        }
    }

    #[test]
    fn slugs_are_unique() {
        let set: HashSet<&str> = TOC.iter().map(|e| e.slug).collect();
        assert_eq!(set.len(), TOC.len());
    }

    #[test]
    fn depths_form_a_strict_tree() {
        let mut stack: Vec<u16> = Vec::new();
        for e in TOC {
            while stack.last().is_some_and(|&d| d >= e.depth) {
                stack.pop();
            }
            let expected = stack.last().map_or(1, |&d| d + 1);
            assert_eq!(e.depth, expected, "depth jump at {}", e.slug);
            stack.push(e.depth);
        }
    }

    #[test]
    fn arabic_body_pages_ascend() {
        let mut prev = 0u32;
        for e in TOC {
            let Some(p) = e.page else { continue };
            let Ok(n) = p.parse::<u32>() else { continue };
            assert!(n >= prev, "page regression at {}: {p}", e.slug);
            prev = n;
        }
    }
}
