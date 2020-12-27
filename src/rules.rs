use crate::annotation::model::{GoAssociation};
use crate::meta::Context;
use crate::ontology::{NodeDeprecated};
use crate::annotation::fields::*;

use std::collections::HashMap;



#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum RuleState {
    Ok,
    Warning,
    Repaired,
    Error,
}

/// RuleResult is the resulting information from running a rule on a `GoAssociation`. 
#[derive(Debug, Clone, PartialEq)]
pub struct RuleResult {
    /// This is the id of the GO Rule: like gorule-0000001
    pub rule: String,
    /// False if the rule failed, true if it passed
    pub valid: bool,
    /// This is a message about the rule failure, typically just the rule title
    pub message: String,
    /// The entity is the string representation of the part(s) of the `GoAssociation` that failed the rule.
    /// For example, if the evidence type caused a rule failure, this might be the found bad evidence ID.
    pub entity: String,
    /// The name of the entity that failed, refering to `entity`. For example, if the evidence type caused
    /// a rule failure, then the `entity` would be the ECO ID of the bad evidenece, and `entity_name` could
    /// be `"evidence"`
    pub entity_name: String,
    /// The level of the failure. These correspond to the RuleTagResult, as well as to the Message Level.
    /// In general, Error RuleState will be filtered.
    pub state: RuleState,
}

impl RuleResult {
    pub fn new<S: Into<String>>(id: S, message: S, entity: S, entity_name: S, valid: bool, state: RuleState) -> RuleResult {
        RuleResult {
            rule: id.into(),
            valid,
            message: message.into(),
            entity_name: entity_name.into(),
            entity: entity.into(),
            state,
        }
    }

    pub fn state(&self) -> RuleState {
        self.state
    }
}

/// This contains a map from rule IDs (like `gorule-0000001`) to the RuleResult of running that rule
/// on a GoAssociation
#[derive(Debug)]
pub struct ResultSet {
    /// A map from Rule ID to RuleResult, which internal association
    /// in the map point at this ResultSet association
    pub all_results: HashMap<String, RuleResult>
}

impl ResultSet {
    pub fn new() -> ResultSet{
        ResultSet {
            all_results: HashMap::default()
        }
    }

    /// Add a single result. The given `result`'s rule field will be used to insert into the `all_results` map.
    pub fn add_result(&mut self, result: RuleResult) {
        self.all_results.insert(result.rule.clone(), result);
    }

    /// Add an many results. Takes a type that implements `IntoIterator` where the Item type
    /// is a tuple of (String, RuleResult), where the first element is the rule ID.
    pub fn add_results<It: IntoIterator<Item=(String, RuleResult)>>(&mut self, results: It) {
        self.all_results.extend(results.into_iter())
    }

    /// Since a line is skipped/filtered if it violates a rule as `RuleState::Error`, this will check
    /// if there are any `RuleResult`s as values that have RuleState::Error. True if if there are any,
    /// false if there are none.
    pub fn line_skipped(&self) -> bool {
        self.all_results.values().any(|r| r.state == RuleState::Error)
    }

    /// Gets the worst RuleState seen in this set of results.
    /// Goes Ok -> Warning -> Repair -> Error
    pub fn worst_level_state(&self) -> Option<RuleState> {
        let mut worst = None;
        for r in self.all_results.values() {
            if worst.is_none() || r.state > worst.unwrap() {
                worst = Some(r.state)
            }
        }
        worst
    }
}

impl Default for ResultSet {
    fn default() -> ResultSet {
        ResultSet::new()
    }
}

pub struct RuleMeta {
    pub rule_id: String,

    pub description: String
}

/// When implementing rules, use this to return the final state of the rule.
pub enum RuleTagResult {
    Pass(GoAssociation),
    Warning(GoAssociation, String, String),
    Repair(GoAssociation, String, String),
    Error(String, String)
}

