//!
//! `model.rs` contains the high-level data model for GO Annotations. The main struct is `GoAssociation`, 
//! representing any association data from a gene product to a a GO Term, with evidence information attached.
//! 
//! Any GAF or GPAD annotation should be convertable into a `GoAssociation`, and back again into an annotation
//! format.
//! 
//! Parsing an annotatino line into a `GoAssociation` can fail, so any attempt to convert will return a
//! Result<GoAssociation, String>, with the error message being a string, if there is an error during conversion.
//! 
//! Future versions could make the error type smarter.
//! 
//! Converting from a `GoAssociation` into some other annotation format should in general not fail, so there is
//! no Error type.
//! 
//! Conversion from annotations to `GoAssociation` generally goes like:
//! 1. CSV parses the line into some basic struct, either a `Vec<String>`, or in this case a Tuple Struct, for gaf
//!     we have named `RawGaf2_1Record`. This type will likely change if we want to make this more generic over all
//!     types of annotation formats.
//! 2. The `TryFrom` trait is implemented for `BaseGaf2_1Row`. `BaseGaf2_1Row` uses the lower level annotation 
//!     data model in the `crate::annotation::fields` module to express what GAF looks like. 
//!     1. Each field type has `TryFrom` implemented for `&str`
//!     2. Conversion makes a series of nested closures, each one representing the succssful conversion of the
//!         previous field: So a Curie is attempted to be converted, and this successful branch is mapped to 
//!         a closure that attempts to convert the next column, etc.
//!     3. In this way, we obtain all 17 successful fields of the `BaseGaf2_1Row` struct, and we can assemble it
//!     4. Any errors in converting a field cancel the operation, returning the error message as an Error in the
//!         conversion of the `BaseGaf2_1Row` as a whole.
//! 3. In the `gaf` module, we provide implementations for the `HasSubject`, `HasRelation`, `HasTerm`, `HasEvidence`,
//!     `HasMetadata`, and `HasExtensions` traits, each of which provide functions for yielding the high level
//!     `Subject`, `Relation`, `Term`, `Evidence`, `Extensions`, and `Metadata` wrapper structs or an Error.
//! 4. This module then has a universal implementation for `ConvertableAnnotation` saying that anything that can
//!     implement all of the above traits can then produce a `GoAssociation`.
//! 5. Since we have just produced a `BaseGaf2_1Row` which has implementations for all of the above traits, we can
//!     then convert to GoAssociations automatically at this point.
//! 
//! The high-level function `parse_annotation` will generically take anything that implements `ConvertableAnnotation`
//! and turn it into a Result<GoAssociation, String>. This is what we use to ultimately produce the validation and rule
//! reports.
//! 
//! The `Context` object contains various metadata that is used to create `GoAssociation`s from different sources. Namely
//! it contains the Ontology, a mapping of Labels to Uris, mapping of ECO Curies to Evidence Codes, and a Curie prefix mapping
//! to go from Curies to Uris and back. `GoAssociation` in general cannot be made without a Context. `Context` has `default()`
//! implemented so basic values are filled in by default.
//! 

use std::convert::TryFrom;
use std::convert::TryInto;

use super::*;
use crate::annotation::fields::*;
use crate::meta::Context;

pub struct AnnotationWithContext<'a, A>(pub A, pub &'a Context);

#[derive(Debug, PartialEq, Clone)]
pub struct Term {
    pub id: Curie,
    pub taxon: Option<Curie>
}

