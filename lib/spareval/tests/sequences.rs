//! rdf-array: sequence patterns, position patterns, functions, ordering and CONSTRUCT over an in-memory dataset.
#![cfg(test)]
#![expect(clippy::unwrap_used, clippy::panic)]

use oxrdf::{Dataset, GraphName, Triple};
use spareval::{QueryEvaluator, QueryResults};
use spargebra::SparqlParser;
use std::str::FromStr;

const DATA: &str = "
<http://example/x> <http://example/p> (\"1\"^^<http://www.w3.org/2001/XMLSchema#integer> \"2\"^^<http://www.w3.org/2001/XMLSchema#integer> \"3\"^^<http://www.w3.org/2001/XMLSchema#integer>) .
<http://example/y> <http://example/p> (<http://example/a> (<http://example/b> <http://example/c>)) .
<http://example/z> <http://example/p> () .
<http://example/w> <http://example/p> \"not a sequence\" .
<http://example/s> <http://example/p> (\"10\"^^<http://www.w3.org/2001/XMLSchema#integer> \"20\"^^<http://www.w3.org/2001/XMLSchema#integer> \"30\"^^<http://www.w3.org/2001/XMLSchema#integer>) .
<http://example/s> <http://example/idx> \"2\"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://example/r> <http://example/q> (\"5\"^^<http://www.w3.org/2001/XMLSchema#integer> \"5\"^^<http://www.w3.org/2001/XMLSchema#integer>) .
<http://example/t> <http://example/q> (\"5\"^^<http://www.w3.org/2001/XMLSchema#integer> \"6\"^^<http://www.w3.org/2001/XMLSchema#integer>) .
";

fn dataset() -> Dataset {
    DATA.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            Triple::from_str(l)
                .unwrap()
                .in_graph(GraphName::DefaultGraph)
        })
        .collect()
}

/// Each solution as "var=value" pairs in variable order, values in N-Triples form.
fn select(query: &str) -> Vec<String> {
    let query = SparqlParser::new()
        .with_prefix("", "http://example/")
        .unwrap()
        .parse_query(query)
        .unwrap();
    let dataset = dataset();
    let QueryResults::Solutions(solutions) = QueryEvaluator::new()
        .prepare(&query)
        .execute(&dataset)
        .unwrap()
    else {
        panic!("not a SELECT")
    };
    solutions
        .map(|s| {
            let s = s.unwrap();
            s.iter()
                .map(|(v, t)| format!("{}={t}", v.as_str()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

fn int(i: i64) -> String {
    format!("\"{i}\"^^<http://www.w3.org/2001/XMLSchema#integer>")
}

#[test]
fn sequence_with_variables() {
    assert_eq!(
        select("SELECT ?s ?b { ?s :p ( ?a ?b 3 ) }"),
        vec![format!("s=<http://example/x> b={}", int(2))]
    );
    assert_eq!(
        select("SELECT ?y { ?s :p ( :a ( ?x ?y ) ) }"),
        vec!["y=<http://example/c>".to_owned()]
    );
    assert_eq!(
        select("SELECT ?s { ?s :p () }"),
        vec!["s=<http://example/z>"]
    );
    assert_eq!(
        select("SELECT ?s { ?s :q ( ?v ?v ) }"),
        vec!["s=<http://example/r>"]
    );
    assert_eq!(
        select("SELECT ?s { ?s :p ( ?a ?b ) }"),
        vec!["s=<http://example/y>"],
        "only the two-element sequence"
    );
    assert!(
        select("SELECT ?s { ?s :p ( ?a ?b ?c ?d ) }").is_empty(),
        "length mismatch"
    );
    assert_eq!(
        select("SELECT ?s { ?s :p ( 1 2 3 ) }"),
        vec!["s=<http://example/x>"]
    );
}

#[test]
fn position_patterns() {
    assert_eq!(
        select("SELECT ?e { :x :p ((?e))[2] }"),
        vec![format!("e={}", int(2))]
    );
    assert!(select("SELECT ?e { :x :p ((?e))[4] }").is_empty());
    assert_eq!(
        select("SELECT ?i ?e { :x :p ((?e))[?i] } ORDER BY ?i"),
        (1..=3)
            .map(|i| format!("i={} e={}", int(i), int(i)))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        select("SELECT ?s ?e { ?s :p ((?e))[*] } ORDER BY ?s ?e").len(),
        3 + 2 + 3
    );
    assert_eq!(
        select("SELECT ?e { :y :p ((((?e))[1]))[2] }"),
        vec!["e=<http://example/b>"]
    );
    assert_eq!(
        select("SELECT ?s { ?s :p ((:a))[1] }"),
        vec!["s=<http://example/y>"]
    );
    assert!(select("SELECT ?s { ?s :p ((:a))[2] }").is_empty());
    assert_eq!(
        select("SELECT ?e { :s :idx ?i . :s :p ((?e))[?i] }"),
        vec![format!("e={}", int(20))]
    );
}

#[test]
fn functions() {
    assert_eq!(
        select("SELECT ?s ?n ?second { ?s :p ?seq . FILTER(isSEQ(?seq)) BIND(SEQLEN(?seq) AS ?n) BIND(?seq[2] AS ?second) } ORDER BY ?s"),
        vec![
            format!("s=<http://example/s> n={} second={}", int(3), int(20)),
            format!("s=<http://example/x> n={} second={}", int(3), int(2)),
            "s=<http://example/y> n=\"2\"^^<http://www.w3.org/2001/XMLSchema#integer> second=(<http://example/b> <http://example/c>)".to_owned(),
            format!("s=<http://example/z> n={}", int(0)),
        ]
    );
    assert_eq!(
        select("SELECT ?s { ?s :p ?o FILTER(!isSEQ(?o)) }"),
        vec!["s=<http://example/w>"]
    );
    assert_eq!(
        select("SELECT ?e { :y :p ?seq BIND(?seq[2][1] AS ?e) }"),
        vec!["e=<http://example/b>"]
    );
    assert_eq!(
        select("SELECT ?seq { BIND(SEQ(1, :a, SEQ()) AS ?seq) }"),
        vec![format!("seq=({} <http://example/a> ())", int(1))]
    );
    assert_eq!(
        select("SELECT ?s { ?s :p ?seq FILTER(?seq = SEQ(1, 2, 3)) }"),
        vec!["s=<http://example/x>"]
    );
}

#[test]
fn order_by() {
    assert_eq!(
        select("SELECT ?o { ?s :p ?o } ORDER BY ?o"),
        vec![
            "o=\"not a sequence\"".to_owned(),
            "o=()".to_owned(),
            // element-wise, with SPARQL's order: IRIs before literals
            "o=(<http://example/a> (<http://example/b> <http://example/c>))".to_owned(),
            format!("o=({} {} {})", int(1), int(2), int(3)),
            format!("o=({} {} {})", int(10), int(20), int(30)),
        ]
    );
}

#[test]
fn construct() {
    let query = SparqlParser::new()
        .parse_query("PREFIX : <http://example/> CONSTRUCT { ?s :r ( ?e [ :k ?e ] ) } WHERE { ?s :p ((?e))[1] }")
        .unwrap();
    let dataset = dataset();
    let QueryResults::Graph(triples) = QueryEvaluator::new()
        .prepare(&query)
        .execute(&dataset)
        .unwrap()
    else {
        panic!()
    };
    let triples = triples.collect::<Result<Vec<_>, _>>().unwrap();
    assert_eq!(triples.len(), 3 * 2, "{triples:?}");
    assert!(triples.iter().any(|t| t.object.is_sequence()));
}