///
/// Validation says whether or not the rule was passed. 
/// We can respond with Repaired(assoc, name, offending), Pass(assoc), Warning(assoc, name, offending), Error(name, offending).
/// The metadata of a Rule can be generated with `description()` and `id()`. `description` should correspond to the
/// title of the rule as defined in the YAML metadata in `github.com/geneontology/go-site`. `id` should correspond to the
/// integer value of the rule, so `gorule-0000001` would be `1`, etc.
/// 
/// In general, implementing `rule_impl` is sufficient to define the logic of the rule, taking a `GoAssociation` and a `Context`. 
/// A rule implementation can then return one of the RuleTagResult variants, corresponding to how the association conforms to
/// the GO Rule. `Repair` should contain the updated annotation.
/// 
/// Rules are actually run with the `validate` function, but this is implemented by default by calling `rule_impl` and handling
/// the boilerplate of the full, final RuleResult paired with the corresponding GoAssociation. The `id` and `description` are used
/// indirectly through the default implementation of `meta()` yielding a `RuleMeta` type continaing everyhing `validate` needs to 
/// fully make a `RuleResult`.
/// 
/// Example:
/// 
/// ```
/// struct Rule99;
/// impl Rule for Rule99 {
///     fn id(&self) -> u32 { 99 }
/// 
///     fn description(&self) -> &'static str { "Data should conform to this example" }
/// 
///     fn rule_impl(&self, association: GoAssociation, context: &Context) -> RuleTagResult {
///         if association.object.id == Curie::new("GO", "1234567") {
///             RuleTagResult::Warning(association, "GO term".into(), "GO:1234567".into())
///         } else {
///             RuleTagResult::Pass(association)
///         }
///     }
/// }
/// ```
///
pub trait Rule {
    fn validate(&self, association: GoAssociation, context: &Context) -> (GoAssociation, RuleResult) {
        match self.rule_impl(association.clone(), context) {
            RuleTagResult::Pass(assoc) => {
                (assoc, RuleResult::new(self.meta().rule_id, self.meta().description, "".into(), "".into(), true, RuleState::Ok))
            },
            RuleTagResult::Warning(assoc, name, offending) => {
                (assoc, RuleResult::new(self.meta().rule_id, self.meta().description, offending, name, true, RuleState::Warning))
            },
            RuleTagResult::Repair(assoc, name, offending) => {
                (assoc, RuleResult::new(self.meta().rule_id, self.meta().description, offending, name, true, RuleState::Repaired))
            },
            RuleTagResult::Error(name, offending) => {
                (association, RuleResult::new(self.meta().rule_id, self.meta().description, offending, name, true, RuleState::Warning))
            }
        }
    }

    fn rule_impl(&self, association: GoAssociation, context: &Context) -> RuleTagResult;

    fn description(&self) -> &'static str;

    fn id(&self) -> u32;

