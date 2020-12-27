use crate::rules::{ResultSet, RuleResult, RuleState};

use std::fmt;
use std::collections::HashMap;

use serde::{Serialize};


#[derive(Debug, Serialize)]
pub struct Report {
    name: String,
    minimum_level: Level,
    #[serde(rename = "messages")]
    messages_by_rule: HashMap<String, Vec<Message>>,
    skipped: usize,
    total: usize,
}

#[derive(Debug, PartialOrd, PartialEq, Clone, Serialize)]
enum Level {
    #[serde(rename = "PASS")]
    Pass = 0,
    #[serde(rename = "WARNING")]
    Warning = 1,
    #[serde(rename = "ERROR")]
    Error = 2
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Level::Pass => write!(f, "PASS"),
            Level::Warning => write!(f, "WARNING"),
            Level::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    line: String,
    message: String,
    level: Level,
    rule: String,
    entity: String,
    entity_name: String
}

impl From<RuleState> for Level {
    fn from(state: RuleState) -> Level {
        match state {
            RuleState::Ok => Level::Pass,
            RuleState::Warning | RuleState::Repaired => Level::Warning,
            RuleState::Error => Level::Error
        }
    }
}

impl<R: fmt::Display> From<(R, RuleResult)> for Message {
    fn from((raw, result): (R, RuleResult)) -> Message {
        let line = format!("{}", raw);
        let level = Level::from(result.state());
        Message {
            line,
            message: result.message,
            level,
            rule: result.rule,
            entity: result.entity,
            entity_name: result.entity_name
        }
    }
}

impl Report {
    pub fn new<S: Into<String>>(name: S) -> Report {
        Report {
            name: name.into(),
            minimum_level: Level::Warning,
            messages_by_rule: HashMap::new(),
            skipped: 0,
            total: 0
        }
    }

    pub fn add_result<R: fmt::Display>(&mut self, original_line: R, result: ResultSet) {
        self.total += 1;
        if result.line_skipped() {
            self.skipped += 1;
        }
        for (rule, a_result) in result.all_results {
            let m: Message = (&original_line, a_result).into();
            if m.level >= self.minimum_level {
                // insert rule -> message into vec by that rule in self.messages_by_rule
                self.messages_by_rule.entry(rule).or_insert_with(Vec::new).push(m);
            }
        }
    }
}

impl Default for Report {
    fn default() -> Report {
        Report::new("")
    }
}

impl fmt::Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut report: String = String::new();
        report.push_str(&format!("## {} Report\n", self.name));
        report.push_str(&format!("* lines: {}\n", self.total));
        report.push_str(&format!("* skipped: {}\n", self.skipped));
        report.push_str(&format!("* valid: {}\n", self.total - self.skipped));

        let mut messages: Vec<(String, Vec<Message>)> = Vec::new();
        for (k, v) in &self.messages_by_rule {
            messages.push((k.clone(), v.clone()));
        }

        messages.sort_by_key(|(k, _)| k.clone());
        
        for (rule, message_list) in messages {
            report.push_str(&format!("### {}\n\n", rule));
            for message in message_list {
                if message.level >= self.minimum_level {
                    let entity_and_name = if message.entity.is_empty() && !message.entity_name.is_empty() {
                        format!("{} for ({})", message.entity_name, message.entity)
                    } else if message.entity_name.is_empty() && !message.entity.is_empty() {
                        format!("({})", message.entity)
                    } else {
                        "".to_string()
                    };
                    report.push_str(&format!("* {} - Violates {}: {} {} -- `{}`\n", message.level, message.rule, message.message, entity_and_name, message.line));
                }
            }
        }
        write!(f, "{}", report)
    }
}

#[cfg(test)]
mod test_report {
    use super::*;
    use crate::resource;
    use crate::annotation::fields::*;
    use crate::annotation::model::*;
    use crate::rules;
    use crate::meta::Context;

    #[test]
    fn test_convert_to_message() {
        let before_assoc = GoAssociation::from((Subject::default(), Curie::new("BFO", "0000050"), Term::new(Curie::new("GO", "1"), None), Evidence::default(), Metadata::default(), Extensions::default()));
        let context = Context::default().add_ontology(resource::load_ontology("resources/alt_id_ont.json").unwrap());
        
        let (_, result_set) = rules::run_rules(before_assoc, &context);

        let rule_20_result = result_set.all_results.get("gorule-0000020").unwrap().to_owned();
        let message: Message = ("`Original Annotation stand-in`".to_string(), rule_20_result).into();

        println!("{:?}", message);
        assert_eq!(message.level, Level::Warning);
        assert_eq!(message.rule, "gorule-0000020");
    }

    #[test]
    fn test_rule_20_message_in_report() {
        let before_assoc = GoAssociation::from((Subject::default(), Curie::new("BFO", "0000050"), Term::new(Curie::new("GO", "1"), None), Evidence::default(), Metadata::default(), Extensions::default()));
        let context = Context::default().add_ontology(resource::load_ontology("resources/alt_id_ont.json").unwrap());
        
        let (_, result_set) = rules::run_rules(before_assoc, &context);

        let mut report = Report::default();
        report.add_result("`Original Annotation stand-in`".to_string(), result_set);

        assert_eq!(report.total, 1);
        assert_eq!(report.messages_by_rule.get("gorule-0000020").unwrap().len(), 1);
    }
}

