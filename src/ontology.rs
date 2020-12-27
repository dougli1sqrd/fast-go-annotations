use daggy::Dag;
use fastobo_graphs::model::{Graph, Node, NodeType};
use daggy::{NodeIndex, EdgeIndex};
use daggy::petgraph::visit::{IntoNodeReferences, GraphBase};
use daggy::walker::{Filter};

use std::iter::FromIterator;

use daggy::Walker;
// use daggy::petgraph::visit::

use std::collections::HashMap;
use std::collections::HashSet;

use crate::annotation::fields::Aspect;

///
/// When choosing which relations to traverse the ontology
/// with, `All` is for traversing any relation, `Listed`
/// for traversing relations only with IDs included in the associated
/// Vec<String>, or by default SubClassOf ('is_a').
#[derive(PartialEq, Debug)]
pub enum AllowedRelations<R> {
    All,
    Listed(Vec<R>),
    SubClassOf
}

pub trait IsSubClassOf {
    fn is_subclass_of(&self) -> bool;
}

impl IsSubClassOf for String {
    fn is_subclass_of(&self) -> bool {
        self == "is_a"
    }
}

impl IsSubClassOf for &str {
    fn is_subclass_of(&self) -> bool {
        *self == "is_a"
    }
}

impl<R: IsSubClassOf+PartialEq> AllowedRelations<R> {
    fn contains_relation(&self, relation: &R) -> bool {
        match self {
            AllowedRelations::All => true,
            AllowedRelations::Listed(allowed) => allowed.contains(&relation),
            AllowedRelations::SubClassOf => relation.is_subclass_of()
        }
    }
}

impl<R> Default for AllowedRelations<R> {
    fn default() -> Self {
        AllowedRelations::SubClassOf
    }
}

///
/// Create an AllowedRelations from a &str.
/// This means the internal R of AllowedRelation needs
/// to be able to be also created from a &str (since that's what's given)
impl<'a, R: From<&'a str>> From<&'a str> for AllowedRelations<R> {
    fn from(s: &'a str) -> AllowedRelations<R> {
        match s {
            "" => AllowedRelations::default(),
            v => AllowedRelations::Listed(vec![R::from(v)])
        }
    }
}

impl<R: Into<String>> From<String> for AllowedRelations<R> {
    fn from(s: String) -> AllowedRelations<R> {
        s.into()
    }
}

impl<R: Into<String>> From<Option<String>> for AllowedRelations<R> {
    fn from(opt: Option<String>) -> AllowedRelations<R> {
        match opt {
            Some(s) => s.into(),
            None => AllowedRelations::All
        }
    }
}

impl From<Vec<String>> for AllowedRelations<String> {
    fn from(list: Vec<String>) -> AllowedRelations<String> {
        if list.is_empty() {
            AllowedRelations::default()
        } else {
            AllowedRelations::Listed(list)
        }
    }
}

trait WeightedGraph<N, E>: GraphBase {

    fn edge_weight(&self, edge: Self::EdgeId) -> Option<&E>;

    fn edge_weight_mut(&mut self, edge: Self::EdgeId) -> Option<&mut E>;

    fn node_weight(&self, node: Self::NodeId) -> Option<&N>;

    fn node_weight_mut(&mut self, node: Self::NodeId) -> Option<&mut N>;
}

impl<N, E> WeightedGraph<N, E> for Dag<N, E> {
    
    fn edge_weight(&self, edge: Self::EdgeId) -> Option<&E> {
        self.edge_weight(edge)
    }

    fn edge_weight_mut(&mut self, edge: Self::EdgeId) -> Option<&mut E> {
        self.edge_weight_mut(edge)
    }

    fn node_weight(&self, node: Self::NodeId) -> Option<&N> {
        self.node_weight(node)
    }

    fn node_weight_mut(&mut self, node: Self::NodeId) -> Option<&mut N> {
        self.node_weight_mut(node)
    }
}


pub struct Ontology {
    node_id_to_index: HashMap<String, NodeIndex>,
    graph: daggy::Dag<Node, String>,
}

impl Ontology {
    pub fn from_obo_graph(obo: &Graph) -> Ontology {
        let mut node_index_lookup: HashMap<String, NodeIndex> = HashMap::new();
        let mut dag: daggy::Dag<Node, String> = daggy::Dag::new();

        for node in &obo.nodes {
            let n: Node = node.clone();
            let id = n.id.clone();
            let index = dag.add_node(n);
            node_index_lookup.insert(id, index);
        }

        for edge in &obo.edges {
            let subject = node_index_lookup.get(&edge.sub);
            let object = node_index_lookup.get(&edge.obj);
            if let (Some(s), Some(o)) = (subject, object) {
                // Confusing, but yes, we switch the order here
                let _ = dag.add_edge(*o, *s, edge.pred.clone());
            }
        }

        Ontology { 
            node_id_to_index: node_index_lookup,
            graph: dag,
        }
    }

