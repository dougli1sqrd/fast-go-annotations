
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

pub mod fields;
pub mod model;
pub mod gaf;

use crate::meta::Context;
use crate::ontology::NodeAspect;

use fields::*;

#[derive(Copy, Clone, Debug)]
pub enum GafVersion {
    Gaf2_1,
    Gaf2_2
}

#[derive(Copy, Clone, Debug)]
pub enum GpadVersion {
    Gpad1_2,
    Gpad2_0
}

#[derive(Copy, Clone, Debug)]
pub enum GpiVersion {
    Gpi1_2
}

#[derive(Copy, Clone, Debug)]
pub enum DocumentType {
    Gaf(GafVersion),
    Gpad(GpadVersion),
    Gpi(GpiVersion)
}

#[derive(Debug, Clone)]
pub struct AnnotationDocument<A> {
    document_type: DocumentType,
    comments: Vec<String>,
    annotations: Vec<A>
}

///                         0       1       2      3                4       5       6       7       8           9           10     11      12      13       14      15       16
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct RawGaf2_1Record(String, String, String, Option<String>, String, String, String, String, char, Option<String>, String, String, String, String, String, String, Option<String>);

impl fmt::Display for RawGaf2_1Record {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn str_opt(opt: &Option<String>) -> &str {
            match opt {
                Some(s) => s,
                None => ""
            }
        }
        let col8 = self.8.clone().to_string();
        let col8str = col8.as_str();
        let row: Vec<&str> = vec![&self.0, &self.1, &self.2, str_opt(&self.3), &self.4, &self.5, &self.6, &self.7, col8str, &str_opt(&self.9), &self.10, &self.11, &self.12, &self.13, &self.14, &self.15, &str_opt(&self.16)];
        write!(f, "{}", row.join("\t"))
    }
}

///
/// The Basic structure of a GAF row.
/// Look in the fields.rs module for information about these types.
/// 
/// This is what is converted into a full `GoAssociation` object.
/// 
/// Lines from the CSV parser get parsed into this structure.
#[derive(Clone, Debug, PartialEq)]
pub struct BaseGaf2_1Row(
    NoSpaceString,                              /// 0
    NoSpaceString,                              /// 1
    NoSpaceString,                              /// 2
    Option<EitherOrBoth<Not, Label>>,           /// 3
    Curie,                                      /// 4
    ListField<Curie>,                           /// 5
    EcoCode,                                    /// 6
    ListField<Curie>,                           /// 7
    Aspect,                                     /// 8
    Option<PlainString>,                        /// 9
    ListField<PlainString>,                     /// 10
    PlainString,                                /// 11
    OneOrTwoItems<Curie>,                       /// 12
    fields::Date,                               /// 13
    NoSpaceString,                              /// 14
    ListField<Conjunction<ClassExpression<Label, Curie>>>, /// 15 part_of(GO:12345)|hello(RO:5432),occurs_in(MGI:123456)
    Option<Curie>                               // 16
);

impl TryFrom<RawGaf2_1Record> for BaseGaf2_1Row {
    type Error = String;

    fn try_from(gaf21_record: RawGaf2_1Record) -> Result<BaseGaf2_1Row, String> {
        let base_row = 
                NoSpaceString::try_from(gaf21_record.0.as_str())
            .and_then(|f0|
                NoSpaceString::try_from(gaf21_record.1.as_str())
            .and_then(|f1|
                NoSpaceString::try_from(gaf21_record.2.as_str())
            .and_then(|f2| 
                match &gaf21_record.3 {
                    None => Ok(None),
                    Some(f) => EitherOrBoth::try_from(f.as_str()).map(Some)
                }
            .and_then(|f3: Option<EitherOrBoth<Not, Label>>|
                Curie::try_from(gaf21_record.4.as_str())
            .and_then(|f4|
                ListField::try_from(gaf21_record.5.as_str())
            .and_then(|f5: ListField<Curie>| 
                EcoCode::try_from(gaf21_record.6.as_str())
            .and_then(|f6| 
                ListField::try_from(gaf21_record.7.as_str())
            .and_then(|f7: ListField<Curie>|
                Aspect::try_from(gaf21_record.8)
            .and_then(|f8|
                match &gaf21_record.9 {
                    None => Ok(None),
                    Some(f) => PlainString::try_from(f.as_str()).map(Some)
                }
            .and_then(|f9|
                ListField::try_from(gaf21_record.10.as_str())
            .and_then(|f10: ListField<PlainString>|
                PlainString::try_from(gaf21_record.11.as_str())
            .and_then(|f11|
                OneOrTwoItems::try_from(gaf21_record.12.as_str())
            .and_then(|f12: OneOrTwoItems<Curie>|
                fields::Date::try_from(gaf21_record.13.as_str())
            .and_then(|f13|
                NoSpaceString::try_from(gaf21_record.14.as_str())
            .and_then(|f14|
                ListField::try_from(gaf21_record.15.as_str())
            .and_then(|f15: ListField<Conjunction<ClassExpression<Label, Curie>>>|
                match &gaf21_record.16 {
                    None => Ok(None),
                    Some(f) => Curie::try_from(f.as_str()).map(Some)
                }
            .map(|f16|
                BaseGaf2_1Row(f0, f1, f2, f3, f4, f5, f6, f7, f8, f9, f10, f11, f12, f13, f14, f15, f16)
            )))))))))))))))));
        
        base_row
    }
}

