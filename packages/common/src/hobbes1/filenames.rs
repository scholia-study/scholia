//! Curated filename roster + slug rules for hobbes1. Filenames are stored
//! verbatim (they were minted once by `hobbes1_tei_to_md` and are now the
//! stable identity of the curated files); `slugify` serves node URL slugs
//! and ltree path segments.

pub const FILENAMES: &[&str] = &[
    "001_engraved_title_page.md",
    "002_title_page.md",
    "003_to_my_most_honor_d_friend_mr_francis_godolphin_of_godolphin.md",
    "004_the_introduction.md",
    "005_of_man.md",
    "006_chap_i_of_sense.md",
    "007_chap_ii_of_imagination.md",
    "008_chap_iii_of_the_consequence_or_trayne_of_imaginations.md",
    "009_chap_iv_of_speech.md",
    "010_chap_v_of_reason_and_science.md",
    "011_chap_vi_of_the_interiour_beginnings_of_voluntary_motions_commonly.md",
    "012_chap_vii_of_the_ends_or_resolutions_of_discourse.md",
    "013_chap_viii_of_the_vertues_commonly_called_intellectuall_and_their.md",
    "014_chap_ix_of_the_severall_subiects_of_knowledge.md",
    "015_chap_x_of_power_worth_dignity_honour_and_worthinesse.md",
    "016_chap_xi_of_the_difference_of_manners.md",
    "017_chap_xii_of_religion.md",
    "018_chap_xiii_of_the_naturall_condition_of_mankind_as_concerning_their.md",
    "019_chap_xiv_of_the_first_and_second_naturall_lawes_and_of_contracts.md",
    "020_chap_xv_of_other_lawes_of_nature.md",
    "021_chap_xvi_of_persons_authors_and_things_personated.md",
    "022_of_common_vvealth.md",
    "023_chap_xvii_of_the_causes_generation_and_definition_of_a_common_wealth.md",
    "024_chap_xviii_of_the_rights_of_soveraignes_by_institution.md",
    "025_chap_xix_of_the_severall_kinds_of_common_wealth_by_institution_and_of.md",
    "026_chap_xx_of_dominion_paternall_and_despoticall.md",
    "027_chap_xxi_of_the_liberty_of_subjects.md",
    "028_chap_xxii_of_systemes_subject_politicall_and_private.md",
    "029_chap_xxiii_of_the_publique_ministers_of_soveraign_power.md",
    "030_chap_xxiv_of_the_nutrition_and_procreation_of_a_common_wealth.md",
    "031_chap_xxv_of_counsell.md",
    "032_chap_xxvi_of_civill_lawes.md",
    "033_chap_xxvii_of_crimes_excuses_and_extenuations.md",
    "034_chap_xxviii_of_punishments_and_rewards.md",
    "035_chap_xxix_of_those_things_that_weaken_or_tend_to_the_dissolution_of_a.md",
    "036_chap_xxx_of_the_office_of_the_soveraign_representative.md",
    "037_chap_xxxi_of_the_kingdome_of_god_by_nature.md",
    "038_of_a_christian_common_wealth.md",
    "039_chap_xxxii_of_the_principles_of_christian_politiques.md",
    "040_chap_xxxiii_of_the_number_antiquity_scope_authority_and_interpreters.md",
    "041_chap_xxxiv_of_the_signification_of_spirit_angel_and_inspiration_in.md",
    "042_chap_xxxv_of_the_signification_in_scripture_of_kingdome_of_god_of.md",
    "043_chap_xxxvi_of_the_word_of_god_and_of_prophets.md",
    "044_chap_xxxvii_of_miracles_and_their_use.md",
    "045_chap_xxxviii_of_the_signification_in_scripture_of_eternall_life_hell.md",
    "046_chap_xxxix_of_the_signification_in_scripture_of_the_word_church.md",
    "047_chap_xl_of_the_rights_of_the_kingdome_of_god_in_abraham_moses_the.md",
    "048_chap_xli_of_the_office_of_our_blessed_saviour.md",
    "049_chap_xlii_of_power_ecclesiasticall.md",
    "050_chap_xliii_of_what_is_necessary_for_a_mans_reception_into_the.md",
    "051_of_the_kingdome_of_darknesse.md",
    "052_chap_xliv_of_spirituall_darknesse_from_misinterpretation_of_scripture.md",
    "053_chap_xlv_of_daemonology_and_other_reliques_of_the_religion_of_the.md",
    "054_chap_xlvi_of_darknesse_from_vain_philosophy_and_fabulous_traditions.md",
    "055_chap_xlvii_of_the_benefit_that_proceedeth_from_such_darknesse_and_to.md",
    "056_a_review_and_conclusion.md",
];

pub fn all_filenames() -> Vec<(usize, String)> {
    FILENAMES
        .iter()
        .enumerate()
        .map(|(i, f)| (i, f.to_string()))
        .collect()
}

pub fn position_number(flat_index: usize) -> usize {
    flat_index + 1
}

/// ASCII slug: lowercased alphanumeric runs joined by `_`, `&` → "and".
pub fn slugify(label: &str) -> String {
    let mut s = String::new();
    let mut last_us = true;
    for ch in label.chars() {
        if ch == '&' {
            if !last_us {
                s.push('_');
            }
            s.push_str("and");
            last_us = false;
        } else if ch.is_ascii_alphanumeric() {
            s.push(ch.to_ascii_lowercase());
            last_us = false;
        } else if !last_us {
            s.push('_');
            last_us = true;
        }
    }
    s.trim_matches('_').to_string()
}
