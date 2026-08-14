//! Canonical TOC for Leviathan (1651) — one row per curated file, generated
//! from the md_reviewed front matter emitted by `hobbes1_tei_to_md` (labels
//! are the diplomatic 1651 heads; pages are the printed folio numbers, with
//! the `b`-suffixed second occurrences of the five twice-printed pages).
//! Row order = position order; flat index N = file NNN+1.

/// (page_1651, depth, label)
pub const TOC: &[(Option<&str>, u16, &str)] = &[
    (None, 1, "Engraved Title Page"),
    (None, 1, "Title Page"),
    (
        None,
        1,
        "TO MY MOST HONOR'D FRIEND Mr _FRANCIS GODOLPHIN_ of _Godolphin._",
    ),
    (Some("1"), 1, "THE INTRODUCTION."),
    (Some("3"), 1, "OF MAN."),
    (Some("3"), 2, "CHAP. I. _Of_ SENSE."),
    (Some("4"), 2, "CHAP. II. _Of_ IMAGINATION."),
    (
        Some("8"),
        2,
        "CHAP. III. _Of the Consequence or_ TRAYNE _of Imaginations._",
    ),
    (Some("12"), 2, "CHAP. IV. _Of_ SPEECH."),
    (Some("18"), 2, "CHAP. V. _Of_ REASON, and SCIENCE."),
    (
        Some("23"),
        2,
        "CHAP. VI. _Of the Interiour Beginnings of Voluntary Motions; commonly called the_ PASSIONS. _And the Speeches by which they are expressed._",
    ),
    (
        Some("30"),
        2,
        "CHAP. VII. _Of the_ Ends, _or_ Resolutions _of_ DISCOURSE.",
    ),
    (
        Some("32"),
        2,
        "CHAP. VIII. _Of the_ VERTUES _commonly called_ INTELLECTUALL; _and their contrary_ DEFECTS.",
    ),
    (
        Some("40"),
        2,
        "CHAP. IX. _Of the Severall_ SUBIECTS _of_ KNOWLEDGE.",
    ),
    (
        Some("41"),
        2,
        "CHAP. X. _Of_ POWER, WORTH, DIGNITY, HONOUR, _and_ WORTHINESSE.",
    ),
    (Some("47"), 2, "CHAP. XI. _Of the difference of_ MANNERS."),
    (Some("52"), 2, "CHAP. XII. OF RELIGION."),
    (
        Some("60"),
        2,
        "CHAP. XIII. _Of the_ NATURALL CONDITION _of Mankind, as concerning their Felicity, and Misery._",
    ),
    (
        Some("64"),
        2,
        "CHAP. XIV. _Of the first and second_ NATURALL LAWES, _and of_ CONTRACTS.",
    ),
    (Some("71"), 2, "CHAP. XV. _Of other Lawes of Nature._"),
    (
        Some("80"),
        2,
        "CHAP. XVI. _Of_ PERSONS, AUTHORS, _and things Personated._",
    ),
    (Some("85"), 1, "OF COMMON-VVEALTH."),
    (
        Some("85"),
        2,
        "CHAP. XVII. _Of the Causes, Generation, and Definition of a_ COMMON-WEALTH.",
    ),
    (
        Some("88"),
        2,
        "CHAP. XVIII. _Of the_ RIGHTS _of Soveraignes by Institution._",
    ),
    (
        Some("94"),
        2,
        "CHAP. XIX. _Of the severall Kinds of_ Common-wealth _by Institution, and of Succession to the Soveraigne Power._",
    ),
    (
        Some("101"),
        2,
        "CHAP. XX. _Of Dominion_ PATERNALL, _and_ DESPOTICALL.",
    ),
    (Some("107"), 2, "CHAP. XXI. _Of the_ LIBERTY _of Subjects._"),
    (
        Some("115"),
        2,
        "CHAP. XXII. _Of_ SYSTEMES _Subject, Politicall, and Private._",
    ),
    (
        Some("123"),
        2,
        "CHAP. XXIII. _Of the_ PUBLIQUE MINISTERS _of Soveraign Power._",
    ),
    (
        Some("127"),
        2,
        "CHAP. XXIV. _Of the_ NUTRITION, _and_ PROCREATION _of a Common-wealth._",
    ),
    (Some("131"), 2, "CHAP. XXV. _Of_ COUNSELL."),
    (Some("136"), 2, "CHAP. XXVI. _Of_ CIVILL LAWES."),
    (
        Some("151"),
        2,
        "CHAP. XXVII. _Of_ CRIMES, EXCUSES, _and_ EXTENUATIONS.",
    ),
    (
        Some("161"),
        2,
        "CHAP. XXVIII. _Of_ PUNISHMENTS, _and_ REWARDS.",
    ),
    (
        Some("167"),
        2,
        "CHAP. XXIX. _Of those things that Weaken, or tend to the_ DISSOLUTION _of a Common-wealth._",
    ),
    (
        Some("175"),
        2,
        "CHAP. XXX. _Of the_ OFFICE _of the Soveraign Representative._",
    ),
    (
        Some("186"),
        2,
        "CHAP. XXXI. _Of the_ KINGDOME OF GOD BY NATURE.",
    ),
    (Some("195"), 1, "OF A CHRISTIAN COMMON-WEALTH."),
    (
        Some("195"),
        2,
        "CHAP. XXXII. _Of the Principles of_ CHRISTIAN POLITIQUES.",
    ),
    (
        Some("199"),
        2,
        "CHAP. XXXIII. _Of the Number, Antiquity, Scope, Authority, and Interpreters of the Books of Holy_ SCRIPTURE.",
    ),
    (
        Some("207"),
        2,
        "CHAP. XXXIV. _Of the Signification of_ SPIRIT, ANGEL, _and_ INSPIRATION _in the Books of Holy Scripture._",
    ),
    (
        Some("216"),
        2,
        "CHAP. XXXV. _Of the Signification in Scripture of_ KINGDOME OF GOD, _of_ HOLY, SACRED, _and_ SACRAMENT.",
    ),
    (
        Some("222"),
        2,
        "CHAP. XXXVI. _Of the_ WORD OF GOD, _and of_ PROPHETS.",
    ),
    (
        Some("233"),
        2,
        "CHAP. XXXVII. _Of_ MIRACLES, _and their Use._",
    ),
    (
        Some("238"),
        2,
        "CHAP. XXXVIII. _Of the Signification in Scripture of_ ETERNALL LIFE, HELL, SALVATION, THE WORLD TO COME, _and_ REDEMPTION.",
    ),
    (
        Some("247b"),
        2,
        "CHAP. XXXIX. _Of the signification in Scripture of the word_ CHURCH.",
    ),
    (
        Some("249"),
        2,
        "CHAP. XL. _Of the_ RIGHTS _of the Kingdome of God, in_ Abraham, Moses, _the_ High Priests, _and the_ Kings of Judah.",
    ),
    (
        Some("261"),
        2,
        "CHAP. XLI. _Of the_ OFFICE _of our BLESSED SAVIOUR._",
    ),
    (Some("267"), 2, "CHAP. XLII. _Of_ POWER ECCLESIASTICALL."),
    (
        Some("321"),
        2,
        "CHAP. XLIII. _Of what is_ NECESSARY _for a Mans Reception into the Kingdome of Heaven._",
    ),
    (Some("333"), 1, "OF THE KINGDOME OF DARKNESSE."),
    (
        Some("333"),
        2,
        "CHAP. XLIV. _Of Spirituall Darknesse from_ MISINTERPRETATION _of Scripture._",
    ),
    (
        Some("352"),
        2,
        "CHAP. XLV. _Of_ DAEMONOLOGY, _and other Reliques of the Religion of the Gentiles._",
    ),
    (
        Some("367"),
        2,
        "CHAP. XLVI. _Of_ DARKNESSE _from_ VAIN PHILOSOPHY, _and_ FABULOUS TRADITIONS.",
    ),
    (
        Some("381"),
        2,
        "CHAP. XLVII. _Of the_ BENEFIT _that proceedeth from such Darknesse, and to whom it accreweth._",
    ),
    (Some("389"), 1, "A _REVIEW,_ and _CONCLUSION._"),
];

pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    TOC.iter()
        .enumerate()
        .map(|(i, (page, depth, label))| (i, page.map(str::to_string), *depth, *label, None))
        .collect()
}
