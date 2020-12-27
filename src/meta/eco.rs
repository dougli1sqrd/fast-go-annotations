use std::collections::HashMap;
use std::iter::Extend;

use crate::annotation::fields::Curie;
use crate::annotation::fields::EcoCode;


pub struct EcoCodeMapping {
    eco_to_curie: HashMap<(EcoCode, Option<Curie>), Curie>,
    curie_to_eco: HashMap<Curie, EcoCode>
}

impl EcoCodeMapping {
    pub fn new() -> EcoCodeMapping {
        EcoCodeMapping {
            eco_to_curie: HashMap::new(),
            curie_to_eco: HashMap::new()
        }
    }

    pub fn eco_to_curie(&self, eco: EcoCode, goref: Option<&Curie>) -> Option<&Curie> {
        self.eco_to_curie.get(&(eco, goref.cloned()))
            .or_else(|| self.eco_to_curie.get(&(eco, None)) )
    }

    pub fn curie_to_eco(&self, curie: &Curie) -> Option<EcoCode> {
        match self.curie_to_eco.get(curie) {
            Some(eco) => Some(*eco),
            None => None
        }
    }
}

impl Default for EcoCodeMapping {
    fn default() -> EcoCodeMapping {
        let mut mapping = EcoCodeMapping::new();
        mapping.eco_to_curie.extend(default_eco_mappings::default_eco_mappings());
        mapping.curie_to_eco.extend(default_eco_mappings::default_eco_mappings()
            .into_iter()
            .map(|((code, _), curie)| (curie, code)));
        
        mapping
    }
}

mod default_eco_mappings {
    use super::*;

    pub fn default_eco_mappings() -> Vec<((EcoCode, Option<Curie>), Curie)> {
        let default_mappings = vec![
            ((EcoCode::EXP, None),                                  Curie::new("ECO", "0000269")),
            ((EcoCode::HDA, None),                                  Curie::new("ECO", "0007005")),
            ((EcoCode::HEP, None),                                  Curie::new("ECO", "0007007")),
            ((EcoCode::HGI, None),                                  Curie::new("ECO", "0007003")),
            ((EcoCode::HMP, None),                                  Curie::new("ECO", "0007001")),
            ((EcoCode::HTP, None),                                  Curie::new("ECO", "0006056")),
            ((EcoCode::IBA, None),                                  Curie::new("ECO", "0000318")),
            ((EcoCode::IBD, None),                                  Curie::new("ECO", "0000319")),
            ((EcoCode::IC ,None),                                   Curie::new("ECO", "0000305")),
            ((EcoCode::IDA, None),                                  Curie::new("ECO", "0000314")),
            ((EcoCode::IEA, None),                                  Curie::new("ECO", "0000501")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000002"))), Curie::new("ECO", "0000256")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000003"))), Curie::new("ECO", "0000501")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000004"))), Curie::new("ECO", "0000501")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000019"))), Curie::new("ECO", "0000265")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000020"))), Curie::new("ECO", "0000265")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000023"))), Curie::new("ECO", "0000501")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000035"))), Curie::new("ECO", "0000265")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000037"))), Curie::new("ECO", "0000322")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000038"))), Curie::new("ECO", "0000323")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000039"))), Curie::new("ECO", "0000322")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000041"))), Curie::new("ECO", "0000322")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000040"))), Curie::new("ECO", "0000323")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000049"))), Curie::new("ECO", "0000265")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000107"))), Curie::new("ECO", "0000256")),
            ((EcoCode::IEA, Some(Curie::new("GO_REF", "0000108"))), Curie::new("ECO", "0000363")),
            ((EcoCode::IEP, None),                                  Curie::new("ECO", "0000270")),
            ((EcoCode::IGC, None),                                  Curie::new("ECO", "0000317")),
            ((EcoCode::IGC, Some(Curie::new("GO_REF", "0000025"))), Curie::new("ECO", "0000354")),
            ((EcoCode::IKR, None),                                  Curie::new("ECO", "0000320")),
            ((EcoCode::IMP, None),                                  Curie::new("ECO", "0000315")),
            ((EcoCode::IMR, None),                                  Curie::new("ECO", "0000320")),
            ((EcoCode::IPI, None),                                  Curie::new("ECO", "0000353")),
            ((EcoCode::IGI, None),                                  Curie::new("ECO", "0000316")),
            ((EcoCode::IRD, None),                                  Curie::new("ECO", "0000321")),
            ((EcoCode::ISA, None),                                  Curie::new("ECO", "0000247")),
            ((EcoCode::ISM, None),                                  Curie::new("ECO", "0000255")),
            ((EcoCode::ISO, None),                                  Curie::new("ECO", "0000266")),
            ((EcoCode::ISS, None),                                  Curie::new("ECO", "0000250")),
            ((EcoCode::ISS, Some(Curie::new("GO_REF", "0000012"))), Curie::new("ECO", "0000031")),
            ((EcoCode::ISS, Some(Curie::new("GO_REF", "0000027"))), Curie::new("ECO", "0000031")),
            ((EcoCode::ISS, Some(Curie::new("GO_REF", "0000011"))), Curie::new("ECO", "0000255")),
            ((EcoCode::NAS, None),                                  Curie::new("ECO", "0000303")),
            ((EcoCode::ND, None),                                   Curie::new("ECO", "0000307")),
            ((EcoCode::RCA, None),                                  Curie::new("ECO", "0000245")),
            ((EcoCode::TAS, None),                                  Curie::new("ECO", "0000304")),
        ];
        default_mappings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_default() {
        let ecomap = EcoCodeMapping::default();
        assert_eq!(ecomap.eco_to_curie(EcoCode::ISO, None), Some(&Curie::new("ECO", "0000266")))
    }

    #[test]
    fn test_ikr_in_default_map() {
        let ecomap = EcoCodeMapping::default();
        assert_eq!(ecomap.eco_to_curie(EcoCode::IKR, None), Some(&Curie::new("ECO", "0000320")))
    }

    #[test]
    fn test_default_has_all_eco_codes() {
        let ecomap = EcoCodeMapping::default();
        for code in EcoCode::iter() {
            println!("code = {:?}", code);
            assert_ne!(ecomap.eco_to_curie(code, None), None);
        }
    }
}

