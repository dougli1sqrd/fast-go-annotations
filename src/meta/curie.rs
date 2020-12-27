use crate::annotation::fields::Curie;
use crate::annotation::fields::Label;
use bimap::BiHashMap;

pub type Uri = String;
pub type UriRef<'a> = &'a str;
pub type Prefix = String;

pub struct CurieMapping {
    mapping: BiHashMap<Uri, Prefix>
}

impl CurieMapping {
    pub fn new() -> CurieMapping {
        CurieMapping {
            mapping: BiHashMap::default()
        }
    }

    pub fn add_mappings<I: Iterator<Item=(Uri, Prefix)>>(&mut self, pairs: I) {
        self.mapping.extend(pairs)
    }

    pub fn uri_for_curie(&self, curie: &Curie) -> Option<Uri> {
        self.mapping.get_by_right(&curie.namespace).map(|uri| format!("{}{}", uri, &curie.identifier))
    }

    /// TODO Gross impl, but will technically work
    pub fn curie_for_uri(&self, uri: UriRef) -> Option<Curie> {
        for uri_prefix in self.mapping.left_values() {
            if let [_, right] = uri.split(uri_prefix).collect::<Vec<&str>>().as_slice() {
                // We know this is in here because of the match
                let prefix = self.mapping.get_by_left(uri_prefix).unwrap();
                return Some(Curie::new(String::from(prefix), String::from(*right)));
            }
        }
        None
    }
}

impl Default for CurieMapping {
    fn default() -> CurieMapping {
        let mut mapping = CurieMapping::new();
        mapping.mapping.extend(default_curie_mapping::default_curie_mapping());
        mapping
    }
}

pub trait LabelToUri {
    fn label_uri(&self, label: &Label) -> Option<&Uri>;

    fn uri_label(&self, uri: &Uri) -> Option<&Label>;
}

pub struct LabelMapping {
    mapping: BiHashMap<Label, Uri>
}

impl LabelMapping {
    pub fn new() -> LabelMapping {
        LabelMapping {
            mapping: BiHashMap::default()
        }
    }
}

impl LabelToUri for LabelMapping {
    fn label_uri(&self, label: &Label) -> Option<&Uri> {
        self.mapping.get_by_left(label)
    }

    fn uri_label(&self, uri: &Uri) -> Option<&Label> {
        self.mapping.get_by_right(uri)
    }
}

impl Default for LabelMapping {
    fn default() -> LabelMapping {
        let mut mapping = LabelMapping::new();
        mapping.mapping.extend(default_label_mapping::default_label_mapping());
        mapping
    }
}



