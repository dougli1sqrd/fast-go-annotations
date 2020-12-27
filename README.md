# Fast GO Annotations

This is a rust parser for GO Annotations. An experimental, for-fun, and proof-of-concept version of https://github.com/biolink/ontobio.

This can be run as a command line tool to parse GAF 2.1 files: http://geneontology.org/docs/go-annotation-file-gaf-format-2.1.

## Install

First ensure you have rust installed with something like [rustup](https://rustup.rs/). This should also install the build tool [cargo](https://doc.rust-lang.org/cargo/).

Then clone this repository.

In the `fast-go-annotations` top directory, run 
```
cargo build
```

You can find the binary at `target/debug/fast-go-annotations`.

## Running

Running help we see:
```
$ ./target/debug/fast-go-annotations --help
Fast GO Annotation Parser 0.1.0

USAGE:
    fast-go-annotations [OPTIONS] --input-file <annotation> --context <PATH> --ontology <PATH>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -f, --input-file <annotation>      
    -c, --context <PATH>               Path to JSON-LD URI Context Mapping
    -r, --ontology <PATH>              Path to OBO JSON Ontology file
    -o, --out <out>                    
        --report-json <report-json>    
        --report-md <report-md>        
```

You can use `cargo run -- [args]` or just invoke the binary directly like in the example.

- Obtain a `go.json` from http://current.geneontology.org/ontology/index.html
- Obtain a `obo_context.jsonld` from https://github.com/prefixcommons/biocontext (just download a raw file from github)
- Download source GAF file at http://current.geneontology.org/products/annotations/index.html

Then you can parse the GAF file with

```
$ ./target/debug/fast-go-annotations --ontology go-ontology.json --context obo_context.jsonld --input-file fb-src.gaf --report-md report.md --report-json report.json --out fb.gaf
```

This produces a report Markdown file as well as a JSON, (compare to something like http://current.geneontology.org/reports/fb.report.md), and a validated version of the input GAF, at the path specified for `--out`.

## Documentation

Generate documentation with:
```
cargo doc
```

And then in a browser navigate to `file:///path/to/code/fast-go-annotations/target/doc/fast_go_annotations/index.html` to see the generated rust documentation for the project.

## Tests

Run tests with
```
cargo test
```