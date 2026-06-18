// ---------------------------------------------------------------------------
// Authoritative TOC for Kant's Kritik der Urteilskraft (Akademie-Ausgabe Band V)
// ---------------------------------------------------------------------------
//
// Each entry: (aa_page, depth, label, slug_override). Labels are in the source's
// original orthography (Urtheilskraft, Eintheilung, …); Latin/antiqua is wrapped
// in `_ … _`. `aa_page` is the Akademie-Ausgabe Band V page where the section
// begins (taken from the `#PgNNN` links in the source Inhaltsübersicht).
//
// Depths form a strict tree (every node's parent sits exactly one level up). The
// source TOC's presentational indent classes (toc1–toc5) are NOT a clean tree —
// e.g. §61 is printed directly under "Zweiter Theil" — so depths are normalised
// to the logical hierarchy, not copied from the indent class. The book itself is
// the implicit depth-0 root (created by the importer); the title row and the AA
// editor's "Anmerkungen" apparatus are not content nodes and are omitted.

struct FlatEntry {
    aa_page: u16,
    depth: u16,
    label: &'static str,
    slug_override: Option<&'static str>,
}

const TOC: &[FlatEntry] = &[
    // -----------------------------------------------------------------------
    // Front matter + Einleitung
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 167,
        depth: 1,
        label: "Vorrede",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 171,
        depth: 1,
        label: "Einleitung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 171,
        depth: 2,
        label: "I. Von der Eintheilung der Philosophie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 174,
        depth: 2,
        label: "II. Vom Gebiete der Philosophie überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 176,
        depth: 2,
        label: "III. Von der Kritik der Urtheilskraft, als einem Verbindungsmittel der zwei Theile der Philosophie zu einem Ganzen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 179,
        depth: 2,
        label: "IV. Von der Urtheilskraft, als einem _a priori_ gesetzgebenden Vermögen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 181,
        depth: 2,
        label: "V. Das Princip der formalen Zweckmäßigkeit der Natur ist ein transscendentales Princip der Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 186,
        depth: 2,
        label: "VI. Von der Verbindung des Gefühls der Lust mit dem Begriffe der Zweckmäßigkeit der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 188,
        depth: 2,
        label: "VII. Von der ästhetischen Vorstellung der Zweckmäßigkeit der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 192,
        depth: 2,
        label: "VIII. Von der logischen Vorstellung der Zweckmäßigkeit der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 195,
        depth: 2,
        label: "IX. Von der Verknüpfung der Gesetzgebungen des Verstandes und der Vernunft durch die Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 199,
        depth: 1,
        label: "Eintheilung des ganzen Werks",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // Erster Theil. Kritik der ästhetischen Urtheilskraft
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 201,
        depth: 1,
        label: "Erster Theil. Kritik der ästhetischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 203,
        depth: 2,
        label: "Erster Abschnitt. Analytik der ästhetischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 203,
        depth: 3,
        label: "Erstes Buch. Analytik des Schönen",
        slug_override: None,
    },
    // 1. Moment
    FlatEntry {
        aa_page: 203,
        depth: 4,
        label: "1. Moment des Geschmacksurtheils der Qualität nach",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 203,
        depth: 5,
        label: "§ 1. Das Geschmacksurtheil ist ästhetisch",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 204,
        depth: 5,
        label: "§ 2. Das Wohlgefallen, welches das Geschmacksurtheil bestimmt, ist ohne alles Interesse",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 205,
        depth: 5,
        label: "§ 3. Das Wohlgefallen am Angenehmen ist mit Interesse verbunden",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 207,
        depth: 5,
        label: "§ 4. Das Wohlgefallen am Guten ist mit Interesse verbunden",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 209,
        depth: 5,
        label: "§ 5. Vergleichung der drei specifisch verschiedenen Arten des Wohlgefallens",
        slug_override: None,
    },
    // 2. Moment
    FlatEntry {
        aa_page: 211,
        depth: 4,
        label: "2. Moment des Geschmacksurtheils, nämlich seiner Quantität nach",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 211,
        depth: 5,
        label: "§ 6. Das Schöne ist das, was ohne Begriff als Object eines allgemeinen Wohlgefallens vorgestellt wird",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 212,
        depth: 5,
        label: "§ 7. Vergleichung des Schönen mit dem Angenehmen und Guten durch obiges Merkmal",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 213,
        depth: 5,
        label: "§ 8. Die Allgemeinheit des Wohlgefallens wird in einem Geschmacksurtheile nur als subjectiv vorgestellt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 216,
        depth: 5,
        label: "§ 9. Untersuchung der Frage: ob im Geschmacksurtheile das Gefühl der Lust vor der Beurtheilung des Gegenstandes, oder diese vor jener vorhergehe",
        slug_override: None,
    },
    // 3. Moment
    FlatEntry {
        aa_page: 219,
        depth: 4,
        label: "3. Moment der Geschmacksurtheile nach der Relation der Zwecke, welche in ihnen in Betrachtung gezogen wird",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 219,
        depth: 5,
        label: "§ 10. Von der Zweckmäßigkeit überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 221,
        depth: 5,
        label: "§ 11. Das Geschmacksurtheil hat nichts als die Form der Zweckmäßigkeit eines Gegenstandes (oder der Vorstellungsart desselben) zum Grunde",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 221,
        depth: 5,
        label: "§ 12. Das Geschmacksurtheil beruht auf Gründen _a priori_",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 223,
        depth: 5,
        label: "§ 13. Das reine Geschmacksurtheil ist von Reiz und Rührung unabhängig",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 223,
        depth: 5,
        label: "§ 14. Erläuterung durch Beispiele",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 226,
        depth: 5,
        label: "§ 15. Das Geschmacksurtheil ist von dem Begriffe der Vollkommenheit gänzlich unabhängig",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 229,
        depth: 5,
        label: "§ 16. Das Geschmacksurtheil, wodurch ein Gegenstand unter der Bedingung eines bestimmten Begriffs für schön erklärt wird, ist nicht rein",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 231,
        depth: 5,
        label: "§ 17. Vom Ideale der Schönheit",
        slug_override: None,
    },
    // 4. Moment
    FlatEntry {
        aa_page: 236,
        depth: 4,
        label: "4. Moment des Geschmacksurtheils nach der Modalität des Wohlgefallens an dem Gegenstande",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 236,
        depth: 5,
        label: "§ 18. Was die Modalität eines Geschmacksurtheils sei",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 237,
        depth: 5,
        label: "§ 19. Die subjective Nothwendigkeit, die wir dem Geschmacksurtheile beilegen, ist bedingt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 237,
        depth: 5,
        label: "§ 20. Die Bedingung der Nothwendigkeit, die ein Geschmacksurtheil vorgiebt, ist die Idee eines Gemeinsinnes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 238,
        depth: 5,
        label: "§ 21. Ob man mit Grunde einen Gemeinsinn voraussetzen könne",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 239,
        depth: 5,
        label: "§ 22. Die Nothwendigkeit der allgemeinen Beistimmung, die in einem Geschmacksurtheil gedacht wird, ist eine subjective Nothwendigkeit, die unter der Voraussetzung eines Gemeinsinnes als objectiv vorgestellt wird",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 240,
        depth: 4,
        label: "Allgemeine Anmerkung zum ersten Abschnitte der Analytik",
        slug_override: None,
    },
    // Zweites Buch. Analytik des Erhabenen
    FlatEntry {
        aa_page: 244,
        depth: 3,
        label: "Zweites Buch. Analytik des Erhabenen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 244,
        depth: 4,
        label: "§ 23. Übergang von dem Beurtheilungsvermögen des Schönen zu dem des Erhabenen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 247,
        depth: 4,
        label: "§ 24. Von der Eintheilung einer Untersuchung des Gefühls des Erhabenen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 248,
        depth: 4,
        label: "A. Vom Mathematisch-Erhabenen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 248,
        depth: 5,
        label: "§ 25. Namenerklärung des Erhabenen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 251,
        depth: 5,
        label: "§ 26. Von der Größenschätzung der Naturdinge, die zur Idee des Erhabenen erforderlich ist",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 257,
        depth: 5,
        label: "§ 27. Von der Qualität des Wohlgefallens in der Beurtheilung des Erhabenen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 260,
        depth: 4,
        label: "B. Vom Dynamisch-Erhabenen der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 260,
        depth: 5,
        label: "§ 28. Von der Natur als einer Macht",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 264,
        depth: 5,
        label: "§ 29. Von der Modalität des Urtheils über das Erhabene der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 266,
        depth: 4,
        label: "Allgemeine Anmerkung zur Exposition der ästhetischen reflectirenden Urtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 279,
        depth: 4,
        label: "Deduction der reinen ästhetischen Urtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 279,
        depth: 5,
        label: "§ 30. Die Deduction der ästhetischen Urtheile über die Gegenstände der Natur darf nicht auf das, was wir in dieser erhaben nennen, sondern nur auf das Schöne gerichtet werden",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 280,
        depth: 5,
        label: "§ 31. Von der Methode der Deduction der Geschmacksurtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 281,
        depth: 5,
        label: "§ 32. Erste Eigenthümlichkeit des Geschmacksurtheils",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 284,
        depth: 5,
        label: "§ 33. Zweite Eigenthümlichkeit des Geschmacksurtheils",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 285,
        depth: 5,
        label: "§ 34. Es ist kein objectives Princip des Geschmacks möglich",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 286,
        depth: 5,
        label: "§ 35. Das Princip des Geschmacks ist das subjective Princip der Urtheilskraft überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 287,
        depth: 5,
        label: "§ 36. Von der Aufgabe einer Deduction der Geschmacksurtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 289,
        depth: 5,
        label: "§ 37. Was wird eigentlich in einem Geschmacksurtheile von einem Gegenstande _a priori_ behauptet?",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 289,
        depth: 5,
        label: "§ 38. Deduction der Geschmacksurtheile",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 291,
        depth: 5,
        label: "§ 39. Von der Mittheilbarkeit einer Empfindung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 293,
        depth: 5,
        label: "§ 40. Vom Geschmacke als einer Art von _sensus communis_",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 296,
        depth: 5,
        label: "§ 41. Vom empirischen Interesse am Schönen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 298,
        depth: 5,
        label: "§ 42. Vom intellectuellen Interesse am Schönen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 303,
        depth: 5,
        label: "§ 43. Von der Kunst überhaupt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 304,
        depth: 5,
        label: "§ 44. Von der schönen Kunst",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 306,
        depth: 5,
        label: "§ 45. Schöne Kunst ist eine Kunst, sofern sie zugleich Natur zu sein scheint",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 307,
        depth: 5,
        label: "§ 46. Schöne Kunst ist Kunst des Genies",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 308,
        depth: 5,
        label: "§ 47. Erläuterung und Bestätigung obiger Erklärung vom Genie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 311,
        depth: 5,
        label: "§ 48. Vom Verhältnisse des Genies zum Geschmack",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 313,
        depth: 5,
        label: "§ 49. Von den Vermögen des Gemüths, welche das Genie ausmachen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 319,
        depth: 5,
        label: "§ 50. Von der Verbindung des Geschmacks mit Genie in Producten der schönen Kunst",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 320,
        depth: 5,
        label: "§ 51. Von der Eintheilung der schönen Künste",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 325,
        depth: 5,
        label: "§ 52. Von der Verbindung der schönen Künste in einem und demselben Producte",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 326,
        depth: 5,
        label: "§ 53. Vergleichung des ästhetischen Werths der schönen Künste untereinander",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 330,
        depth: 5,
        label: "§ 54. Anmerkung",
        slug_override: None,
    },
    // Zweiter Abschnitt. Dialektik der ästhetischen Urtheilskraft
    FlatEntry {
        aa_page: 337,
        depth: 2,
        label: "Zweiter Abschnitt. Dialektik der ästhetischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 337,
        depth: 3,
        label: "§ 55.",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 338,
        depth: 3,
        label: "§ 56. Vorstellung der Antinomie des Geschmacks",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 339,
        depth: 3,
        label: "§ 57. Auflösung der Antinomie des Geschmacks",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 346,
        depth: 3,
        label: "§ 58. Vom Idealismus der Zweckmäßigkeit der Natur sowohl als Kunst, als dem alleinigen Princip der ästhetischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 351,
        depth: 3,
        label: "§ 59. Von der Schönheit als Symbol der Sittlichkeit",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 354,
        depth: 3,
        label: "§ 60. Anhang. Von der Methodenlehre des Geschmacks",
        slug_override: None,
    },
    // -----------------------------------------------------------------------
    // Zweiter Theil. Kritik der teleologischen Urtheilskraft
    // -----------------------------------------------------------------------
    FlatEntry {
        aa_page: 357,
        depth: 1,
        label: "Zweiter Theil. Kritik der teleologischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 359,
        depth: 2,
        label: "§ 61. Von der objectiven Zweckmäßigkeit der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 362,
        depth: 2,
        label: "Erste Abtheilung. Analytik der teleologischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 362,
        depth: 3,
        label: "§ 62. Von der objectiven Zweckmäßigkeit, die bloß formal ist, zum Unterschiede von der materialen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 366,
        depth: 3,
        label: "§ 63. Von der relativen Zweckmäßigkeit der Natur zum Unterschiede von der innern",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 369,
        depth: 3,
        label: "§ 64. Von dem eigenthümlichen Charakter der Dinge als Naturzwecke",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 372,
        depth: 3,
        label: "§ 65. Dinge als Naturzwecke sind organisirte Wesen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 376,
        depth: 3,
        label: "§ 66. Vom Princip der Beurtheilung der innern Zweckmäßigkeit in organisirten Wesen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 377,
        depth: 3,
        label: "§ 67. Vom Princip der teleologischen Beurtheilung der Natur überhaupt als System der Zwecke",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 381,
        depth: 3,
        label: "§ 68. Von dem Princip der Teleologie als innerem Princip der Naturwissenschaft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 385,
        depth: 2,
        label: "Zweite Abtheilung. Dialektik der teleologischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 385,
        depth: 3,
        label: "§ 69. Was eine Antinomie der Urtheilskraft sei",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 386,
        depth: 3,
        label: "§ 70. Vorstellung dieser Antinomie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 388,
        depth: 3,
        label: "§ 71. Vorbereitung zur Auflösung obiger Antinomie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 389,
        depth: 3,
        label: "§ 72. Von den mancherlei Systemen über die Zweckmäßigkeit der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 392,
        depth: 3,
        label: "§ 73. Keines der obigen Systeme leistet das, was es vorgiebt",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 395,
        depth: 3,
        label: "§ 74. Die Ursache der Unmöglichkeit, den Begriff einer Technik der Natur dogmatisch zu behandeln, ist die Unerklärlichkeit eines Naturzwecks",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 397,
        depth: 3,
        label: "§ 75. Der Begriff einer objectiven Zweckmäßigkeit der Natur ist ein kritisches Princip der Vernunft für die reflectirende Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 401,
        depth: 3,
        label: "§ 76. Anmerkung",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 405,
        depth: 3,
        label: "§ 77. Von der Eigenthümlichkeit des menschlichen Verstandes, wodurch uns der Begriff eines Naturzwecks möglich wird",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 410,
        depth: 3,
        label: "§ 78. Von der Vereinigung des Princips des allgemeinen Mechanismus der Materie mit dem teleologischen in der Technik der Natur",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 416,
        depth: 2,
        label: "Anhang. Methodenlehre der teleologischen Urtheilskraft",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 416,
        depth: 3,
        label: "§ 79. Ob die Teleologie als zur Naturlehre gehörend abgehandelt werden müsse",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 417,
        depth: 3,
        label: "§ 80. Von der nothwendigen Unterordnung des Princips des Mechanismus unter dem teleologischen in Erklärung eines Dinges als Naturzwecks",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 421,
        depth: 3,
        label: "§ 81. Von der Beigesellung des Mechanismus zum teleologischen Princip in der Erklärung eines Naturzwecks als Naturproducts",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 425,
        depth: 3,
        label: "§ 82. Von dem teleologischen System in den äußern Verhältnissen organisirter Wesen",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 429,
        depth: 3,
        label: "§ 83. Von dem letzten Zwecke der Natur als eines teleologischen Systems",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 434,
        depth: 3,
        label: "§ 84. Von dem Endzwecke des Daseins einer Welt, d. i. der Schöpfung selbst",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 436,
        depth: 3,
        label: "§ 85. Von der Physikotheologie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 442,
        depth: 3,
        label: "§ 86. Von der Ethikotheologie",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 447,
        depth: 3,
        label: "§ 87. Von dem moralischen Beweise des Daseins Gottes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 453,
        depth: 3,
        label: "§ 88. Beschränkung der Gültigkeit des moralischen Beweises",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 459,
        depth: 3,
        label: "§ 89. Von dem Nutzen des moralischen Arguments",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 461,
        depth: 3,
        label: "§ 90. Von der Art des Fürwahrhaltens in einem teleologischen Beweise des Daseins Gottes",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 467,
        depth: 3,
        label: "§ 91. Von der Art des Fürwahrhaltens durch einen praktischen Glauben",
        slug_override: None,
    },
    FlatEntry {
        aa_page: 475,
        depth: 2,
        label: "Allgemeine Anmerkung zur Teleologie",
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
    use super::super::filenames::slugify;
    use super::*;

    #[test]
    fn test_flat_entry_count() {
        let flat = flat_toc_entries();
        assert_eq!(flat.len(), TOC.len());
        assert_eq!(flat.len(), 122);
        assert_eq!(flat[0].3, "Vorrede");
        assert_eq!(
            flat.last().unwrap().3,
            "Allgemeine Anmerkung zur Teleologie"
        );
    }

    #[test]
    fn test_five_top_level_divisions() {
        let roots: Vec<&str> = TOC
            .iter()
            .filter(|e| e.depth == 1)
            .map(|e| e.label)
            .collect();
        assert_eq!(
            roots,
            vec![
                "Vorrede",
                "Einleitung",
                "Eintheilung des ganzen Werks",
                "Erster Theil. Kritik der ästhetischen Urtheilskraft",
                "Zweiter Theil. Kritik der teleologischen Urtheilskraft",
            ]
        );
    }

    /// Every node deeper than depth 1 must have a findable parent: scanning
    /// backwards, the nearest entry one level up appears before any entry
    /// shallower than the parent's depth. This guarantees a valid ltree path.
    #[test]
    fn test_every_node_has_findable_parent() {
        for (i, e) in TOC.iter().enumerate() {
            if e.depth <= 1 {
                continue;
            }
            let target = e.depth - 1;
            let mut found = false;
            for prev in TOC[..i].iter().rev() {
                if prev.depth == target {
                    found = true;
                    break;
                }
                if prev.depth < target {
                    break;
                }
            }
            assert!(
                found,
                "entry {i} ({}) at depth {} has no parent",
                e.label, e.depth
            );
        }
    }

    /// Sibling nodes (same parent) must have distinct slugs so ltree paths are
    /// unique. Parent is identified by the same backward scan as the importer.
    #[test]
    fn test_sibling_slugs_unique() {
        use std::collections::HashMap;
        let parent_of = |i: usize| -> Option<usize> {
            let target = TOC[i].depth.checked_sub(1)?;
            if target == 0 {
                return None;
            }
            for j in (0..i).rev() {
                if TOC[j].depth == target {
                    return Some(j);
                }
                if TOC[j].depth < target {
                    break;
                }
            }
            None
        };
        let mut seen: HashMap<(Option<usize>, String), usize> = HashMap::new();
        for (i, e) in TOC.iter().enumerate() {
            let slug = e
                .slug_override
                .map(|s| s.to_string())
                .unwrap_or_else(|| slugify(e.label));
            let key = (parent_of(i), slug.clone());
            if let Some(prev) = seen.insert(key, i) {
                panic!("sibling slug collision: entries {prev} and {i} share slug {slug:?}");
            }
        }
    }

    #[test]
    fn test_section_paragraph_count() {
        // §§ 1–91 each appear exactly once as a node.
        let para_nodes = TOC.iter().filter(|e| e.label.starts_with("§ ")).count();
        assert_eq!(para_nodes, 91);
    }
}
