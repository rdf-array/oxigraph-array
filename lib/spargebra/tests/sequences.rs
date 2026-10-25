//! rdf-array: collections are sequence patterns; position patterns and the sequence functions parse.
#![cfg(test)]
#![expect(clippy::unwrap_used)]

use spargebra::{Query, SparqlParser};

fn query(q: &str) -> Query {
    SparqlParser::new()
        .with_prefix("", "http://example/")
        .unwrap()
        .parse_query(q)
        .unwrap()
}

#[test]
fn collection_is_one_pattern() {
    let q = query("SELECT * { ?s :p ( ?x ( 2 ?y ) () ) }");
    let text = q.to_sse();
    assert!(!text.contains("first"), "{text}");
    assert!(
        text.contains("(seq ?x (seq \"2\"^^<http://www.w3.org/2001/XMLSchema#integer> ?y) (seq))"),
        "{text}"
    );
    // Display writes SPARQL that reads back the same
    assert_eq!(query(&q.to_string()), q);
}

#[test]
fn position_patterns() {
    for (q, sse) in [
        ("SELECT * { ?s :p ((?e))[2] }", "(seqpos ?e 2)"),
        ("SELECT * { ?s :p ((?e))[?i] }", "(seqpos ?e ?i)"),
        ("SELECT * { ?s :p ((?e))[*] }", "(seqpos ?e *)"),
        (
            "SELECT * { ?s :p ((((?e))[1]))[2] }",
            "(seqpos (seqpos ?e 1) 2)",
        ),
        ("SELECT * { ?s :p ( ( ?e ) ) [ 2 ] }", "(seqpos ?e 2)"),
    ] {
        let parsed = query(q);
        assert!(parsed.to_sse().contains(sse), "{q}: {}", parsed.to_sse());
        assert_eq!(query(&parsed.to_string()), parsed, "{q}");
    }
    // without [ ] it is a nested sequence
    assert!(
        query("SELECT * { ?s :p ((?e)) }")
            .to_sse()
            .contains("(seq (seq ?e))")
    );
}

#[test]
fn bad_position_patterns() {
    for q in [
        "SELECT * { ?s :p ((?e))[1.5] }",
        "SELECT * { ?s :p ((?e))[] }",
        "SELECT * { ?s :p ((?e))[0] }",
        "SELECT * { ?s ( ?p ) ?o }",
        "CONSTRUCT { ?s :p ((?e))[1] } WHERE {}",
    ] {
        assert!(
            SparqlParser::new()
                .with_prefix("", "http://example/")
                .unwrap()
                .parse_query(q)
                .is_err(),
            "{q}"
        );
    }
}

#[test]
fn functions() {
    let q = query(
        "SELECT * { ?s :p ?seq FILTER(isSEQ(?seq) && SEQLEN(?seq) = 3 && ?seq = SEQ(1, :a, SEQ())) BIND(?seq[2][1] AS ?e) }",
    );
    let text = q.to_sse();
    for f in ["isseq", "seqlen", "<http://example/a> ( seq))", "seqat"] {
        assert!(text.contains(f), "{f}: {text}");
    }
    assert_eq!(query(&q.to_string()), q);
}

#[test]
fn updates() {
    for u in [
        "PREFIX : <http://example/> INSERT DATA { :n :p (4 5) . :n :q () . (1) :r :o }",
        "PREFIX : <http://example/> DELETE DATA { :n :p (4 5) }",
        "PREFIX : <http://example/> DELETE { ?s :tags ?seq } INSERT { ?s :tag ?t } WHERE { ?s :tags ?seq . ?s :tags ((?t))[*] }",
    ] {
        let parsed = SparqlParser::new().parse_update(u).unwrap();
        assert!(!parsed.to_sse().contains("first"), "{}", parsed.to_sse());
        // Display wraps a WHERE clause into a projection, so compare the second round with the first
        let again = SparqlParser::new()
            .parse_update(&parsed.to_string())
            .unwrap();
        assert_eq!(
            SparqlParser::new()
                .parse_update(&again.to_string())
                .unwrap(),
            again
        );
    }
    SparqlParser::new()
        .parse_update("PREFIX : <http://example/> INSERT { ?s :p ((?e))[1] } WHERE {}")
        .unwrap_err();
}

#[test]
fn upstream_update_round_trip_adds_a_projection() {
    let parsed = SparqlParser::new()
        .parse_update("PREFIX : <http://example/> DELETE { ?s :p ?o } WHERE { ?s :p ?o }")
        .unwrap();
    assert_ne!(
        SparqlParser::new()
            .parse_update(&parsed.to_string())
            .unwrap(),
        parsed
    );
}
