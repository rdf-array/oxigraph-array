//! rdf-array: sequences in the store, through loading, SPARQL, updates and dumps.
#![cfg(test)]
#![expect(clippy::unwrap_used, clippy::panic)]

use oxigraph::io::RdfFormat;
use oxigraph::model::*;
use oxigraph::sparql::{QueryResults, SparqlEvaluator};
use oxigraph::store::Store;

const DATA: &str = r#"@prefix : <http://example/> .
:x :p (1 2 3) .
:y :p (:a (:b :c)) .
:z :p () .
(:a :b) :q :o .
:v :coding ( [ :system :loinc ; :code "1234-5" ] ) .
:t :tags ("red" "green") .
"#;

fn store() -> Store {
    let store = Store::new().unwrap();
    store.load_from_slice(RdfFormat::Turtle, DATA).unwrap();
    store
}

fn select(store: &Store, query: &str) -> Vec<String> {
    let QueryResults::Solutions(solutions) = SparqlEvaluator::new()
        .with_prefix("", "http://example/")
        .unwrap()
        .parse_query(query)
        .unwrap()
        .on_store(store)
        .execute()
        .unwrap()
    else {
        panic!("not a SELECT")
    };
    solutions
        .map(|s| {
            s.unwrap()
                .iter()
                .map(|(v, t)| format!("{}={t}", v.as_str()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

#[test]
fn load_and_lookup() {
    let store = store();
    assert_eq!(store.len().unwrap(), 8);
    let ex = |l: &str| NamedNode::new_unchecked(format!("http://example/{l}"));
    let seq = Sequence::new([ex("a"), ex("b")]);
    assert_eq!(
        store
            .quads_for_pattern(Some((&seq).into()), None, None, None)
            .count(),
        1
    );
    assert!(
        store
            .contains(QuadRef::new(
                &ex("z"),
                &ex("p"),
                &Sequence::empty(),
                GraphNameRef::DefaultGraph
            ))
            .unwrap()
    );
    assert!(
        !store
            .contains(QuadRef::new(
                &ex("z"),
                &ex("p"),
                vocab::rdf::NIL,
                GraphNameRef::DefaultGraph
            ))
            .unwrap()
    );
    // no ladder was stored
    assert_eq!(
        store
            .quads_for_pattern(None, Some(vocab::rdf::FIRST), None, None)
            .count(),
        0
    );
}

#[test]
fn query() {
    let store = store();
    assert_eq!(
        select(&store, "SELECT ?c { ?s :coding ( [ :code ?c ] ) }"),
        vec!["c=\"1234-5\""]
    );
    assert_eq!(
        select(&store, "SELECT ?e { :y :p ((((?e))[1]))[2] }"),
        vec!["e=<http://example/b>"]
    );
    assert_eq!(
        select(&store, "SELECT ?x ?o { ( :a ?x ) :q ?o }"),
        vec!["x=<http://example/b> o=<http://example/o>"]
    );
    assert_eq!(
        select(&store, "SELECT ?n { :x :p ?s BIND(SEQLEN(?s) AS ?n) }"),
        vec!["n=\"3\"^^<http://www.w3.org/2001/XMLSchema#integer>"]
    );
}

#[test]
fn update() {
    let store = store();
    let update = |u: &str| {
        SparqlEvaluator::new()
            .with_prefix("", "http://example/")
            .unwrap()
            .parse_update(u)
            .unwrap()
            .on_store(&store)
            .execute()
            .unwrap()
    };
    update("INSERT DATA { :n :p (4 5) . :n :q () . (:k) :r [ :s (:l) ] }");
    assert_eq!(
        select(&store, "SELECT ?x { :n :p ( 4 ?x ) }"),
        vec!["x=\"5\"^^<http://www.w3.org/2001/XMLSchema#integer>"]
    );
    assert_eq!(
        select(&store, "SELECT ?l { (:k) :r [ :s ( ?l ) ] }"),
        vec!["l=<http://example/l>"]
    );
    update("DELETE DATA { :n :p (4 5) }");
    assert!(select(&store, "SELECT ?x { :n :p ?x }").is_empty());
    update(
        "DELETE { ?s :tags ?seq } INSERT { ?s :tag ?t } WHERE { ?s :tags ?seq . ?s :tags ((?t))[*] }",
    );
    assert_eq!(
        select(&store, "SELECT ?t { :t :tag ?t } ORDER BY ?t"),
        vec!["t=\"green\"", "t=\"red\""]
    );
    update("INSERT { ?s :pair ( ?t [ :of ?s ] ) } WHERE { ?s :tag ?t }");
    assert_eq!(
        select(&store, "SELECT ?t { :t :pair ( ?t [ :of :t ] ) }").len(),
        2
    );
}

#[test]
fn dump_and_reload() {
    let store = store();
    for format in [RdfFormat::NQuads, RdfFormat::TriG] {
        let dump = store.dump_to_writer(format, Vec::new()).unwrap();
        let again = Store::new().unwrap();
        again.load_from_slice(format, &dump).unwrap();
        assert_eq!(again.len().unwrap(), store.len().unwrap(), "{format}");
        let mut a = Dataset::from_iter(store.iter().map(Result::unwrap));
        let mut b = Dataset::from_iter(again.iter().map(Result::unwrap));
        a.canonicalize(dataset::CanonicalizationAlgorithm::Unstable);
        b.canonicalize(dataset::CanonicalizationAlgorithm::Unstable);
        assert_eq!(a, b, "{format}");
    }
}
