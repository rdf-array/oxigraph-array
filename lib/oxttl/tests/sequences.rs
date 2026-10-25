//! rdf-array: collections are sequence terms in every Turtle-family syntax.
#![expect(clippy::panic, clippy::unwrap_used)]

use oxrdf::{BlankNode, Literal, NamedNode, Quad, Sequence, Subject, Term, Triple};
use oxttl::n3::N3Term;
use oxttl::{N3Parser, NQuadsParser, NTriplesParser, TriGParser, TurtleParser, TurtleSerializer};

fn turtle(data: &str) -> Vec<Triple> {
    TurtleParser::new()
        .for_slice(data)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn int(i: i64) -> Term {
    Literal::from(i).into()
}

fn ex(local: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("http://ex/{local}"))
}

#[test]
fn turtle_collections_are_sequences() {
    let triples = turtle("@prefix : <http://ex/> . :s :p ( 1 ( 2 3 ) () ) .");
    assert_eq!(triples.len(), 1);
    assert_eq!(
        triples[0].object,
        Term::from(Sequence::new([
            int(1),
            Sequence::new([int(2), int(3)]).into(),
            Sequence::empty().into()
        ]))
    );
}

#[test]
fn turtle_empty_collection_is_not_nil() {
    let triples = turtle("@prefix : <http://ex/> . :s :p () .");
    assert_eq!(triples[0].object, Term::from(Sequence::empty()));
}

#[test]
fn turtle_collection_subject() {
    let triples = turtle("@prefix : <http://ex/> . ( :a :b ) :p :o . () :p :o .");
    assert_eq!(
        triples[0].subject,
        Subject::from(Sequence::new([ex("a"), ex("b")]))
    );
    assert_eq!(triples[1].subject, Subject::from(Sequence::empty()));
}

#[test]
fn turtle_collection_with_blank_node_property_list() {
    let triples = turtle("@prefix : <http://ex/> . :s :p ( [ :q 1 ] :c ) ; :r 2 .");
    assert_eq!(triples.len(), 3, "{triples:?}");
    let inner = triples
        .iter()
        .find(|t| t.predicate == ex("q"))
        .unwrap()
        .subject
        .clone();
    let Subject::BlankNode(inner) = inner else {
        panic!()
    };
    let outer = triples.iter().find(|t| t.predicate == ex("p")).unwrap();
    assert_eq!(
        outer.object,
        Term::from(Sequence::new([Term::from(inner), ex("c").into()]))
    );
    assert!(triples.iter().any(|t| t.predicate == ex("r")));
}

#[test]
fn turtle_nested_collections_in_property_lists() {
    let triples = turtle("@prefix : <http://ex/> . ( [ :p ( 1 ) ] ) :q [ :r () ] .");
    assert_eq!(triples.len(), 3, "{triples:?}");
    let p = triples.iter().find(|t| t.predicate == ex("p")).unwrap();
    assert_eq!(p.object, Term::from(Sequence::new([int(1)])));
    let q = triples.iter().find(|t| t.predicate == ex("q")).unwrap();
    assert!(q.subject.is_sequence());
    let r = triples.iter().find(|t| t.predicate == ex("r")).unwrap();
    assert_eq!(r.object, Term::from(Sequence::empty()));
}

#[test]
fn trig_collections() {
    let quads = TriGParser::new()
        .for_slice("@prefix : <http://ex/> . :g { :s :p ( 1 2 ) }")
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);
    assert_eq!(quads[0].object, Term::from(Sequence::new([int(1), int(2)])));
}

#[test]
fn ntriples_sequences() {
    let triples = NTriplesParser::new()
        .for_slice(
            "<http://ex/s> <http://ex/p> (\"1\"^^<http://www.w3.org/2001/XMLSchema#integer> (<http://ex/a> _:b \"c\"@en) ()) .\n(<http://ex/a>) <http://ex/p> () .\n",
        )
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(triples.len(), 2);
    assert_eq!(
        triples[0].object,
        Term::from(Sequence::new([
            int(1),
            Sequence::new([
                Term::from(ex("a")),
                BlankNode::new_unchecked("b").into(),
                Literal::new_language_tagged_literal_unchecked("c", "en").into()
            ])
            .into(),
            Sequence::empty().into()
        ]))
    );
    assert!(triples[1].subject.is_sequence());
    // the N-Triples-array form reads back as the same triple
    let again = NTriplesParser::new()
        .for_slice(&format!("{} .\n", triples[0]))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(again, vec![triples[0].clone()]);
}

#[test]
fn nquads_sequences() {
    let quads = NQuadsParser::new()
        .for_slice("(<http://ex/a>) <http://ex/p> (1) <http://ex/g> .\n")
        .collect::<Result<Vec<_>, _>>();
    assert!(quads.is_err(), "N-Quads has no bare numbers");
    let quads = NQuadsParser::new()
        .for_slice("(<http://ex/a>) <http://ex/p> (\"1\") <http://ex/g> .\n")
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(
        quads,
        vec![Quad::new(
            Sequence::new([ex("a")]),
            ex("p"),
            Sequence::new([Literal::new_simple_literal("1")]),
            ex("g")
        )]
    );
    assert!(
        NQuadsParser::new()
            .for_slice("<http://ex/a> <http://ex/p> <http://ex/o> () .\n")
            .collect::<Result<Vec<_>, _>>()
            .is_err(),
        "a sequence is not a graph name"
    );
    assert!(
        NTriplesParser::new()
            .for_slice("<http://ex/a> <http://ex/p> (<http://ex/o>\n) .\n")
            .collect::<Result<Vec<_>, _>>()
            .is_err(),
        "a sequence is on one line"
    );
}

#[test]
fn n3_collections() {
    let quads = N3Parser::new()
        .for_slice("@prefix : <http://ex/> . :s :p ( ?x 1 ( ) ) .")
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(quads.len(), 1);
    let N3Term::Sequence(elements) = &quads[0].object else {
        panic!("{:?}", quads[0].object)
    };
    assert_eq!(elements.len(), 3);
    assert!(matches!(elements[0], N3Term::Variable(_)));
    assert_eq!(elements[2], N3Term::Sequence(Vec::new()));
}

#[test]
fn turtle_round_trip() {
    let data = "@prefix : <http://ex/> . ( :a ) :p ( 1 ( \"x\" ) () ) , () .";
    let triples = turtle(data);
    let mut serializer = TurtleSerializer::new()
        .with_prefix("", "http://ex/")
        .unwrap()
        .for_writer(Vec::new());
    for t in &triples {
        serializer.serialize_triple(t).unwrap();
    }
    let written = String::from_utf8(serializer.finish().unwrap()).unwrap();
    assert!(written.contains("( :a ) :p ( 1 ( \"x\" ) () ) , ()"), "{written}");
    assert_eq!(turtle(&written), triples);
}
