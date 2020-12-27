//!
//! This module contains the data model for line annotation data.
//! `Curie` represents a Compact URI where there is a namespace and identifier, like GO:1234567
//! 
//! `ListField` reprsents a field that is a list separated by the pipe (`|`)
//! 
//! `EcoCode` is the GAF-centric enum for all evidence codes that can be turned into ECO Curies.
//! 
//! `Aspect` is the GAF-centric enum for CellComponent, MolecularFunction, or BioProcess
//! 
//! `NoSpaceString` is a string that should have no strings. Parsing returns an error if there is a space
//! in the string.
//! 
//! `EitherOrBoth` is an enum reprsenting a Left value, a Right Value, or both, separated by a pipe (`|`).
//! This is used for GAF Qualifier field, for example.
//! 
//! `OneOrTwoItems` is an enum reprsenting a field with one item of a type, or two of the same type separated by a pipe (`|`).
//! This is used for GAF taxon field, where the second item can be the "interacting taxon".
//! 
//! `Not` represents a value for "NOT" in the qualifier. This is the type of the Left field in the EitherOrBoth type.
//! 
//! `SingleChar` parses only one character. This is used to parse the aspect.
//! 
//! `Label` A string value
//! 
//! `ClassExpression` is a general `<relation>(<filler>)`, where the filler is inside some parenthases after the relation. 
//! This is used in the extensions field. 
//! 
//! `Conjunction` represents a list of values separated by a comma. This is used in the extensions field.
//! 
//! `Property` represents a key-value pair, as seen in GPAD, for example. 
//! 

use std::convert::TryFrom;
use std::str::FromStr;
use std::fmt;
use chrono::prelude::*;
use chrono::Utc;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Curie {
    pub namespace: String,
    pub identifier: String
}

impl Curie {
    pub fn new<S: Into<String>>(namespace: S, identifier: S) -> Curie {
        Curie {
            namespace: namespace.into(),
            identifier: identifier.into()
        }
    }

    pub fn same_namespace<S: PartialEq<String>>(&self, ns: S) -> bool {
        ns == self.namespace
    }

    pub fn string(&self) -> String {
        format!("{}:{}", self.namespace, self.identifier)
    }
}

impl fmt::Display for Curie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string())
    }
}

impl TryFrom<&str> for Curie {
    type Error = String;

    fn try_from(entity: &str) -> Result<Curie, Self::Error> {
        let split: Vec<&str> = entity.splitn(2, ':').collect();
        match split.as_slice() {
            [_] => Err("Curies cannot be empty. They take the form `Namespace:Identifier`".into()),
            [first, second] if (*first, *second) == ("", "") => Err("Curies cannot be empty. They take the form `Namespace:Identifier`".into()),
            [first, _] if first.is_empty() => Err("Curie Namespaces cannot be empty".into()),
            [_, second] if second.is_empty() => Err("Curie Identifiers cannot be empty".into()),
            [namespace, identifier] => Ok(Curie {namespace: String::from(*namespace), identifier: String::from(*identifier)}),
            _ => Err("Nope".into())
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListField<I> {
    items: Vec<I>
}

impl<I> ListField<I> {
    pub fn new(items: Vec<I>) -> ListField<I> {
        ListField {
            items
        }
    }

    pub fn items(&self) -> &[I] {
        self.items.as_slice()
    }

    pub fn map_new<T, M>(&self, m: M) -> ListField<T>
        where M: FnMut(&I) -> T {
        
        let new_items: Vec<T> = self.items.iter().map(m).collect();
        ListField { items: new_items }
    }

    pub fn map_new_results<T, M, E>(&self, m: M) -> Result<ListField<T>, E>
        where M: FnMut(&I) -> Result<T, E> {

        let new_items: Result<Vec<T>, E> = self.items.iter().map(m).collect();
        new_items.map(|items| ListField { items })
    }
}

impl<I: fmt::Display> fmt::Display for ListField<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.items.iter().map(|i| i.to_string()).collect::<Vec<String>>().join("|");
        write!(f, "{}", s)
    }
}

impl<'a, I: Clone + std::fmt::Debug + TryFrom<&'a str, Error=String>> TryFrom<&'a str> for ListField<I> {
    type Error = String;

    fn try_from(entity: &'a str) -> Result<ListField<I>, Self::Error> {
        if entity.is_empty() {
            return Ok(ListField::new(vec![]))
        }

        let (parsed, errors): (Vec<_>, Vec<_>) = entity.split('|')
            .map(|el| I::try_from(el))
            .partition(Result::is_ok);
        
        let errors: Vec<String> = errors.into_iter()
            .map(Result::unwrap_err)
            .collect();
        
        if !errors.is_empty() {
            Err(format!("Errors parsing `{}`: {}", entity, errors.join("; ")))
        } else {
            Ok(ListField {
                items: parsed.into_iter()
                    .map(Result::unwrap)
                    .collect()
            })
        }
    }
}

