extern crate fastobo_graphs;
extern crate daggy;
#[macro_use] 
extern crate lazy_static;
extern crate json_ld;

use std::process;
use std::io::Write;
use std::fs::File;

use clap::{Arg, App};

pub mod ontology;
pub mod annotation;
pub mod meta;
pub mod rules;
pub mod resource;
pub mod report;
pub mod validate;

fn main() {
    // println!("Hello, world!");

    let matches = App::new("Fast GO Annotation Parser")
        .version("0.1.0")
        .arg(Arg::with_name("ontology")
            .short("r")
            .long("ontology")
            .value_name("PATH")
            .help("Path to OBO JSON Ontology file")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("context")
            .short("c")
            .long("context")
            .value_name("PATH")
            .help("Path to JSON-LD URI Context Mapping")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("annotation")
            .short("f")
            .long("input-file")
            .takes_value(true)
            .required(true))
        .arg(Arg::with_name("out")
            .short("o")
            .long("out")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("report-md")
            .long("report-md")
            .takes_value(true)
            .required(false))
        .arg(Arg::with_name("report-json")
            .long("report-json")
            .takes_value(true)
            .required(false))
        .get_matches();

    let ontology_path = matches.value_of("ontology").unwrap();
    let context = matches.value_of("context").unwrap();
    let annotation = matches.value_of("annotation").unwrap();
    let maybe_out = matches.value_of("out");

    let annotation_reader = resource::read_annotation_file(annotation).unwrap_or_else(|e| {
        println!("Error loading annotations: {}", e);
        process::exit(1);
    });

    let uri_map = resource::load_prefix_context(context).unwrap_or_else(|e| {
        println!("Error making URI prefix context: {}", e);
        process::exit(1);
    });

    let ontology_graph = resource::load_ontology(ontology_path).unwrap_or_else(|e| {
        println!("Error building ontology: {}", e);
        process::exit(1);
    });

    let out = match maybe_out {
        Some(out_path) => Some(resource::write_annotation_file(out_path).unwrap_or_else(|e| {
            println!("Could not make output at {}: {}", out_path, e);
            process::exit(1);
        })),
        None => None
    };

    let mut validation_context = meta::Context::default();
    validation_context.uri_mapping.add_mappings(uri_map.into_iter());
    validation_context = validation_context.add_ontology(ontology_graph);

    let report_result = validation_annotations_into_results(annotation_reader, out, validation_context);

    if let Some(md_path) = matches.value_of("report-md") {
        match &report_result {
            Ok(r) => {
                let mut f = File::create(md_path).unwrap_or_else(|e| {
                    println!("Problem Creating file at `{}`: {}", md_path, e);
                    process::exit(1);
                });
                let _ = write!(f, "{}", r);
            }
            Err(err) => {
                println!("Error reading CSV: {}", err);
                process::exit(1);
            }
        };
    }

    if let Some(json_path) = matches.value_of("report-json") {
        match report_result {
            Ok(r) => {
                resource::write_json_report(&r, json_path).unwrap_or_else(|e| {
                    println!("Error! {}", e);
                    process::exit(1);
                });
            }
            Err(err) => println!("Error reading CSV: {}", err)
        };
    }

    
}

fn validation_annotations_into_results(mut annotations_reader: (String, csv::Reader<File>), mut annotations_writer: Option<csv::Writer<File>>, context: meta::Context) -> Result<report::Report, csv::Error> {
    let deserialized = annotations_reader.1.deserialize();
    let name = annotations_reader.0;
    let mut report = report::Report::new(name);

    for next in deserialized {
        let raw: annotation::RawGaf2_1Record = match next {
            Ok(record) => record,
            Err(err) => { return Err(err) }
        };

        let (maybe_assoc, next_report) = validate::parse_and_report_gaf_2_1(raw, &context, report);
        report = next_report;

        if let (Some(assoc), Some(writer)) = (maybe_assoc, &mut annotations_writer) {
            let base: annotation::BaseGaf2_1Row = (assoc, &context).into();
            let raw: annotation::RawGaf2_1Record = base.into();
            if let Err(e) = writer.serialize(raw) {
                return Err(e)
            }
        }
    }

    Ok(report)

}