mod default_label_mapping {
    use super::*;
    pub fn default_label_mapping() -> Vec<(Label, Uri)> {
        let label_to_curie = vec![
            (Label("occurs_in".into()), "http://purl.obolibrary.org/obo/BFO_0000066".into()),
            (Label("happens_during".into()), "http://purl.obolibrary.org/obo/RO_0002092".into()),
            (Label("has_input".into()), "http://purl.obolibrary.org/obo/RO_0002233".into()),
            (Label("results_in_specification_of".into()), "http://purl.obolibrary.org/obo/RO_0002356".into()),
            (Label("part_of".into()), "http://purl.obolibrary.org/obo/BFO_0000050".into()),
            (Label("has_part".into()), "http://purl.obolibrary.org/obo/BFO_0000051".into()),
            (Label("results_in_development_of".into()), "http://purl.obolibrary.org/obo/RO_0002296".into()),
            (Label("results_in_movement_of".into()), "http://purl.obolibrary.org/obo/RO_0002565".into()),
            (Label("occurs_at".into()), "http://purl.obolibrary.org/obo/GOREL_0000501".into()),
            (Label("stabilizes".into()), "http://purl.obolibrary.org/obo/GOREL_0000018".into()),
            (Label("positively_regulates".into()), "http://purl.obolibrary.org/obo/RO_0002213".into()),
            (Label("regulates_transport_of".into()), "http://purl.obolibrary.org/obo/RO_0002011".into()),
            (Label("regulates_transcription_of".into()), "http://purl.obolibrary.org/obo/GOREL_0098788".into()),
            (Label("causally_upstream_of".into()), "http://purl.obolibrary.org/obo/RO_0002411".into()),
            (Label("regulates_activity_of".into()), "http://purl.obolibrary.org/obo/GOREL_0098702".into()),
            (Label("adjacent_to".into()), "http://purl.obolibrary.org/obo/RO_0002220".into()),
            (Label("results_in_acquisition_of_features_of".into()), "http://purl.obolibrary.org/obo/RO_0002315".into()),
            (Label("results_in_morphogenesis_of".into()), "http://purl.obolibrary.org/obo/RO_0002298".into()),
            (Label("results_in_maturation_of".into()), "http://purl.obolibrary.org/obo/RO_0002299".into()),
            (Label("has_participant".into()), "http://purl.obolibrary.org/obo/RO_0000057".into()),
            (Label("transports_or_maintains_localization_of".into()), "http://purl.obolibrary.org/obo/RO_0002313".into()),
            (Label("negatively_regulates".into()), "http://purl.obolibrary.org/obo/RO_0002212".into()),
            (Label("regulates".into()), "http://purl.obolibrary.org/obo/RO_0002211".into()),
            (Label("regulates_expression_of".into()), "http://purl.obolibrary.org/obo/GOREL_0098789".into()),
            (Label("has_target_end_location".into()), "http://purl.obolibrary.org/obo/RO_0002339".into()),
            (Label("produced_by".into()), "http://purl.obolibrary.org/obo/RO_0003001".into()),
            (Label("has_end_location".into()), "http://purl.obolibrary.org/obo/RO_0002232".into()),
            (Label("directly_positively_regulates".into()), "http://purl.obolibrary.org/obo/RO_0002629".into()),
            (Label("has_direct_input".into()), "http://purl.obolibrary.org/obo/GOREL_0000752".into()),
            (Label("enables".into()), "http://purl.obolibrary.org/obo/RO_0002327".into()),
            (Label("enabled_by".into()), "http://purl.obolibrary.org/obo/RO_0002333".into()),
            (Label("involved_in".into()), "http://purl.obolibrary.org/obo/RO_0002331".into()),
            (Label("acts_upstream_of".into()), "http://purl.obolibrary.org/obo/RO_0002263".into()),
            (Label("colocalizes_with".into()), "http://purl.obolibrary.org/obo/RO_0002325".into()),
            (Label("contributes_to".into()), "http://purl.obolibrary.org/obo/RO_0002326".into()),
            (Label("acts_upstream_of_or_within".into()), "http://purl.obolibrary.org/obo/RO_0002264".into()),
            (Label("acts_upstream_of_or_within_positive_effect".into()), "http://purl.obolibrary.org/obo/RO_0004032".into()),
            (Label("acts_upstream_of_or_within_negative_effect".into()), "http://purl.obolibrary.org/obo/RO_0004033".into()),
            (Label("acts_upstream_of_negative_effect".into()), "http://purl.obolibrary.org/obo/RO_0004035".into()),
            (Label("acts_upstream_of_positive_effect".into()), "http://purl.obolibrary.org/obo/RO_0004034".into()),
            (Label("located_in".into()), "http://purl.obolibrary.org/obo/RO_0001025".into()),
            (Label("is_active_in".into()), "http://purl.obolibrary.org/obo/RO_0002432".into()),
            (Label("exists_during".into()), "http://purl.obolibrary.org/obo/GOREL_0000032".into()),
            (Label("coincident_with".into()), "http://purl.obolibrary.org/obo/RO_0002008".into()),
            (Label("has_regulation_target".into()), "http://purl.obolibrary.org/obo/GOREL_0000015".into()),
            (Label("not_happens_during".into()), "http://purl.obolibrary.org/obo/GOREL_0000025".into()),
            (Label("not_exists_during".into()), "http://purl.obolibrary.org/obo/GOREL_0000026".into()),
            (Label("directly_negatively_regulates".into()), "http://purl.obolibrary.org/obo/RO_0002449".into()),
            (Label("inhibited_by".into()), "http://purl.obolibrary.org/obo/GOREL_0000508".into()),
            (Label("activated_by".into()), "http://purl.obolibrary.org/obo/GOREL_0000507".into()),
            (Label("regulates_o_acts_on_population_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001008".into()),
            (Label("regulates_o_occurs_in".into()), "http://purl.obolibrary.org/obo/GOREL_0001004".into()),
            (Label("regulates_o_results_in_movement_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001005".into()),
            (Label("acts_on_population_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001006".into()),
            (Label("regulates_o_has_input".into()), "http://purl.obolibrary.org/obo/GOREL_0001030".into()),
            (Label("regulates_o_has_participant".into()), "http://purl.obolibrary.org/obo/GOREL_0001016".into()),
            (Label("has_output_o_axis_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001002".into()),
            (Label("regulates_o_results_in_formation_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001025".into()),
            (Label("regulates_o_results_in_acquisition_of_features_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001010".into()),
            (Label("regulates_o_has_agent".into()), "http://purl.obolibrary.org/obo/GOREL_0001011".into()),
            (Label("results_in_formation_of".into()), "http://purl.obolibrary.org/obo/RO_0002297".into()),
            (Label("has_output_o_axis_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001002".into()),
            (Label("has_start_location".into()), "http://purl.obolibrary.org/obo/RO_0002231".into()),
            (Label("has_output".into()), "http://purl.obolibrary.org/obo/GOREL_0000006".into()),
            (Label("results_in_commitment_to".into()), "http://purl.obolibrary.org/obo/RO_0002348".into()),
            (Label("regulates_o_results_in_commitment_to".into()), "http://purl.obolibrary.org/obo/GOREL_0001022".into()),
            (Label("regulates_o_has_output".into()), "http://purl.obolibrary.org/obo/GOREL_0001003".into()),
            (Label("has_target_end_location".into()), "http://purl.obolibrary.org/obo/RO_0002339".into()),
            (Label("regulates_o_results_in_development_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001023".into()),
            (Label("results_in_determination_of".into()), "http://purl.obolibrary.org/obo/RO_0002349".into()),
            (Label("has_target_end_location".into()), "http://purl.obolibrary.org/obo/RO_0002339".into()),
            (Label("regulates_o_results_in_maturation_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001012".into()),
            (Label("regulates_o_results_in_morphogenesis_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001026".into()),
            (Label("has_agent".into()), "http://purl.obolibrary.org/obo/RO_0002218".into()),
            (Label("causally_upstream_of_or_within".into()), "http://purl.obolibrary.org/obo/RO_0002418".into()),
            (Label("overlaps".into()), "http://purl.obolibrary.org/obo/RO_0002131".into()),
            (Label("has_target_start_location".into()), "http://purl.obolibrary.org/obo/RO_0002338".into()),
            (Label("capable_of_part_of".into()), "http://purl.obolibrary.org/obo/RO_0002216".into()),
            (Label("regulates_o_results_in_specification_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001027".into()),
            (Label("results_in_division_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001019".into()),
            (Label("regulates_translation_of".into()), "http://purl.obolibrary.org/obo/GOREL_0098790".into()),
            (Label("imports".into()), "http://purl.obolibrary.org/obo/RO_0002340".into()),
            (Label("directly_regulates".into()), "http://purl.obolibrary.org/obo/RO_0002578".into()),
            (Label("regulates_o_results_in_division_of".into()), "http://purl.obolibrary.org/obo/GOREL_0001024".into())
        ];
        label_to_curie
    }
}

mod default_curie_mapping {
    use super::*;

    pub fn default_curie_mapping() -> Vec<(Uri, Prefix)> {
        let uri_prefixes = vec![
            ("http://purl.obolibrary.org/obo/RO_".into(), "RO".into()),
            ("http://purl.obolibrary.org/obo/GO_".into(), "GO".into()),
            ("http://purl.obolibrary.org/obo/ECO_".into(), "ECO".into()),
            ("http://purl.obolibrary.org/obo/BFO_".into(), "BFO".into()),
            ("http://purl.obolibrary.org/obo/GOREL_".into(), "GO_REL".into()),
        ];
        uri_prefixes
    }
}