impl Term {
    pub fn new(id: Curie, taxon: Option<Curie>) -> Term {
        Term {
            id, taxon
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Subject {
    pub id: Curie,
    pub label: NoSpaceString,
    pub fullname: Option<PlainString>,
    pub synonyms: ListField<PlainString>,
    pub kind: PlainString,
    pub taxon: Option<Curie>
}

impl Subject {
    pub fn new(id: Curie, label: NoSpaceString, fullname: Option<PlainString>, synonyms: ListField<PlainString>, kind: PlainString, taxon: Option<Curie>) -> Subject {
        Subject { id, label, fullname, synonyms, kind, taxon }
    }
}

impl Default for Subject {
    fn default() -> Subject {
        Subject::new(Curie::new("NS", "12345"), NoSpaceString::new("test_label"), None, ListField::new(vec![]), PlainString("gene_product".into()), None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    pub id: Curie,
    pub has_supporting_reference: ListField<Curie>,
    pub with_support_from: ListField<Conjunction<Curie>>
}

impl Evidence {
    pub fn new(id: Curie, references: ListField<Curie>, with_from: ListField<Conjunction<Curie>>) -> Evidence {
        Evidence {
            id,
            has_supporting_reference: references,
            with_support_from: with_from
        }
    }
}

impl Default for Evidence {
    fn default() -> Evidence {
        Evidence::new(Curie::new("ECO", "0000000"), ListField::new(vec![]), ListField::new(vec![]))
    }
}

pub type Relation = Curie;

#[derive(Debug, PartialEq, Clone)]
pub struct Metadata {
    pub negated: bool,
    pub aspect: Option<Aspect>,
    pub interacting_taxon: Option<Curie>,
    pub provided_by: NoSpaceString,
    pub date: fields::Date,
    pub properties: ListField<Property>
}

impl Default for Metadata {
    fn default() -> Metadata {
        Metadata {
            negated: false,
            aspect: None,
            interacting_taxon: None,
            provided_by: NoSpaceString::new("Test"),
            date: fields::Date::try_from("20200101").unwrap(),
            properties: ListField::new(vec![])
        }
    }
}


pub struct Extensions {
    subject: Option<ClassExpression<Curie, Curie>>,
    object: ListField<Conjunction<ClassExpression<Curie, Curie>>>
}

impl Default for Extensions {
    fn default() -> Extensions {
        Extensions::new(None, ListField::new(vec![]))
    }
}

impl Extensions {
    pub fn new(subject: Option<ClassExpression<Curie, Curie>>, object: ListField<Conjunction<ClassExpression<Curie, Curie>>>) -> Extensions {
        Extensions { subject, object }
    }
}

pub trait HasEvidence<Error> {
    fn evidence(&self, context: &Context) -> Result<Evidence, Error>;
}

pub trait HasSubject<Error> {
    fn subject(&self, context: &Context) -> Result<Subject, Error>;
}

pub trait HasRelation<Error> {
    fn relation(&self, context: &Context) -> Result<Relation, Error>;
}

pub trait HasTerm<Error> {
    fn term(&self, context: &Context) -> Result<Term, Error>;
}

pub trait HasMetadata<Error> {
    fn metadata(&self, context: &Context) -> Result<Metadata, Error>;
}

pub trait HasExtensions<Error> {
    fn extensions(&self, context: &Context) -> Result<Extensions, Error>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GoAssociation {
    // source_line: RawGaf2_1Record,
    pub subject: Subject,
    pub relation: Relation,
    pub object: Term,
    pub negated: bool,
    pub aspect: Option<Aspect>,
    pub interacting_taxon: Option<Curie>,
    pub evidence: Evidence,
    pub subject_extension: Option<ClassExpression<Curie, Curie>>,
    pub object_extension: ListField<Conjunction<ClassExpression<Curie, Curie>>>,
    pub provided_by: NoSpaceString,
    pub date: fields::Date,
    pub properties: ListField<Property>
}


pub trait ConvertableAnnotation: 
    HasEvidence<String> + 
    HasSubject<String> +
    HasRelation<String> +
    HasTerm<String> +
    HasMetadata<String> +
    HasExtensions<String> {}

/// Provide Blanket implementation of ConvertableAnnotation for anything
/// that implements HasEvidence, HasSubject, HasRelation, HasTerm, HasMetadata,
/// and HasExtensions
impl<A> ConvertableAnnotation for A
    where A:
        HasEvidence<String> + 
        HasSubject<String> +
        HasRelation<String> +
        HasTerm<String> +
        HasMetadata<String> +
        HasExtensions<String> { }


pub fn convert_raw<R, B>(raw: R, context: &Context) -> Result<GoAssociation, String>
    where
        R: TryInto<B, Error=String>,
        B: ConvertableAnnotation {

    raw.try_into().and_then(|b: B| parse_annotation(b, context))
}


impl From<(Subject, Relation, Term, Evidence, Metadata, Extensions)> for GoAssociation {
    fn from((subject, relation, term, evidence, metadata, extensions): (Subject, Relation, Term, Evidence, Metadata, Extensions)) -> GoAssociation {
        GoAssociation {
            subject,
            relation,
            object: term,
            evidence,
            negated: metadata.negated,
            aspect: metadata.aspect,
            interacting_taxon: metadata.interacting_taxon,
            provided_by: metadata.provided_by,
            date: metadata.date,
            properties: metadata.properties,
            subject_extension: extensions.subject,
            object_extension: extensions.object
        }
    }
}


impl<'a, Annotation> TryFrom<&AnnotationWithContext<'_, Annotation>> for GoAssociation 
    where Annotation: ConvertableAnnotation {

    type Error = String;

    fn try_from(AnnotationWithContext(annotation, context): &AnnotationWithContext<Annotation>) -> Result<GoAssociation, Self::Error> {

        annotation.subject(context)
            .and_then(|subject|
                annotation.relation(context)
            .and_then(|relation|
                annotation.term(context)
            .and_then(|term|
                annotation.evidence(context)
            .and_then(|evidence|
                annotation.metadata(context)
            .and_then(|metadata|
                annotation.extensions(context)
            .map(|extensions|
                GoAssociation::from((subject, relation, term, evidence, metadata, extensions)
            )))))))
    }
}

pub fn parse_annotation<A: ConvertableAnnotation>(annotation: A, context: &Context) -> Result<GoAssociation, String> {
    let annotation_with_context = AnnotationWithContext(annotation, context);
    GoAssociation::try_from(&annotation_with_context)
}


#[cfg(test)]
mod test {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_generic_parse() {
        let base = BaseGaf2_1Row(
            NoSpaceString::new("MGI"),
            NoSpaceString::try_from("MGI:98961").unwrap(),
            NoSpaceString::new("Wnt7a"),
            None,
            Curie::try_from("GO:0099175").unwrap(),
            ListField::new(vec![Curie::new("MGI", "MGI:5014434"), Curie::new("PMID", "21670302")]),
            EcoCode::IMP,
            ListField::new(vec![]),
            Aspect::BioProcess,
            Some(PlainString("wingless-type MMTV integration site family, member 7A".into())),
            ListField::new(vec![PlainString("tw".into()), PlainString("Wnt-7a".into())]),
            PlainString("protein".into()),
            OneOrTwoItems::One(Curie::new("taxon", "10090")),
            fields::Date{date: chrono::Utc.ymd(2018, 7, 11)},
            NoSpaceString::new("SynGO"),
            ListField::new(vec![
                Conjunction::new(vec![
                    ClassExpression::new(Label("occurs_in".into()), Curie::new("GO", "0098978")),
                    ClassExpression::new(Label("occurs_in".into()), Curie::new("EMAPA", "35405"))
                ])
            ]),
            None
        );

        let context = Context::default();
        let parsed = parse_annotation(base, &context);

        let expected = GoAssociation {
            subject: Subject::new(
                        Curie::new("MGI", "MGI:98961"),
                        NoSpaceString::new("Wnt7a"),
                        Some(PlainString("wingless-type MMTV integration site family, member 7A".into())),
                        ListField::new(vec![PlainString("tw".into()), PlainString("Wnt-7a".into())]), 
                        PlainString("protein".into()),
                        Some(Curie::new("taxon", "10090"))),
            relation: Curie::new("RO", "0002331"),
            object: Term::new(
                        Curie::new("GO", "0099175"),
                        Some(Curie::new("taxon", "10090"))),
            negated: false,
            aspect: Some(Aspect::BioProcess),
            interacting_taxon: None,
            evidence: Evidence::new(
                Curie::new("ECO", "0000315"), 
                ListField::new(vec![Curie::new("MGI", "MGI:5014434"), Curie::new("PMID", "21670302")]),
                ListField::new(vec![])),
            subject_extension: None,
            object_extension: ListField::new(vec![
                Conjunction::new(vec![
                    ClassExpression::new(Curie::new("BFO", "0000066"), Curie::new("GO", "0098978")),
                    ClassExpression::new(Curie::new("BFO", "0000066"), Curie::new("EMAPA", "35405"))
                ])
            ]),
            provided_by: NoSpaceString::new("SynGO"),
            date: fields::Date{date: chrono::Utc.ymd(2018, 7, 11)},
            properties: ListField::new(vec![])
        };

        assert_eq!(parsed.unwrap(), expected);
    }
}