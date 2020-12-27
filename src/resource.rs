//!
//! This module contains various functions for dealing with external resources.
//! 
//! For loading any external file into the interanal models, this is where that processing happens.
//! 
//! `load_prefix_contexts` loads a json-ld file with the prefix mappings into a Vec<(String, String)>
//! which can then be used to provide the `crate::meta::curie::CurieMapping` for a `Context`.
//! 
//! `load_ontology` will load a obo-json file with a contained ontology into the `Ontology` object
//! used in the Context.
//! 
//! `read_annotation_file` creates the CSV parser for the given file which will then be used to 
//! make `GoAssociation`s.
//! 
//! `write_annotation_file` creates a CSV writer to write out a parsed `GoAssociation` as an
//! annotation.
//! 
//! `write_json_report` takes the `Report` object and writes it out as JSON with serde.
//! 

use serde_json::{Value};
use std::fs::File;
use std::io::{BufReader};
use std::path::Path;
use std::fmt;
use csv::{ReaderBuilder, WriterBuilder};

use crate::ontology::Ontology;
use crate::report::Report;

#[derive(Debug)]
pub enum ResourceError {
    IoError(std::io::Error),
    Json(serde_json::Error),
    Context(String),
    OboError(fastobo_graphs::error::Error),
    CsvError(csv::Error)
}

impl fmt::Display for ResourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResourceError::IoError(err) => write!(f, "{}", err),
            ResourceError::Json(err) => write!(f, "{}", err),
            ResourceError::Context(err) => write!(f, "{}", err),
            ResourceError::OboError(err) => write!(f, "{}", err),
            ResourceError::CsvError(err) => write!(f, "{}", err)
        }
    }
}

pub fn load_prefix_context<P: AsRef<Path>>(path: P) -> Result<Vec<(String, String)>, ResourceError> {
    let value: Result<Value, ResourceError> = File::open(path)
        .map(BufReader::new).map_err(ResourceError::IoError)
        .and_then(|buf| serde_json::from_reader(buf).map_err(ResourceError::Json));

    let context = match value {
        Ok(v) => match v.get("@context") {
            Some(context) => match context.as_object() {
                Some(obj) => obj.iter().map(|(k, val)| {
                    match val.as_str() {
                        Some(vstr) => Ok((String::from(vstr), String::from(k.as_str()))),
                        None => Err(ResourceError::Context(format!("Value of key `{}` is not a string", k.clone())))
                    }
                }).collect::<Result<Vec<(String, String)>, ResourceError>>(),
                None => Err(ResourceError::Context("Value of `@context` is not a json object (mapping)".into()))
            },
            None => Err(ResourceError::Context("Could not find `@context` key in json-ld".into()))
        },
        Err(err) => Err(err)
    };

    context
}

pub fn load_ontology<P: AsRef<Path>>(path: P) -> Result<Ontology, ResourceError> {
    fastobo_graphs::from_file(path).map_err(ResourceError::OboError)
        .map(|obodoc| Ontology::from_obo_graph(&obodoc.graphs[0]))
}

pub fn read_annotation_file<P: AsRef<Path>>(path: P) -> Result<(String, csv::Reader<File>), ResourceError> {
    let p: &Path = path.as_ref();
    let name = p.canonicalize().unwrap().file_name().unwrap().to_str().unwrap().to_owned();

    ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .has_headers(false)
        .comment(Some(b'!'))
        .from_path(path)
        .map_err(ResourceError::CsvError)
        .map(|r| (name, r))
}

pub fn write_annotation_file<P: AsRef<Path>>(path: P) -> Result<csv::Writer<File>, ResourceError> {
    let p: &Path = path.as_ref();

    WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(csv::QuoteStyle::Never)
        .has_headers(false)
        .from_path(p)
        .map_err(ResourceError::CsvError)
}

pub fn write_json_report<P: AsRef<Path>>(report: &Report, path: P) -> Result<(), ResourceError> {
    File::create(path).map_err(ResourceError::IoError)
        .and_then(|f: File| match serde_json::to_writer_pretty(f, report) {
            Err(err) => Err(ResourceError::Json(err)),
            _ => Ok(())
        } )
}

#[cfg(test)]
mod test_csv {
    use super::*;

    #[test]
    fn test_csv_reader_without_comments() {
        let example = "MGI\tMGI:98961\tWnt7a\t\tGO:0099175\tMGI:MGI:5014434|PMID:21670302\tIMP\t\tP\twingless-type MMTV integration site family, member 7A\ttw|Wnt-7a\tprotein\ttaxon:10090\t20180711\tSynGO\toccurs_in(GO:0098978),occurs_in(EMAPA:35405)\t\n
                ! Hello world\n
                MGI\tMGI:98961\tWnt7a\t\tGO:0099175\tMGI:MGI:5014434|PMID:21670302\tIMP\t\tP\twingless-type MMTV integration site family, member 7A\ttw|Wnt-7a\tprotein\ttaxon:10090\t20180711\tSynGO\toccurs_in(GO:0098978),occurs_in(EMAPA:35405)\t";

        let mut reader = ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .has_headers(false)
            .trim(csv::Trim::All)
            .from_reader(example.as_bytes());
        
        assert_eq!(reader.records().next().unwrap().unwrap().get(1), Some("MGI:98961"));
        assert_eq!(reader.records().next().unwrap().unwrap().get(0), Some("! Hello world"));
        assert_eq!(reader.records().next().unwrap().unwrap().get(1), Some("MGI:98961"))
    }
}
