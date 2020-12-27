use super::fields::*;
use super::{BaseGaf2_1Row};
use super::model::{HasSubject, HasRelation, HasTerm, HasEvidence, HasMetadata, HasExtensions, 
    Subject, Relation, Term, Evidence, Metadata, Extensions};
use crate::meta::Context;

impl HasSubject<String> for BaseGaf2_1Row {

    fn subject(&self, _: &Context) -> Result<Subject, String> {
        let id = Curie::new(&self.0.value, &self.1.value);
        let label = self.2.clone();
        let fullname = self.9.clone();
        let synonyms = self.10.clone();
        let kind = self.11.clone();
        let taxon = Some(match &self.12 {
            OneOrTwoItems::One(t) => t,
            OneOrTwoItems::Two(t, _) => t
        }.clone());

        Ok(Subject::new(id, label, fullname, synonyms, kind, taxon))
    }
}

impl HasRelation<String> for BaseGaf2_1Row {
    /// Relation is either from Qualifier, or from Aspect
    fn relation(&self, context: &Context) -> Result<Relation, String> {

        fn relation_from_aspect(aspect: Aspect) -> Relation {
            match aspect {
                Aspect::BioProcess => Curie::new("RO", "0002331"),
                Aspect::CellComponent => Curie::new("BFO", "0000050"),
                Aspect::MolecularFunction => Curie::new("RO", "0002327")
            }
        }

        let qualifier_label = match &self.3 {
            Some(qual) => match qual {
                EitherOrBoth::Right(label) => Some(label),
                EitherOrBoth::Both(_, label) => Some(label),
                _ => None
            },
            _ => None
        };

        if let Some(label) = qualifier_label {
            if let Some(rel) = context.label_to_curie(&label) {
                Ok(rel)
            } else {
                Ok(relation_from_aspect(self.8))
            }
        } else {
            Ok(relation_from_aspect(self.8))
        }
    }
}

impl HasTerm<String> for BaseGaf2_1Row {

    fn term(&self, _: &Context) -> Result<Term, String> {
        
        if self.4.same_namespace("GO") {
            let id = self.4.clone();
            let taxon = Some(match &self.12 {
                OneOrTwoItems::One(t) => t,
                OneOrTwoItems::Two(t, _) => t
            }.clone());
            Ok(Term::new(id, taxon))
        } else {
            Err("Curie must be a GO term".into())
        }   
    }
}

impl HasEvidence<String> for BaseGaf2_1Row {

    fn evidence(&self, context: &Context) -> Result<Evidence, String> {
        // Convert column index 6, evidence code into an evidence CURIE
        // Grab the first of any GO_REF Curies in references
        let goref = &self.5.items()
            .iter()
            .filter(|curie| curie.same_namespace("GO_REF"))
            .take(1)
            .next();
        
        let eco_curie = context.eco_mapping.eco_to_curie(self.6, *goref);
        if let Some(curie) = eco_curie {
            let references = self.5.clone();
            let withfrom: ListField<Conjunction<Curie>> = self.7.map_new(|curie| Conjunction::new(vec![curie.clone()]));
            Ok(Evidence::new(curie.clone(), references, withfrom))
        } else {
            Err(format!("Could not find ECO CURIE for `{:?}`", &self.6))
        }
    }
}

impl HasMetadata<String> for BaseGaf2_1Row {

    fn metadata(&self, _: &Context) -> Result<Metadata, String> {

        let interacting_taxon = match &self.12 {
            OneOrTwoItems::Two(_, t) => Some(t.clone()),
            _ => None
        };

        let negated = self.3.as_ref().map(|either_both| {
            match either_both {
                EitherOrBoth::Both(_, _) => true,
                EitherOrBoth::Left(_) => true,
                _ => false
            }
        }).unwrap_or(false);

        Ok(Metadata {
            negated,
            aspect: Some(self.8),
            interacting_taxon,
            provided_by: self.14.clone(),
            date: self.13.clone(),
            properties: ListField::new(vec![])
        })
    }
}


impl HasExtensions<String> for BaseGaf2_1Row {

    fn extensions(&self, context: &Context) -> Result<Extensions, String> {

        let subject_extension = self.16.as_ref()
            .map(|sub| ClassExpression::new(Curie::new("rdfs", "subClassOf"), sub.clone()));
        
        // Turn ClassExpression with Label into ClassExpression with Curie
        let map_label_expression = |label_expr: &ClassExpression<Label, Curie>| {
            
            let ClassExpression { relation, filler } = label_expr;
            context.label_to_curie(&relation)
                .ok_or(format!("Could not find relation CURIE for `{}`", relation.0))
                .map(|curie_rel| ClassExpression::new(curie_rel, filler.clone()))
        }; // Result<ClassExpression<Curie, Curie>, String>

        let object_extension = Ok(&self.15)
            .and_then(|obj_ext: &ListField<Conjunction<ClassExpression<Label, Curie>>>| 
                // convert to Result<ListField<Conjunction<ClassExpression<Curie, Curie>>>>
                obj_ext.map_new_results(|conjunctions|
                    conjunctions.map_new_results(|expression|
                        map_label_expression(expression)))
            );

        object_extension.map(|obj_extension| Extensions::new(subject_extension, obj_extension))
    }
}
