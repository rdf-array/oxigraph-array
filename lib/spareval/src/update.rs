use crate::{QueryEvaluationError, QuerySolutionIter};
#[cfg(feature = "sparql-12")]
use oxrdf::Triple;
use oxrdf::{BlankNode, GraphName, NamedNode, Quad, Sequence, Subject, Term};
use rustc_hash::FxHashMap;
use sparesults::QuerySolution;
use spargebra::term::{
    GraphNamePattern, GroundQuadPattern, GroundTermPattern, NamedNodePattern, QuadPattern,
    TermPattern,
};
#[cfg(feature = "sparql-12")]
use spargebra::term::{GroundTriplePattern, TriplePattern};
use std::collections::VecDeque;

/// A [`Quad`] to delete or insert.
#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum DeleteInsertQuad {
    Delete(Quad),
    Insert(Quad),
}

/// Output of [`PreparedDeleteInsertUpdate::execute`](super::PreparedDeleteInsertUpdate::execute).
pub struct DeleteInsertIter<'a> {
    solutions: QuerySolutionIter<'a>,
    delete: Vec<GroundQuadPattern>,
    insert: Vec<QuadPattern>,
    buffer: VecDeque<DeleteInsertQuad>,
    bnodes: FxHashMap<BlankNode, BlankNode>,
}

impl<'a> DeleteInsertIter<'a> {
    pub(crate) fn new(
        solutions: QuerySolutionIter<'a>,
        delete: Vec<GroundQuadPattern>,
        insert: Vec<QuadPattern>,
    ) -> Self {
        Self {
            solutions,
            delete,
            insert,
            buffer: VecDeque::new(),
            bnodes: FxHashMap::default(),
        }
    }
}

impl Iterator for DeleteInsertIter<'_> {
    type Item = Result<DeleteInsertQuad, QueryEvaluationError>;

    fn next(&mut self) -> Option<Result<DeleteInsertQuad, QueryEvaluationError>> {
        loop {
            if let Some(quad) = self.buffer.pop_front() {
                return Some(Ok(quad));
            }
            let solution = match self.solutions.next()? {
                Ok(solution) => solution,
                Err(e) => return Some(Err(e)),
            };
            for quad in &self.delete {
                if let Some(quad) = fill_ground_quad_pattern(quad, &solution) {
                    self.buffer.push_back(DeleteInsertQuad::Delete(quad));
                }
            }
            for quad in &self.insert {
                if let Some(quad) = fill_quad_pattern(quad, &solution, &mut self.bnodes) {
                    self.buffer.push_back(DeleteInsertQuad::Insert(quad));
                }
            }
            self.bnodes.clear();
        }
    }
}

fn fill_quad_pattern(
    quad: &QuadPattern,
    solution: &QuerySolution,
    bnodes: &mut FxHashMap<BlankNode, BlankNode>,
) -> Option<Quad> {
    Some(Quad {
        subject: Subject::try_from(fill_term_or_var(&quad.subject, solution, bnodes)?).ok()?,
        predicate: fill_named_node_or_var(&quad.predicate, solution)?,
        object: fill_term_or_var(&quad.object, solution, bnodes)?,
        graph_name: fill_graph_name_or_var(&quad.graph_name, solution)?,
    })
}

fn fill_term_or_var(
    term: &TermPattern,
    solution: &QuerySolution,
    bnodes: &mut FxHashMap<BlankNode, BlankNode>,
) -> Option<Term> {
    Some(match term {
        TermPattern::NamedNode(term) => term.clone().into(),
        TermPattern::BlankNode(bnode) => convert_blank_node(bnode, bnodes).into(),
        TermPattern::Literal(term) => term.clone().into(),
        #[cfg(feature = "sparql-12")]
        TermPattern::Triple(triple) => fill_triple_pattern(triple, solution, bnodes)?.into(),
        TermPattern::Variable(v) => solution.get(v)?.clone(),
        // rdf-array: blank nodes inside a template sequence are fresh per solution too
        TermPattern::Sequence(elements) => elements
            .iter()
            .map(|e| fill_term_or_var(e, solution, bnodes))
            .collect::<Option<Sequence>>()?
            .into(),
        // The parser rejects position patterns in templates
        TermPattern::SequencePosition { .. } => return None,
    })
}

