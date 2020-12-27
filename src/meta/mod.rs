//!
//! meta contains structs and functions providing easy methods to convert between various data representations
//! that are seen in annotation data. 
//! 
//! Namely, the `Context` struct contains various metadata that is used to create `GoAssociation`s from different sources. 
//! It contains the Ontology, a mapping of Labels to Uris, mapping of ECO Curies to Evidence Codes, and a Curie prefix mapping
//! to go from Curies to Uris and back. `GoAssociation` in general cannot be made without a Context. `Context` has `default()`
//! implemented so basic values are filled in by default.
//! 

pub mod curie;
pub mod eco;

use crate::annotation::fields::*;

use curie::*;
use eco::EcoCodeMapping;
use crate::ontology::Ontology;

pub struct Context {
    pub uri_mapping: curie::CurieMapping,
    pub label_mapping: curie::LabelMapping,
    pub eco_mapping: EcoCodeMapping,
    pub ontology: Ontology
}

impl Context {
    pub fn add_ontology(mut self, ontology: Ontology) -> Context {
        self.ontology = ontology;
        self
    }
}

impl Default for Context {
    fn default() -> Context {
        Context {
            uri_mapping: curie::CurieMapping::default(),
            label_mapping: curie::LabelMapping::default(),
            eco_mapping: EcoCodeMapping::default(),
            ontology: Ontology::default()
        }
    }
}

impl Context {
    pub fn label_to_curie(&self, label: &Label) -> Option<Curie> {
        // Label -> Uri -> Curie
        self.label_mapping.label_uri(label)
            .and_then(|uri| self.uri_mapping.curie_for_uri(uri))
    }

    pub fn curie_to_label(&self, curie: &Curie) -> Option<Label> {
        self.uri_mapping.uri_for_curie(curie)
            .and_then(|uri| self.label_mapping.uri_label(&uri).cloned())
    }
}
