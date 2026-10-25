//! rdf-array: `@list` is a sequence term, and a sequence is written as `@list`.
#![expect(clippy::unwrap_used)]

use oxjsonld::{JsonLdParser, JsonLdSerializer};
use oxrdf::{GraphName, Literal, NamedNode, Quad, Sequence, Term};

fn parse(data: &str) -> Vec<Quad> {
    JsonLdParser::new()
        .for_slice(data)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn ex(local: &str) -> NamedNode {
    NamedNode::new_unchecked(format!("http://ex/{local}"))
}

fn int(i: i64) -> Term {
    Literal::from(i).into()
}

#[test]
fn list_is_a_sequence() {
    let quads = parse(
        r#"{"@id": "http://ex/s", "http://ex/p": {"@list": [1, {"@list": [2, 3]}, {"@list": []}, {"@id": "http://ex/a"}]}}"#,
    );
    assert_eq!(
        quads,
        vec![Quad::new(
            ex("s"),
            ex("p"),
            Sequence::new([
                int(1),
                Sequence::new([int(2), int(3)]).into(),
                Sequence::empty().into(),
                ex("a").into()
            ]),
            GraphName::DefaultGraph
        )]
    );
}

#[test]
fn list_with_a_node() {
    let quads = parse(
        r#"{"@id": "http://ex/s", "http://ex/p": {"@list": [{"http://ex/q": 1}]}}"#,
    );
    assert_eq!(quads.len(), 2, "{quads:?}");
    let inner = quads.iter().find(|q| q.predicate == ex("q")).unwrap();
    let outer = quads.iter().find(|q| q.predicate == ex("p")).unwrap();
    assert_eq!(
        outer.object,
        Term::from(Sequence::new([Term::from(inner.subject.clone())]))
    );
}

#[test]
fn context_list_container() {
    let quads = parse(
        r#"{"@context": {"p": {"@id": "http://ex/p", "@container": "@list"}}, "@id": "http://ex/s", "p": ["a"]}"#,
    );
    assert_eq!(
        quads[0].object,
        Term::from(Sequence::new([Literal::new_simple_literal("a")]))
    );
}

#[test]
fn round_trip() {
    let quads = vec![Quad::new(
        ex("s"),
        ex("p"),
        Sequence::new([int(1), Sequence::empty().into(), ex("a").into()]),
        GraphName::DefaultGraph,
    )];
    let mut serializer = JsonLdSerializer::new().for_writer(Vec::new());
    for q in &quads {
        serializer.serialize_quad(q).unwrap();
    }
    let written = String::from_utf8(serializer.finish().unwrap()).unwrap();
    assert!(written.contains("@list"), "{written}");
    assert_eq!(parse(&written), quads);
    let mut serializer = JsonLdSerializer::new().for_writer(Vec::new());
    assert!(
        serializer
            .serialize_quad(&Quad::new(
                Sequence::empty(),
                ex("p"),
                ex("o"),
                GraphName::DefaultGraph
            ))
            .is_err()
    );
}
