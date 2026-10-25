//! rdf-array: sequence bindings in every results format, against rdf-tests-contrib's sparql/results fixtures.
#![cfg(test)]
#![expect(clippy::unwrap_used, clippy::panic)]

use oxrdf::{Literal, NamedNode, Sequence, Term, Variable};
use sparesults::{
    QueryResultsFormat, QueryResultsParser, QueryResultsSerializer, ReaderQueryResultsParserOutput,
};

fn ex(local: &str) -> Term {
    NamedNode::new_unchecked(format!("http://example/{local}")).into()
}

fn int(i: i64) -> Term {
    Literal::from(i).into()
}

fn expected() -> Vec<(Term, Term)> {
    vec![
        (
            ex("w"),
            Literal::new_simple_literal("not a sequence").into(),
        ),
        (ex("x"), Sequence::new([int(1), int(2), int(3)]).into()),
        (
            ex("y"),
            Sequence::new([ex("a"), Sequence::new([ex("b"), ex("c")]).into()]).into(),
        ),
        (ex("z"), Sequence::empty().into()),
    ]
}

fn parse(format: QueryResultsFormat, data: &str) -> Vec<(Term, Term)> {
    let ReaderQueryResultsParserOutput::Solutions(solutions) =
        QueryResultsParser::from_format(format)
            .for_reader(data.as_bytes())
            .unwrap()
    else {
        panic!("not solutions")
    };
    solutions
        .map(|s| {
            let s = s.unwrap();
            (s.get("s").unwrap().clone(), s.get("seq").unwrap().clone())
        })
        .collect()
}

fn serialize(format: QueryResultsFormat) -> String {
    let variables = vec![Variable::new_unchecked("s"), Variable::new_unchecked("seq")];
    let mut serializer = QueryResultsSerializer::from_format(format)
        .serialize_solutions_to_writer(Vec::new(), variables.clone())
        .unwrap();
    for (s, seq) in expected() {
        serializer
            .serialize([
                (variables[0].as_ref(), s.as_ref()),
                (variables[1].as_ref(), seq.as_ref()),
            ])
            .unwrap();
    }
    String::from_utf8(serializer.finish().unwrap()).unwrap()
}

#[test]
fn json() {
    assert_eq!(
        parse(
            QueryResultsFormat::Json,
            include_str!("sequences/sequences.srj")
        ),
        expected()
    );
    assert_eq!(
        parse(
            QueryResultsFormat::Json,
            &serialize(QueryResultsFormat::Json)
        ),
        expected()
    );
}

#[test]
fn xml() {
    assert_eq!(
        parse(
            QueryResultsFormat::Xml,
            include_str!("sequences/sequences.srx")
        ),
        expected()
    );
    assert_eq!(
        parse(QueryResultsFormat::Xml, &serialize(QueryResultsFormat::Xml)),
        expected()
    );
}

#[test]
fn tsv() {
    assert_eq!(
        parse(
            QueryResultsFormat::Tsv,
            include_str!("sequences/sequences.tsv")
        ),
        expected()
    );
    assert_eq!(
        parse(QueryResultsFormat::Tsv, &serialize(QueryResultsFormat::Tsv)),
        expected()
    );
}

#[test]
fn csv() {
    assert_eq!(
        serialize(QueryResultsFormat::Csv).replace("\r\n", "\n"),
        include_str!("sequences/sequences.csv")
    );
}
