//! rdf-array: rdf:parseType="Collection" is a sequence term, rdf:li stays rdf:li.
#![cfg(test)]
#![expect(clippy::unwrap_used)]

use oxrdf::vocab::rdf;
use oxrdf::{BlankNode, Literal, NamedNode, Sequence, Term, Triple};
use oxrdfxml::{RdfXmlParser, RdfXmlSerializer};

fn parse(data: &str) -> Vec<Triple> {
    RdfXmlParser::new()
        .for_slice(data)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn ex(local: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("http://ex/{local}"))
}

const HEAD: &str =
    r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:ex="http://ex/">"#;

#[test]
fn collection_is_a_sequence() {
    let triples = parse(&format!(
        r#"{HEAD}<rdf:Description rdf:about="http://ex/s"><ex:p rdf:parseType="Collection"><rdf:Description rdf:about="http://ex/a"/><rdf:Description rdf:nodeID="b"/></ex:p></rdf:Description></rdf:RDF>"#
    ));
    assert_eq!(
        triples,
        vec![Triple::new(
            ex("s"),
            ex("p"),
            Sequence::new([Term::from(ex("a")), BlankNode::new_unchecked("b").into()])
        )]
    );
}

#[test]
fn empty_collection() {
    for body in [
        r#"<ex:p rdf:parseType="Collection"></ex:p>"#,
        r#"<ex:p rdf:parseType="Collection"/>"#,
    ] {
        let triples = parse(&format!(
            r#"{HEAD}<rdf:Description rdf:about="http://ex/s">{body}</rdf:Description></rdf:RDF>"#
        ));
        assert_eq!(
            triples,
            vec![Triple::new(ex("s"), ex("p"), Sequence::empty())]
        );
    }
}

#[test]
fn li_is_not_numbered() {
    let triples = parse(&format!(
        r#"{HEAD}<rdf:Seq rdf:about="http://ex/s"><rdf:li>a</rdf:li><rdf:li>b</rdf:li></rdf:Seq></rdf:RDF>"#
    ));
    assert_eq!(
        triples,
        vec![
            Triple::new(ex("s"), rdf::TYPE, rdf::SEQ),
            Triple::new(
                ex("s"),
                NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#li"),
                Literal::new_simple_literal("a")
            ),
            Triple::new(
                ex("s"),
                NamedNode::new_unchecked("http://www.w3.org/1999/02/22-rdf-syntax-ns#li"),
                Literal::new_simple_literal("b")
            ),
        ]
    );
}

#[test]
fn round_trip() {
    let triples = vec![
        Triple::new(
            ex("s"),
            ex("p"),
            Sequence::new([Term::from(ex("a")), BlankNode::new_unchecked("b").into()]),
        ),
        Triple::new(ex("s"), ex("q"), Sequence::empty()),
    ];
    let mut serializer = RdfXmlSerializer::new()
        .with_prefix("ex", "http://ex/")
        .unwrap()
        .for_writer(Vec::new());
    for t in &triples {
        serializer.serialize_triple(t).unwrap();
    }
    let written = String::from_utf8(serializer.finish().unwrap()).unwrap();
    assert!(
        written.contains(r#"rdf:parseType="Collection""#),
        "{written}"
    );
    assert_eq!(parse(&written), triples);
}

#[test]
fn not_writable() {
    for t in [
        Triple::new(Sequence::empty(), ex("p"), ex("o")),
        Triple::new(
            ex("s"),
            ex("p"),
            Sequence::new([Literal::new_simple_literal("a")]),
        ),
        Triple::new(
            ex("s"),
            ex("p"),
            Sequence::new([Term::from(Sequence::empty())]),
        ),
    ] {
        let mut serializer = RdfXmlSerializer::new().for_writer(Vec::new());
        assert!(serializer.serialize_triple(&t).is_err(), "{t}");
    }
}
