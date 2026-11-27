//! English node slugs for the Doctrine of Being 1812
//! translation edition, one per `toc` entry in document order. Frozen:
//! generated once from the English front-matter labels (same slugify and
//! counter-suffix rule as `toc`'s German slugs), so later label edits never
//! move URLs.

/// English slugs, one per `toc` entry, in document order.
pub const EN_SLUGS: &[&str] = &[
    "preface",
    "introduction",
    "logic",
    "on_its_general_division",
    "book_one_being",
    "with_what_must_the_beginning_of_science_be_made",
    "general_division_of_being",
    "first_section_determinateness",
    "chapter_1_being",
    "a_being",
    "b_nothing",
    "c_becoming",
    "unity_of_being_and_nothing",
    "remark_1",
    "remark_2",
    "remark_3",
    "remark_4",
    "2_moments_of_becoming",
    "3_sublation_of_becoming",
    "remark",
    "chapter_2_existence",
    "a_existence_as_such",
    "1_existence_in_general",
    "2_reality",
    "a_otherness",
    "b_being_for_other_and_being_in_itself",
    "c_reality",
    "remark_5",
    "3_something",
    "b_determinateness",
    "1_limit",
    "2_determinateness",
    "a_determination",
    "b_constitution",
    "c_quality",
    "remark_6",
    "3_alteration",
    "a_alteration_of_constitution",
    "b_the_ought_and_restriction",
    "remark_7",
    "c_negation",
    "remark_8",
    "c_qualitative_infinity",
    "1_finitude_and_infinity",
    "2_reciprocal_determination_of_the_finite_and_the_infinite",
    "3_return_of_infinity_into_itself",
    "remark_9",
    "chapter_3_being_for_itself",
    "a_being_for_itself_as_such",
    "1_being_for_itself_in_general",
    "2_the_moments_of_being_for_itself",
    "a_the_moment_of_its_being_in_itself",
    "b_being_for_one",
    "remark_10",
    "c_ideality",
    "3_becoming_of_the_one",
    "b_the_one",
    "1_the_one_and_the_void",
    "remark_11",
    "2_many_ones",
    "remark_12",
    "3_mutual_repulsion",
    "c_attraction",
    "1_one_one",
    "2_equilibrium_of_attraction_and_repulsion",
    "remark_13",
    "3_transition_to_quantity",
    "second_section_magnitude",
    "remark_14",
    "chapter_1_quantity",
    "a_pure_quantity",
    "remark_1_2",
    "remark_2_2",
    "b_continuous_and_discrete_magnitude",
    "remark_15",
    "c_limitation_of_quantity",
    "chapter_2_quantum",
    "a_number",
    "remark_1_3",
    "remark_2_3",
    "b_extensive_and_intensive_quantum",
    "1_their_difference",
    "2_identity_of_extensive_and_intensive_magnitude",
    "remark_16",
    "3_alteration_of_the_quantum",
    "c_quantitative_infinity",
    "1_its_concept",
    "2_the_infinite_progress",
    "remark_1_4",
    "remark_2_4",
    "3_infinity_of_the_quantum",
    "remark_17",
    "chapter_3_the_quantitative_ratio",
    "a_the_direct_ratio",
    "b_the_inverse_ratio",
    "c_the_ratio_of_powers",
    "remark_18",
    "third_section_measure",
    "chapter_1_specific_quantity",
    "a_the_specific_quantum",
    "b_the_rule",
    "1_the_qualitative_and_quantitative_determinateness_of_magnitude",
    "2_quality_and_quantum",
    "3_distinction_of_both_sides_as_qualities",
    "remark_19",
    "c_relation_of_qualities",
    "chapter_2_relation_of_self_subsistent_measures",
    "a_the_relation_of_self_subsistent_measures",
    "1_neutrality",
    "2_specification_of_neutrality",
    "3_elective_affinity",
    "remark_20",
    "b_nodal_line_of_measure_relations",
    "remark_21",
    "c_the_measureless",
    "chapter_3_the_becoming_of_essence",
    "a_indifference",
    "b_the_self_subsistent_as_inverse_ratio_of_its_factors",
    "remark_22",
    "c_emergence_of_essence",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hegel3::toc;
    use std::collections::HashSet;

    #[test]
    fn one_slug_per_toc_entry() {
        assert_eq!(EN_SLUGS.len(), toc::toc_len());
    }

    #[test]
    fn slugs_are_unique() {
        let unique: HashSet<_> = EN_SLUGS.iter().collect();
        assert_eq!(unique.len(), EN_SLUGS.len());
    }

    #[test]
    fn slugs_are_ltree_safe() {
        for slug in EN_SLUGS {
            assert!(
                !slug.is_empty()
                    && slug
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "bad slug {slug:?}"
            );
        }
    }
}
