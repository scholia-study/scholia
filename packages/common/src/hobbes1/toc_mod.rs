//! Modernized-layer TOC labels for hobbes1, one per file in position order —
//! the modern-spelling forms of the 1651 heads (drives reader TOC labels and
//! node slugs). Pages/depths are shared with `toc::TOC`; only labels differ.

pub const MODERNIZED_LABELS: &[&str] = &[
    "Engraved Title Page",
    "Title Page",
    "TO MY MOST HONOR'D FRIEND Mr _FRANCIS GODOLPHIN_ of _Godolphin._",
    "THE INTRODUCTION.",
    "OF MAN.",
    "CHAP. I. _Of_ SENSE.",
    "CHAP. II. _Of_ IMAGINATION.",
    "CHAP. III. _Of the Consequence or_ TRAIN _of Imaginations._",
    "CHAP. IV. _Of_ SPEECH.",
    "CHAP. V. _Of_ REASON, and SCIENCE.",
    "CHAP. VI. _Of the Interior Beginnings of Voluntary Motions; commonly called the_ PASSIONS. _And the Speeches by which they are expressed._",
    "CHAP. VII. _Of the_ Ends, _or_ Resolutions _of_ DISCOURSE.",
    "CHAP. VIII. _Of the_ VIRTUES _commonly called_ INTELLECTUAL; _and their contrary_ DEFECTS.",
    "CHAP. IX. _Of the Several_ SUBJECTS _of_ KNOWLEDGE.",
    "CHAP. X. _Of_ POWER, WORTH, DIGNITY, HONOUR, _and_ WORTHINESS.",
    "CHAP. XI. _Of the difference of_ MANNERS.",
    "CHAP. XII. OF RELIGION.",
    "CHAP. XIII. _Of the_ NATURAL CONDITION _of Mankind, as concerning their Felicity, and Misery._",
    "CHAP. XIV. _Of the first and second_ NATURAL LAWS, _and of_ CONTRACTS.",
    "CHAP. XV. _Of other Laws of Nature._",
    "CHAP. XVI. _Of_ PERSONS, AUTHORS, _and things Personated._",
    "OF COMMON-WEALTH.",
    "CHAP. XVII. _Of the Causes, Generation, and Definition of a_ COMMON-WEALTH.",
    "CHAP. XVIII. _Of the_ RIGHTS _of Sovereigns by Institution._",
    "CHAP. XIX. _Of the several Kinds of_ Common-wealth _by Institution, and of Succession to the Sovereign Power._",
    "CHAP. XX. _Of Dominion_ PATERNAL, _and_ DESPOTICAL.",
    "CHAP. XXI. _Of the_ LIBERTY _of Subjects._",
    "CHAP. XXII. _Of_ SYSTEMS _Subject, Political, and Private._",
    "CHAP. XXIII. _Of the_ PUBLIC MINISTERS _of Sovereign Power._",
    "CHAP. XXIV. _Of the_ NUTRITION, _and_ PROCREATION _of a Common-wealth._",
    "CHAP. XXV. _Of_ COUNSEL.",
    "CHAP. XXVI. _Of_ CIVIL LAWS.",
    "CHAP. XXVII. _Of_ CRIMES, EXCUSES, _and_ EXTENUATIONS.",
    "CHAP. XXVIII. _Of_ PUNISHMENTS, _and_ REWARDS.",
    "CHAP. XXIX. _Of those things that Weaken, or tend to the_ DISSOLUTION _of a Common-wealth._",
    "CHAP. XXX. _Of the_ OFFICE _of the Sovereign Representative._",
    "CHAP. XXXI. _Of the_ KINGDOM OF GOD BY NATURE.",
    "OF A CHRISTIAN COMMON-WEALTH.",
    "CHAP. XXXII. _Of the Principles of_ CHRISTIAN POLITICS.",
    "CHAP. XXXIII. _Of the Number, Antiquity, Scope, Authority, and Interpreters of the Books of Holy_ SCRIPTURE.",
    "CHAP. XXXIV. _Of the Signification of_ SPIRIT, ANGEL, _and_ INSPIRATION _in the Books of Holy Scripture._",
    "CHAP. XXXV. _Of the Signification in Scripture of_ KINGDOM OF GOD, _of_ HOLY, SACRED, _and_ SACRAMENT.",
    "CHAP. XXXVI. _Of the_ WORD OF GOD, _and of_ PROPHETS.",
    "CHAP. XXXVII. _Of_ MIRACLES, _and their Use._",
    "CHAP. XXXVIII. _Of the Signification in Scripture of_ ETERNAL LIFE, HELL, SALVATION, THE WORLD TO COME, _and_ REDEMPTION.",
    "CHAP. XXXIX. _Of the signification in Scripture of the word_ CHURCH.",
    "CHAP. XL. _Of the_ RIGHTS _of the Kingdom of God, in_ Abraham, Moses, _the_ High Priests, _and the_ Kings of Judah.",
    "CHAP. XLI. _Of the_ OFFICE _of our BLESSED SAVIOUR._",
    "CHAP. XLII. _Of_ POWER ECCLESIASTICAL.",
    "CHAP. XLIII. _Of what is_ NECESSARY _for a Man's Reception into the Kingdom of Heaven._",
    "OF THE KINGDOM OF DARKNESS.",
    "CHAP. XLIV. _Of Spiritual Darkness from_ MISINTERPRETATION _of Scripture._",
    "CHAP. XLV. _Of_ DEMONOLOGY, _and other Relics of the Religion of the Gentiles._",
    "CHAP. XLVI. _Of_ DARKNESS _from_ VAIN PHILOSOPHY, _and_ FABULOUS TRADITIONS.",
    "CHAP. XLVII. _Of the_ BENEFIT _that proceedeth from such Darkness, and to whom it accrueth._",
    "A _REVIEW,_ and _CONCLUSION._",
];

pub fn flat_toc_entries() -> Vec<crate::FlatTocEntry> {
    super::toc::TOC
        .iter()
        .zip(MODERNIZED_LABELS)
        .enumerate()
        .map(|(i, ((page, depth, _), label))| (i, page.map(str::to_string), *depth, *label, None))
        .collect()
}