    fn node_id_to_index(&self, id: String) -> Option<NodeIndex> {
        self.node_id_to_index.get(&id).copied()
    }

    pub fn get_node(&self, id: String) -> Option<&Node> {
        match self.graph.node_weight(*self.node_id_to_index.get(&id)?) {
            Some(w) => Some(&w),
            _ => None
        }
    }

    pub fn node_filter<F: FnMut(&Node) -> bool>(&self, mut f: F) -> Vec<&Node>{
        self.graph.node_references()
            .map(|(_, node)| node)
            .filter(|node| f(*node))
            .collect()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn node(&self, id: String) -> Option<&Node> {
        self.node_id_to_index(id).and_then(|index| self.graph.node_weight(index))
    }

    pub fn nodes(&self) -> Vec<&Node> {
        self.graph.node_references().map(|(_, node)| node).collect()
    }

    pub fn has_node(&self, id: String) -> bool {
        self.node_id_to_index.contains_key(&id)
    }

    pub fn node_type(&self, id: String) -> Option<NodeType> {
        match self.node_id_to_index(id) {
            Some(index) => match self.graph.node_weight(index) {
                Some(node) => node.ty.clone(),
                _ => None
            },
            _ => None
        }
    }

    /// Gets the immediate children of `node` along relations specified in `relations`.
    /// Anything passed to `relations` must be able to be made into an AllowedRelations<String>.
    /// This includes Vec<String>, Option<String>, and String (as well as &str versions).
    /// Empty `String`s and `Vec`s are considered to be "SubClassOf", or in the graph
    /// representation here more specifically "is_a", by default.
    /// 
    /// This builds a `daggy::Filter` `Walker` that only accepts nodes whose 
    /// edges (corresponding to Relations) are contained in the set of relations
    /// of `relations`.
    pub fn children<R>(&self, node: String, relations: R) -> Vec<&Node>
        where
            R: Into<AllowedRelations<String>> {

        if let Some(id) = self.node_id_to_index(node) {
            let allowed_relations = relations.into();
            let children = self.graph.children(id);

            let filter = daggy::walker::Filter::new(children, |g, (edge, _)| {
                if let Some(edge_rel) = g.edge_weight(*edge) {
                    allowed_relations.contains_relation(edge_rel)
                } else {
                    false
                }
            });

            filter.iter(&self.graph)
                .filter_map(|(_, node)| { self.graph.node_weight(node) })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn descendants<R>(&self, node: String, relations: R) -> Vec<&Node>
        where
            R: Into<AllowedRelations<String>> 
    {
        if let Some(start) = self.node_id_to_index(node) {
            let allowed_relations = relations.into();

            let mut visited: HashSet<NodeIndex> = HashSet::new();
            let mut accumulated: Vec<(EdgeIndex, NodeIndex)> = vec![];

            // 0. Visit the current_node by adding it to the `visited` set.
            // 1. Get the children of the current_node.
            // 2. Make a filter based on the above children, to only walk on allowed relations
            // 3. Collect the Node Ids, and add them to the accumulator
            // 4. Pop the accumulator skipping nodes that are visited until we get Some()
            let descendants_walker = self.graph.recursive_walk(start, |g, current_node| {
                visited.insert(current_node);
                let children = g.children(current_node);
                let seen_filter = Filter::new(children, |_, (_, node)| {
                    !visited.contains(node)
                });
                let relation_filter = Filter::new(seen_filter, |g, (edge, _)| {
                    if let Some(edge_rel) = g.edge_weight(*edge) {
                        allowed_relations.contains_relation(edge_rel)
                    } else {
                        // edge not in the graph?
                        false
                    }
                });
                accumulated.extend(relation_filter.iter(g));
                // `(edge, node)`s `pop`ped here are already filtered for seen in `seen_filter`
                accumulated.pop()
            });

            descendants_walker.iter(&self.graph)
                .filter_map(|(_, n)| { self.graph.node_weight(n) })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn descendants_closure<R>(&self, node: String, relations: R) -> Closure
        where
            R: Into<AllowedRelations<String>> + Clone 
    {
        let descendants = self.descendants(node.clone(), relations.clone());
        let rels = match relations.into() {
            AllowedRelations::All => None,
            AllowedRelations::SubClassOf => Some(vec!["is_a".into()]),
            AllowedRelations::Listed(v) => Some(v)
        };

        Closure::new(node, rels, descendants.iter().map(|node| node.id.clone()))
    }
}

///
/// I want a type that expresses the idea that we've got a simple, full ontology closure
/// on some number of relations. Since in the rules, I'll often want to lookup if a term
/// that appears in a line is in some sub-class closure of some other term.
/// 
/// I'm not sure of other uses for this though. Initially it seems like all I want
/// is a collection of term names that are in the closure of some other term.
/// 
/// Maybe we would want to have metadata around still, what relations were used
/// to compute the closure, etc?
pub struct Closure {
    relations: Option<Vec<String>>,
    top_term: String,
    terms: HashSet<String>
}

use std::iter::IntoIterator;

impl Closure {

    pub fn new<Col>(top: String, relations: Option<Vec<String>>, terms: Col) -> Closure
        where
            Col: IntoIterator<Item=String>
    {
        Closure {
            relations,
            top_term: top,
            terms: HashSet::from_iter(terms)
        }
    }

    pub fn contains(&self, term: String) -> Contained {
        if self.top_term == term {
            Contained::AsClosureTerm
        } else if self.terms.contains(&term) {
            Contained::InClosure
        } else {
            Contained::Outside
        }
    }

    pub fn closed_over_relations(&self) -> Option<Vec<&str>> {
        self.relations.as_ref().map(|rels| {
            rels.iter().map(|s| s.as_ref()).collect()
        })
    }
}

#[derive(Clone, Copy)]
pub enum Contained {
    InClosure,
    Outside,
    /// This variant for expressing that the term in question is
    /// in the closure, but only as the root. if closure of A is
    /// X, Y, Z, and we asked if A is in the closure, we would use
    /// this variant.
    AsClosureTerm
}

impl Default for Ontology {
    fn default() -> Ontology {
        Ontology {
            node_id_to_index: HashMap::new(),
            graph: daggy::Dag::new(),
        }
    }
}

pub trait NodeDeprecated {
    fn deprecated(&self) -> bool;

    fn replaced_by(&self) -> Option<String>;
}

pub trait NodeAspect {
    fn aspect(&self) -> Option<Aspect>;
}

impl NodeDeprecated for Node {
    fn deprecated(&self) -> bool {
        // Get meta.deprecated. If true, then we're done.
        // If it's None or Some(false) then try looking in the basic_property_values
        match self.meta.as_ref().map(|m| m.deprecated ) {
            Some(d) if d => d,
            _ => {
                let deprecated = "http://www.w3.org/2002/07/owl#deprecated";
                self.meta.as_ref().map(|meta| {
                    meta.basic_property_values.iter()
                        .filter(|propval| propval.pred == deprecated)
                        .take(1)
                        .next()
                        .map(|propval| propval.val == "true")
                }).flatten()
                    .unwrap_or(false)
            }
        }
    }

    fn replaced_by(&self) -> Option<String> {
        let replaced_by = "http://purl.obolibrary.org/obo/IAO_0100001";
        self.meta.as_ref().map(|meta| {
            meta.basic_property_values.iter()
                .filter(|propval| propval.pred == replaced_by)
                .take(1)
                .next()
                .map(|propval| propval.val.clone())
        }).flatten()
    }
}

impl NodeAspect for Node {
    fn aspect(&self) -> Option<Aspect> {
        let obo_namespace = "http://www.geneontology.org/formats/oboInOwl#hasOBONamespace";
        let namespace = self.meta.as_ref().map(|meta| {
            meta.basic_property_values.iter()
                .filter(|propval| propval.pred == obo_namespace)
                .take(1)
                .next()
                .map(|propval| propval.val.clone())
        }).flatten();
        match namespace?.as_ref() {
            "biological_process" => Some(Aspect::BioProcess),
            "molecular_function" => Some(Aspect::MolecularFunction),
            "cellular_component" => Some(Aspect::CellComponent),
            _ => None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource;

    #[test]
    fn test_deprecated_term_with_replaced_by() {
        let ontology = resource::load_ontology("resources/alt_id_ont.json").unwrap();
        let term = ontology.get_node("http://purl.obolibrary.org/obo/GO_1".into()).unwrap();
        
        assert_eq!(term.deprecated(), true);
        assert_eq!(term.replaced_by(), Some("http://purl.obolibrary.org/obo/GO_2".into()));
    }

    #[test]
    fn test_deprecated_term_with_deprecated_field() {
        let ontology = resource::load_ontology("resources/alt_id_ont.json").unwrap();
        let term = ontology.get_node("http://purl.obolibrary.org/obo/GO_3".into()).unwrap();

        assert_eq!(term.deprecated(), true);
    }

}

