use crate::annotation::model;
use crate::annotation::{RawGaf2_1Record, BaseGaf2_1Row};
use crate::rules;
use crate::rules::{ResultSet, RuleResult, RuleState};
use crate::meta::Context;
use crate::report::Report;


/// TODO do we want to use this at all??
// enum ValidateResult<T, R, W, E> {
//     Ok(T),
//     Repair(R),
//     Warning(W),
//     Error(E)
// }

// enum ResultMessage<A, M> {
//     Just(A),
//     Info(A, M)
// }

impl From<String> for RuleResult {
    fn from(error: String) -> RuleResult {
        RuleResult::new("gorule-0000001", &error, "", "", false, RuleState::Error)
    }
}

/// TODO how to genericize?
/// Validation takes a `RawGaf2_1Record` and a `Context` and produces a tuple with the original line, an optional
/// `GoAssociation`, and the `ResultSet` from the rules.
/// 
/// This works by invoking the `convert_raw` function in the `model` module and then running any `GoAssociation` produced
/// into the Rules.
/// 
/// If there are errors in parsing, these get wrapped up as a `ResultSet` as well.
/// 
/// The Optional `GoAssociation` is None if the original line could not be parsed or if there were any ERROR rules
pub fn validate_gaf_2_1(line: RawGaf2_1Record, context: &Context) -> (RawGaf2_1Record, Option<model::GoAssociation>, ResultSet) {
    let original = line.clone();

    let association = model::convert_raw::<RawGaf2_1Record, BaseGaf2_1Row>(line, context);
    let (results, maybe_assoc) = match association {
        Ok(assoc) => {
            let (assoc, result_set) = rules::run_rules(assoc, context);
            if result_set.worst_level_state() == Some(RuleState::Error) {
                // Don't return the assoc if there is an Error
                // Should this be in the rules somehow?
                (result_set, None)
            } else {
                (result_set, Some(assoc))
            }
        },
        Err(err) => {
            let mut rule_set = ResultSet::new();
            rule_set.add_result(RuleResult::from(err));
            (rule_set, None)
        }
    };
    (original, maybe_assoc, results)
}

///
/// Wraps `validate_gaf_2_1`, but takes an existing mutable Report. Results from `validate_gaf_2_1` are then added to the report,
/// and the Option `GoAssociation` is returned along with the updated report.
/// 
/// Note: The report is being passed in directly, and not as a mutable reference. This is in part why we return it again
/// as per rust's ownership rules. This function takes ownership of the report, and then gives ownership back by
/// returning the given report.
pub fn parse_and_report_gaf_2_1(line: RawGaf2_1Record, context: &Context, mut report: Report) -> (Option<model::GoAssociation>, Report) {
    let (original, association, result) = validate_gaf_2_1(line, context);
    report.add_result(original, result);
    (association, report)
}