///
/// ```
/// assert_eq!(format!("{:?}", EcoCode::EXP), "EXP");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum EcoCode {
    EXP,
    IDA,
    IPI,
    IMP,
    IMR,
    IGI,
    IEP,
    HTP,
    HDA,
    HMP,
    HGI,
    HEP,
    IBA,
    IBD,
    IKR,
    IRD,
    ISS,
    ISO,
    ISA,
    ISM,
    IGC,
    RCA,
    TAS,
    NAS,
    IC,
    ND,
    IEA,
}

impl fmt::Display for EcoCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for EcoCode {
    type Error = String;

    fn try_from(entity: &str) -> Result<EcoCode, String> {
        for code in EcoCode::iter() {
            // TODO is format! here slow? I think it's allocating which is kinda lame
            if format!("{:?}", code) == entity {
                return Ok(code)
            }
        }
        Err(format!("ECO code `{}` not found", entity))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Aspect {
    CellComponent,
    MolecularFunction,
    BioProcess
}

impl Aspect {
    pub fn as_char(&self) -> char {
        match self {
            Aspect::CellComponent => 'C',
            Aspect::MolecularFunction => 'F',
            Aspect::BioProcess => 'P'
        }
    }
}

impl fmt::Display for Aspect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

impl TryFrom<char> for Aspect {
    type Error = String;

    fn try_from(entity: char) -> Result<Aspect, String> {
        match entity {
            'C' => Ok(Aspect::CellComponent),
            'F' => Ok(Aspect::MolecularFunction),
            'P' => Ok(Aspect::BioProcess),
            _ => Err(format!("Aspect must be `C`, `F`, or `P`, but received `{}`", entity))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoSpaceString {
    pub value: String
}

impl NoSpaceString {
    pub fn new<S: Into<String>>(s: S) -> NoSpaceString {
        NoSpaceString {
            value: s.into()
        }
    }
}

impl TryFrom<&str> for NoSpaceString {
    type Error = String;

    fn try_from(entity: &str) -> Result<NoSpaceString, Self::Error> {
        if entity.contains(' ') {
            Err(String::from("Spaces are not allowed"))
        } else {
            Ok(NoSpaceString::new(entity))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EitherOrBoth<L, R> {
    Left(L),
    Right(R),
    Both(L, R)
}

impl<'a, L, R> TryFrom<&'a str> for EitherOrBoth<L, R> 
    where 
        L: TryFrom<&'a str, Error=String>,
        R: TryFrom<&'a str, Error=String> {
    
    type Error = String;
    
    ///
    /// This tries to match an L, and failing, then an R.
    /// If we match an L, then the iterator advances and we try matching an R.
    fn try_from(entity: &'a str) -> Result<EitherOrBoth<L, R>, Self::Error> {
        let mut split = entity.splitn(2, '|');

        if let Some(first) = split.next() {
            // let first_string = String::from(first);
            match L::try_from(first) {
                Ok(an_l) => {
                    // Here we correctly found a Left on the first item, so now we have try the second
                    if let Some(second) = split.next() {
                        // We found a second item, try to parse into Right
                        match R::try_from(second) {
                            Ok(an_r) => {
                                // matched a Right as well as a Left, so we have Both
                                Ok(EitherOrBoth::Both(an_l, an_r))
                            },
                            Err(right_err) => {
                                // We found a second item, but it failed to parse into Right, so this is an error
                                Err(format!("Failed to parse {}: {}", entity, right_err))
                            }
                        }
                    } else {
                        Ok(EitherOrBoth::Left(an_l))
                    }
                },
                Err(left_err) => {
                    // This branch means it's either an error, or an R
                    // So try to make an R
                    match R::try_from(first) {
                        Ok(an_r) => {
                            // Here we found a correct Right on the first item, so we're also done, and we found a Right
                            Ok(EitherOrBoth::Right(an_r))
                        },
                        Err(right_err) => {
                            // Both chances to match Left and Right failed, so we're done I guess
                            Err(format!("Failed to parse {}: {} or {}", entity, left_err, right_err))
                        }
                    }
                }
            }
        } else {
            unreachable!("The split iterator should always have at least one element in it");
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OneOrTwoItems<I> {
    One(I),
    Two(I, I)
}

impl<I: fmt::Display> fmt::Display for OneOrTwoItems<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OneOrTwoItems::One(one) => write!(f, "{}", one),
            OneOrTwoItems::Two(first, second) => write!(f, "{}|{}", first, second)
        }
    }
}

impl<'a, I: TryFrom<&'a str, Error=String>> TryFrom<&'a str> for OneOrTwoItems<I> {
    type Error = String;

    fn try_from(entity: &'a str) -> Result<OneOrTwoItems<I>, Self::Error> {
        let mut split = entity.splitn(2, '|');

        if let Some(first) = split.next() {
            // let first_string = String::from(first);
            match I::try_from(first) {
                Ok(one) => {
                    // We matched an instance, now lets try again on the next split element
                    if let Some(second) = split.next() {
                        // let second_string = String::from(second);
                        match I::try_from(second) {
                            Ok(two) => {
                                // Here we have found both items
                                Ok(OneOrTwoItems::Two(one, two))
                            },
                            Err(two_err) => {
                                // The second item failed, so we bail
                                Err(format!("Error parsing {}: {}", entity, two_err))
                            }
                        }
                    } else {
                        // No second item, so we just use the One
                        Ok(OneOrTwoItems::One(one))
                    }
                },
                Err(one_err) => {
                    // Could not match the first item, we're done
                    Err(format!("Error parsing {}: {}", entity, one_err))
                }
            }
        } else {
            unreachable!("There split iterator should always have at least one element in it")
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Not;

impl Not {
    pub fn string(&self) -> String {
        "NOT".into()
    }
}

impl TryFrom<&str> for Not {
    type Error = String;

    fn try_from(entity: &str) -> Result<Not, String> {
        if entity == "NOT" {
            Ok(Not)
        } else {
            Err(format!("`{}` should be `NOT`", entity))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SingleChar(pub char);

impl TryFrom<&str> for SingleChar {
    type Error = String;

    fn try_from(entity: &str) -> Result<SingleChar, Self::Error> {
        let char_res = char::from_str(entity);
        match char_res {
            Ok(c) => Ok(SingleChar(c)),
            Err(e) => Err(format!("{}", e))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlainString(pub String);

impl fmt::Display for PlainString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for PlainString {
    type Error = String;

    fn try_from(entity: &str) -> Result<PlainString, Self::Error> {
        if entity.is_empty() {
            Err(String::from("Field cannot be empty"))
        } else {
            Ok(PlainString(entity.to_string()))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Date {
    pub date: chrono::Date<Utc>
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.date.format("%Y%m%d"))
    }
}

impl TryFrom<&str> for Date {
    type Error = String;

    fn try_from(entity: &str) -> Result<Date, Self::Error> {
        let date = NaiveDate::parse_from_str(entity, "%Y%m%d");
        match date {
            Ok(d) => {
                Ok(Date { 
                    date: Utc.from_utc_date(&d)
                })
            },
            Err(err) => Err(format!("{}", err))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label(pub String);

impl TryFrom<&str> for Label {
    type Error = String;

    fn try_from(entity: &str) -> Result<Label, Self::Error> {
        // just forward to NoSpaceString for now
        let nospace: Result<NoSpaceString, String> = NoSpaceString::try_from(entity);
        nospace.map(|s| Label(s.value))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ClassExpression<R, F> {
    pub relation: R,
    pub filler: F
}

impl<R, F> ClassExpression<R, F> {
    pub fn new(relation: R, filler: F) -> ClassExpression<R, F> {
        ClassExpression {
            relation, filler
        }
    }
}

impl<R: fmt::Display, F: fmt::Display> fmt::Display for ClassExpression<R, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.relation, self.filler)
    }
}

impl<'a, R, F> TryFrom<&'a str> for ClassExpression<R, F>
    where
        R: TryFrom<&'a str, Error=String>,
        F: TryFrom<&'a str, Error=String> {
    
    type Error = String;

    fn try_from(entity: &'a str) -> Result<ClassExpression<R, F>, String> {
        lazy_static! {
            static ref PATTERN: regex::Regex = regex::Regex::new(r"^(.+)\((.+)\)$").unwrap();
        }

        if PATTERN.is_match(&entity) {
            let captures: regex::Captures = PATTERN.captures(&entity).unwrap();
            match captures.get(1) {
                Some(match_r) => match R::try_from(match_r.as_str()) {
                    Ok(relation) => {
                        // Here we have a relation, so now let's try the filler
                        match captures.get(2) {
                            Some(match_f) => match F::try_from(match_f.as_str()) {
                                Ok(filler) => {
                                    // And now we found the filler, so we have a full match
                                    Ok(ClassExpression::new(relation, filler))
                                },
                                Err(err) => Err(err)
                            },
                            None => Err(format!("Could not parse filler in `{}`", entity))
                        }
                    },
                    Err(err) => Err(err)
                },
                None => Err(format!("Could not parse Relation in `{}`", entity))
            }

        } else {
            Err(format!("Error parsing {}. Must be `relation(filler)`", entity))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Conjunction<C> {
    elements: Vec<C>
}

impl<C> Conjunction<C> {
    pub fn new<VecC: Into<Vec<C>>>(elements: VecC) -> Conjunction<C> {
        Conjunction {
            elements: elements.into()
        }
    }

    pub fn elements(&self) -> &[C] {
        self.elements.as_slice()
    }

    pub fn map_new<T, M>(&self, m: M) -> Conjunction<T>
        where M: FnMut(&C) -> T {
    
        let new_items: Vec<T> = self.elements.iter().map(m).collect();
        Conjunction { elements: new_items }
    }

    pub fn map_new_results<T, M, E>(&self, m: M) -> Result<Conjunction<T>, E>
        where M: FnMut(&C) -> Result<T, E> {

        let new_items: Result<Vec<T>, E> = self.elements.iter().map(m).collect();
        new_items.map(|elements| Conjunction { elements })
    }
}

impl<C: fmt::Display> fmt::Display for Conjunction<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.elements.iter().map(|c| c.to_string()).collect::<Vec<String>>().join(",");
        write!(f, "{}", s)
    }
}

impl<'a, C: TryFrom<&'a str, Error=String> + std::fmt::Debug> TryFrom<&'a str> for Conjunction<C> {
    type Error = String;

    fn try_from(entity: &'a str) -> Result<Conjunction<C>, Self::Error>{
        let (parsed, errors): (Vec<_>, Vec<_>) = entity.split(',')
            .map(|el| C::try_from(el))
            .partition(Result::is_ok);
        
        let errors: Vec<String> = errors.into_iter()
            .map(Result::unwrap_err)
            .collect();
        
        if !errors.is_empty() {
            Err(format!("Errors parsing `{}`: {}", entity, errors.join("; ")))
        } else {
            Ok(Conjunction {
                elements: parsed.into_iter()
                    .map(Result::unwrap)
                    .collect()
            })
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Property(pub String, pub String);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_eco_display() {
        assert_eq!(format!("{:?}", EcoCode::EXP), "EXP");
    }

    #[test]
    fn test_curie_try_from_correct() {
        assert_eq!(Curie::try_from("MGI:1234"), Ok(Curie::new("MGI", "1234")));
    }

    #[test]
    fn test_curie_multi_id() {
        assert_eq!(Curie::try_from("MGI:MGI:1234"), Ok(Curie::new("MGI", "MGI:1234")));
    }

    #[test]
    fn test_list_correct() {
        assert_eq!(ListField::try_from("MGI:1234|FB:5678"), Ok(ListField::new(vec![Curie::new("MGI", "1234"), Curie::new("FB", "5678")])))
    }

    #[test]
    fn test_empty_list() {
        let empty: Result<ListField<Curie>, String> = ListField::try_from("");
        assert_eq!(empty, Ok(ListField::new(vec![])))
    }

    #[test]
    fn test_eitherorboth() {
        assert_eq!(EitherOrBoth::try_from("NOT|MGI:1234"), Ok(EitherOrBoth::Both(Not, Curie::new("MGI", "1234"))))
    }

    #[test]
    fn test_one_or_two() {
        assert_eq!(OneOrTwoItems::try_from("taxon:1234|taxon:5678"), Ok(OneOrTwoItems::Two(Curie::new("taxon", "1234"), Curie::new("taxon", "5678"))))
    }

    #[test]
    fn test_single_char() {
        assert_eq!(SingleChar::try_from("c"), Ok(SingleChar('c')));
    }

    #[test]
    fn test_class_expression() {
        assert_eq!(ClassExpression::try_from("RO:1234(GO:5678)"), Ok(ClassExpression::new(Curie::new("RO", "1234"), Curie::new("GO", "5678"))))
    }

    #[test]
    fn test_conjunction() {
        assert_eq!(Conjunction::try_from("RO:1234,GO:1234"), Ok(Conjunction::new(vec![Curie::new("RO", "1234"), Curie::new("GO", "1234")])))
    }

    #[test]
    fn test_annotation_extension() {
        let extension = "part_of(GO:12345),part_of(MGI:5678)|foo_bar(FB:1234)";
        let parsed: Result<ListField<Conjunction<ClassExpression<Label, Curie>>>, String> = ListField::try_from(extension);
        assert_eq!(parsed, Ok(ListField::new(vec![
            Conjunction::new(vec![
                ClassExpression::new(Label(String::from("part_of")), Curie::new("GO", "12345")),
                ClassExpression::new(Label(String::from("part_of")), Curie::new("MGI", "5678"))
            ]),
            Conjunction::new(vec![
                ClassExpression::new(Label(String::from("foo_bar")), Curie::new("FB", "1234"))
            ])
        ])));
    }
}