fn fill_named_node_or_var(term: &NamedNodePattern, solution: &QuerySolution) -> Option<NamedNode> {
    Some(match term {
        NamedNodePattern::NamedNode(term) => term.clone(),
        NamedNodePattern::Variable(v) => {
            if let Term::NamedNode(s) = solution.get(v)? {
                s.clone()
            } else {
                return None;
            }
        }
    })
}

fn fill_graph_name_or_var(term: &GraphNamePattern, solution: &QuerySolution) -> Option<GraphName> {
    Some(match term {
        GraphNamePattern::NamedNode(term) => term.clone().into(),
        GraphNamePattern::DefaultGraph => GraphName::DefaultGraph,
        GraphNamePattern::Variable(v) => match solution.get(v)? {
            Term::NamedNode(node) => node.clone().into(),
            Term::BlankNode(node) => node.clone().into(),
            Term::Literal(_) => return None,
            #[cfg(feature = "sparql-12")]
            Term::Triple(_) => return None,
            Term::Sequence(_) => return None,
        },
    })
}

#[cfg(feature = "sparql-12")]
fn fill_triple_pattern(
    triple: &TriplePattern,
    solution: &QuerySolution,
    bnodes: &mut FxHashMap<BlankNode, BlankNode>,
) -> Option<Triple> {
    Some(Triple {
        subject: Subject::try_from(fill_term_or_var(&triple.subject, solution, bnodes)?).ok()?,
        predicate: fill_named_node_or_var(&triple.predicate, solution)?,
        object: fill_term_or_var(&triple.object, solution, bnodes)?,
    })
}
fn fill_ground_quad_pattern(quad: &GroundQuadPattern, solution: &QuerySolution) -> Option<Quad> {
    Some(Quad {
        subject: Subject::try_from(fill_ground_term_or_var(&quad.subject, solution)?).ok()?,
        predicate: fill_named_node_or_var(&quad.predicate, solution)?,
        object: fill_ground_term_or_var(&quad.object, solution)?,
        graph_name: fill_graph_name_or_var(&quad.graph_name, solution)?,
    })
}

fn fill_ground_term_or_var(term: &GroundTermPattern, solution: &QuerySolution) -> Option<Term> {
    Some(match term {
        GroundTermPattern::NamedNode(term) => term.clone().into(),
        GroundTermPattern::Literal(term) => term.clone().into(),
        #[cfg(feature = "sparql-12")]
        GroundTermPattern::Triple(triple) => fill_ground_triple_pattern(triple, solution)?.into(),
        GroundTermPattern::Variable(v) => solution.get(v)?.clone(),
        GroundTermPattern::Sequence(elements) => elements
            .iter()
            .map(|e| fill_ground_term_or_var(e, solution))
            .collect::<Option<Sequence>>()?
            .into(),
        GroundTermPattern::SequencePosition { .. } => return None,
    })
}

#[cfg(feature = "sparql-12")]
fn fill_ground_triple_pattern(
    triple: &GroundTriplePattern,
    solution: &QuerySolution,
) -> Option<Triple> {
    Some(Triple {
        subject: Subject::try_from(fill_ground_term_or_var(&triple.subject, solution)?).ok()?,
        predicate: fill_named_node_or_var(&triple.predicate, solution)?,
        object: fill_ground_term_or_var(&triple.object, solution)?,
    })
}

fn convert_blank_node(node: &BlankNode, bnodes: &mut FxHashMap<BlankNode, BlankNode>) -> BlankNode {
    bnodes.entry(node.clone()).or_default().clone()
}