    fn meta(&self) -> RuleMeta {
        RuleMeta {
            description: self.description().to_string(),
            rule_id: format!("gorule-{:0width$}", self.id(), width=7)
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Rule02;

impl Rule for Rule02 {
    fn description(&self) -> &'static str {"No 'NOT' annotations to 'protein binding ; GO:0005515'"}
    
    fn id(&self) -> u32 {2}

    fn rule_impl(&self, association: GoAssociation, _: &Context) -> RuleTagResult {
        let goterm = Curie::new("GO", "0005515");
        if association.object.id == goterm && association.negated {
            RuleTagResult::Warning(association, "GO Term".into(), goterm.to_string())
        } else {
            RuleTagResult::Pass(association)
        }
    }
}

#[derive(Debug, Clone)]
struct Rule11(Vec<Curie>, Curie);

impl Default for Rule11 {
    fn default() -> Rule11 {
        Rule11(vec![Curie::new("GO", "0003674"), Curie::new("GO", "0005575"), Curie::new("GO", "0008150")], Curie::new("ECO", "0000307"))
    }
}

impl Rule for Rule11 {
    fn description(&self) -> &'static str {"ND evidence code should be to root nodes only, and no terms other than root nodes can have the evidence code ND"}

    fn id(&self) -> u32 {11}

    fn rule_impl(&self, association: GoAssociation, _: &Context) -> RuleTagResult {
        if (self.0.contains(&association.object.id) && association.evidence.id == self.1) 
                || (!self.0.contains(&association.object.id) && association.evidence.id != self.1) {

            RuleTagResult::Pass(association)
        } else {
            RuleTagResult::Error("GO Term".into(), association.object.id.to_string())
        }
    }
}

#[derive(Debug, Clone)]
struct Rule18;

impl Rule for Rule18 {
    fn description(&self) -> &'static str { "IPI annotations require a With/From entry" }

    fn id(&self) -> u32 {18}

    fn rule_impl(&self, association: GoAssociation, _: &Context) -> RuleTagResult {
        let ipi = Curie::new("ECO", "0000353");
        if association.evidence.id == ipi {
            // If evidence is IPI, then we should expect a withfrom entry
            if association.evidence.with_support_from.items().is_empty() {
                RuleTagResult::Pass(association)
            } else {
                RuleTagResult::Warning(association, "with/from".into(), "Empty".into())
            }
        } else {
            RuleTagResult::Pass(association)
        }
    }
}

#[derive(Debug, Default, Clone)]
struct Rule20;

impl Rule for Rule20 {

    fn description(&self) -> &'static str {"Automatic repair of annotations to merged or obsoleted terms"}

    fn id(&self) -> u32 {20}

    fn rule_impl(&self, mut association: GoAssociation, context: &Context) -> RuleTagResult {
        let go_uri = context.uri_mapping.uri_for_curie(&association.object.id).unwrap();
        let node = context.ontology.node(go_uri);
        match node {
            Some(node) => {
                if node.deprecated() {
                    match node.replaced_by() {
                        Some(replaced) => {
                            let repl_curie = context.uri_mapping.curie_for_uri(&replaced).expect("This is a GO URI, GO included by default");
                            association.object.id = repl_curie;
                            let goterm = association.object.id.to_string();
                            RuleTagResult::Repair(association, "GO term repaired".into(), goterm)
                        },
                        None => {
                            RuleTagResult::Error("GO term could not be repaired".into(), association.object.id.to_string())
                        }
                    }
                } else {
                    RuleTagResult::Pass(association)
                }
            },
            None => {
                RuleTagResult::Error("GO term is not in ontology".into(), association.object.id.to_string())
            }
        }
    }
}

fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(Rule02),
        Box::new(Rule11::default()),
        Box::new(Rule20)
    ]
}

pub fn run_rules(association: GoAssociation, context: &Context) -> (GoAssociation, ResultSet) {
    let mut results: Vec<(String, RuleResult)> = vec![];
    let mut current_association = association;
    for rule in rules() {
        let (validated_assoc, result) = rule.validate(current_association, context);
        current_association = validated_assoc;
        results.push((result.rule.clone(), result));
    }
    let mut result_set = ResultSet::new();
    result_set.add_results(results);
    (current_association, result_set)
}

#[cfg(test)]
mod test_rules {
    use super::*;
    use crate::annotation::model::*;
    use crate::resource;

    #[test]
    fn test_rule_20() {
        let before_assoc = GoAssociation::from((Subject::default(), Curie::new("BFO", "0000050"), Term::new(Curie::new("GO", "1"), None), Evidence::default(), Metadata::default(), Extensions::default()));
        let rule20 = Rule20;
        let context = Context::default().add_ontology(resource::load_ontology("resources/alt_id_ont.json").unwrap());
        
        let (assoc, result) = rule20.validate(before_assoc, &context);
        assert_eq!(assoc.object.id, Curie::new("GO", "2"));
        assert_eq!(result.state, RuleState::Repaired);
    }

    #[test]
    fn test_all_rules_with_just_rule20() {
        let before_assoc = GoAssociation::from((Subject::default(), Curie::new("BFO", "0000050"), Term::new(Curie::new("GO", "1"), None), Evidence::default(), Metadata::default(), Extensions::default()));
        let context = Context::default().add_ontology(resource::load_ontology("resources/alt_id_ont.json").unwrap());
        
        let (assoc, result_set) = run_rules(before_assoc, &context);
        
        assert_eq!(result_set.all_results.get("gorule-0000020").unwrap().state, RuleState::Repaired);
        assert_eq!(assoc.object.id, Curie::new("GO", "2"));
    }
}
