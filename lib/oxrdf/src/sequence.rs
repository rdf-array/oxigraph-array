//! rdf-array: the sequence term.

use crate::triple::{Term, TermRef};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// An owned RDF sequence (rdf-array): an ordered list of [terms](Term), itself a term.
///
/// It stands where RDF 1.1 builds an `rdf:first`/`rdf:rest` ladder: Turtle's `( 1 2 )` is one
/// sequence term, and `()` is the empty sequence, not `rdf:nil`. Two sequences are equal when
/// their elements are equal, in order.
///
/// The default string formatter is returning an N-Triples-array compatible representation:
/// ```
/// use oxrdf::{Literal, NamedNode, Sequence};
///
/// assert_eq!(
///     "(<http://example.com/a> \"b\")",
///     Sequence::new([
///         oxrdf::Term::from(NamedNode::new("http://example.com/a")?),
///         Literal::new_simple_literal("b").into()
///     ])
///     .to_string()
/// );
/// assert_eq!("()", Sequence::default().to_string());
/// # Result::<_,oxrdf::IriParseError>::Ok(())
/// ```
#[derive(Eq, PartialEq, Debug, Clone, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Sequence {
    elements: Vec<Term>,
}

impl Sequence {
    /// Builds a sequence from its elements.
    #[inline]
    pub fn new(elements: impl IntoIterator<Item = impl Into<Term>>) -> Self {
        Self {
            elements: elements.into_iter().map(Into::into).collect(),
        }
    }

    /// The empty sequence `()`.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    #[inline]
    pub fn elements(&self) -> &[Term] {
        &self.elements
    }

    #[inline]
    pub fn into_elements(self) -> Vec<Term> {
        self.elements
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// The element at a zero-based position.
    #[inline]
    pub fn get(&self, index: usize) -> Option<TermRef<'_>> {
        self.elements.get(index).map(Term::as_ref)
    }

    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = TermRef<'_>> {
        self.elements.iter().map(Term::as_ref)
    }
}

impl fmt::Display for Sequence {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            element.fmt(f)?;
        }
        f.write_str(")")
    }
}

impl From<Vec<Term>> for Sequence {
    #[inline]
    fn from(elements: Vec<Term>) -> Self {
        Self { elements }
    }
}

impl<T: Into<Term>> FromIterator<T> for Sequence {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::new(iter)
    }
}

impl IntoIterator for Sequence {
    type Item = Term;
    type IntoIter = std::vec::IntoIter<Term>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a> IntoIterator for &'a Sequence {
    type Item = &'a Term;
    type IntoIter = std::slice::Iter<'a, Term>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

#[cfg(test)]
#[expect(clippy::panic_in_result_fn)]
mod tests {
    use crate::*;
    use std::str::FromStr;

    #[test]
    fn sequence_display_and_parse() -> Result<(), TermParseError> {
        let t = Term::from_str("( 1 ( \"a\" _:b ) <http://ex/c> () )")?;
        let Term::Sequence(s) = &t else {
            panic!("not a sequence: {t}")
        };
        assert_eq!(s.len(), 4);
        assert_eq!(
            t.to_string(),
            "(\"1\"^^<http://www.w3.org/2001/XMLSchema#integer> (\"a\" _:b) <http://ex/c> ())"
        );
        assert_eq!(Term::from_str(&t.to_string())?, t);
        assert_eq!(Term::from_str("()")?, Term::Sequence(Sequence::empty()));
        assert_ne!(
            Term::from_str("()")?,
            Term::from(vocab::rdf::NIL.into_owned())
        );
        Ok(())
    }

    #[test]
    fn sequence_subject() -> Result<(), TermParseError> {
        let t = Triple::from_str("( <http://ex/a> ) <http://ex/p> ( ) .")?;
        assert!(t.subject.is_sequence());
        assert!(t.object.is_sequence());
        assert!(Quad::from_str("<http://ex/a> <http://ex/p> <http://ex/o> ( ) .").is_err());
        Ok(())
    }

    #[test]
    fn dataset_with_sequences() -> Result<(), TermParseError> {
        let p = NamedNode::new_unchecked("http://ex/p");
        let mut d = Dataset::new();
        let s1 = Sequence::new([Term::from(BlankNode::new_unchecked("x")), Literal::from(1).into()]);
        let s2 = Sequence::new([Term::from(s1.clone())]);
        d.insert(QuadRef::new(&s1, &p, &s2, GraphNameRef::DefaultGraph));
        d.insert(QuadRef::new(&s2, &p, &s1, GraphNameRef::DefaultGraph));
        assert_eq!(d.len(), 2);
        assert_eq!(d.quads_for_subject(&s1).count(), 1);
        assert_eq!(d.quads_for_object(&s1).count(), 1);
        assert_eq!(d.quads_for_subject(&Sequence::empty()).count(), 0);
        let mut e = Dataset::new();
        let t1 = Sequence::new([Term::from(BlankNode::new_unchecked("y")), Literal::from(1).into()]);
        let t2 = Sequence::new([Term::from(t1.clone())]);
        e.insert(QuadRef::new(&t1, &p, &t2, GraphNameRef::DefaultGraph));
        e.insert(QuadRef::new(&t2, &p, &t1, GraphNameRef::DefaultGraph));
        assert_ne!(d, e);
        d.canonicalize(dataset::CanonicalizationAlgorithm::Unstable);
        e.canonicalize(dataset::CanonicalizationAlgorithm::Unstable);
        assert_eq!(d, e);
        Ok(())
    }
}