impl From<(model::GoAssociation, &Context)> for BaseGaf2_1Row {
    fn from((association, context): (model::GoAssociation, &Context)) -> BaseGaf2_1Row {
        let qualifier_field = if association.negated {
            EitherOrBoth::Both(Not, context.curie_to_label(&association.relation).unwrap())
        } else {
            EitherOrBoth::Right(context.curie_to_label(&association.relation).unwrap())
        };

        let aspect = match association.aspect {
            Some(a) => a,
            None => {
                let node = context.ontology.get_node(context.uri_mapping.uri_for_curie(&association.object.id).unwrap()).unwrap();
                node.aspect().unwrap()
            }
        };

        let taxon = if let Some(t) = association.object.taxon {
            if let Some(interacting) = association.interacting_taxon {
                OneOrTwoItems::Two(t, interacting)
            } else {
                OneOrTwoItems::One(t)
            }
        } else {
            OneOrTwoItems::One(Curie::new("taxon", "000000"))
        };

        let tovec = association.evidence.with_support_from.items().to_vec();
        let nested_vec: Vec<Vec<Curie>> = tovec.iter().map(|conj| conj.elements().to_vec() ).collect();
        let flattened: Vec<Curie> = nested_vec.into_iter().flatten().collect();
        let withfrom = ListField::new(flattened);
        
        let label_extensions = association.object_extension.map_new(|c| {
            c.map_new(|cls| {
                let ClassExpression {relation, filler} = cls;
                let label = context.curie_to_label(relation).unwrap();
                ClassExpression::new(label, filler.clone())
            })
        });

        let subject_ext = association.subject_extension.map(|subj| {
            let ClassExpression {relation: _, filler} = subj;
            filler
        });

        BaseGaf2_1Row(
            NoSpaceString::new(association.subject.id.identifier),
            NoSpaceString::new(association.subject.id.namespace),
            association.subject.label,
            Some(qualifier_field),
            association.object.id,
            association.evidence.has_supporting_reference,
            context.eco_mapping.curie_to_eco(&association.evidence.id).unwrap(),
            withfrom,
            aspect,
            association.subject.fullname,
            association.subject.synonyms,
            association.subject.kind,
            taxon,
            association.date,
            association.provided_by,
            label_extensions,
            subject_ext
        )
    }
}

impl From<BaseGaf2_1Row> for RawGaf2_1Record {
    fn from(base_row: BaseGaf2_1Row) -> RawGaf2_1Record {
        RawGaf2_1Record(
            base_row.0.value,
            base_row.1.value,
            base_row.2.value,
            base_row.3.map(|q| match q {
                EitherOrBoth::Left(l) => l.string(),
                EitherOrBoth::Right(r) => r.0,
                EitherOrBoth::Both(l, r) => format!("{}|{}", l.string(), r.0)
            }),
            base_row.4.string(),
            base_row.5.to_string(),
            base_row.6.to_string(),
            base_row.7.to_string(),
            base_row.8.as_char(),
            base_row.9.map(|p| p.0),
            base_row.10.to_string(),
            base_row.11.0,
            base_row.12.to_string(),
            base_row.13.to_string(),
            base_row.14.value,
            base_row.15.to_string(),
            base_row.16.map(|c| c.to_string())
        )
    }
}



#[cfg(test)]
mod test {
    use super::*;

    use csv::{ReaderBuilder};
    use chrono::TimeZone;

    #[test]
    fn test_reader_into_raw_2_1_gaf() {

        let example = "MGI\tMGI:98961\tWnt7a\t\tGO:0099175\tMGI:MGI:5014434|PMID:21670302\tIMP\t\tP\twingless-type MMTV integration site family, member 7A\ttw|Wnt-7a\tprotein\ttaxon:10090\t20180711\tSynGO\toccurs_in(GO:0098978),occurs_in(EMAPA:35405)\t";
        let mut gaf_reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .has_headers(false)
            .comment(Some(b'!'))
            .from_reader(example.as_bytes());

        let raw: Result<RawGaf2_1Record, csv::Error> = gaf_reader.deserialize().next().unwrap();
        println!("{:?}", raw);
        let expected = RawGaf2_1Record(
            "MGI".into(),
            "MGI:98961".into(),
            "Wnt7a".into(),
            None,
            "GO:0099175".into(),
            "MGI:MGI:5014434|PMID:21670302".into(),
            "IMP".into(),
            "".into(),
            'P',
            Some("wingless-type MMTV integration site family, member 7A".into()),
            "tw|Wnt-7a".into(),
            "protein".into(),
            "taxon:10090".into(),
            "20180711".into(),
            "SynGO".into(),
            "occurs_in(GO:0098978),occurs_in(EMAPA:35405)".into(),
            None
        );
        assert_eq!(raw.unwrap(), expected);
        // let base: Result<BaseGaf2_1Row, String> = match raw {
        //     Ok(record) => BaseGaf2_1Row::try_from(record),
        //     Err(err) => Err(format!("CSV parse error: {}", err))
        // };
    }

    #[test]
    fn test_convert_gaf_to_base_row() {
        let example = "MGI\tMGI:98961\tWnt7a\t\tGO:0099175\tMGI:MGI:5014434|PMID:21670302\tIMP\t\tP\twingless-type MMTV integration site family, member 7A\ttw|Wnt-7a\tprotein\ttaxon:10090\t20180711\tSynGO\toccurs_in(GO:0098978),occurs_in(EMAPA:35405)\t";
        let mut gaf_reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .has_headers(false)
            .comment(Some(b'!'))
            .from_reader(example.as_bytes());

        let raw: RawGaf2_1Record = gaf_reader.deserialize().next().unwrap().unwrap();
        // println!("raw: {:?}", raw);
        let base: Result<BaseGaf2_1Row, String> = BaseGaf2_1Row::try_from(raw);
        // println!("{:?}", base);
        
        let expected = BaseGaf2_1Row(
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

        assert_eq!(base.unwrap(), expected)
    }
}