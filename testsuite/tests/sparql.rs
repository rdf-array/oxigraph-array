#![cfg(test)]

use anyhow::Result;
use oxigraph_testsuite::check_testsuite;

#[test]
fn sparql10_w3c_query_syntax_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql10/manifest-syntax.ttl",
        &[
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/syntax-sparql3/manifest#syn-bad-26", /* tokenizer */
        ],
    )
}

#[test]
fn sparql10_w3c_query_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql10/manifest-evaluation.ttl",
        &[
            // rdf-array: rdf-tests-array is newer than the rdf-tests v0.5.13 pinned, and these tests are new or changed
            // upstream since; rdf-tests-array left the SPARQL suites as they are
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/graph/manifest#graph-nested-2",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/graph/manifest#graph-nested-3",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/dataset/manifest#dawg-dataset-11",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/dataset/manifest#dawg-dataset-12b",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#case-insensitive-booleans",
            // Multiple writing of the same xsd:integer. Our system does some normalization.
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#dawg-str-1",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#dawg-str-2",
            // Simple literal vs xsd:string. We apply RDF 1.1
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/distinct/manifest#distinct-2",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-08",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-10",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-11",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-12",
            // DATATYPE("foo"@en) returns rdf:langString in RDF 1.1
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#dawg-datatype-2",
            // We use XSD 1.1 equality on dates
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#date-2",
            // This test relies on naive iteration on the input file
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/reduced/manifest#reduced-1",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/reduced/manifest#reduced-2",
            // Our scoping of variables introduced by GRAPH is wrong
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/graph/manifest#graph-variable-scope",
            "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/graph/manifest#graph-optional",
        ],
    )
}

#[test]
fn sparql11_query_w3c_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql11/manifest-sparql11-query.ttl",
        &[
            // rdf-array: rdf-tests-array is newer than the rdf-tests v0.5.13 pinned, and these tests are new or changed
            // upstream since; rdf-tests-array left the SPARQL suites as they are
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest#graph-no-var",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest#graph-from-named-1",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest#graph-from-named-2",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest#graph-from-named-3",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest#values-construct-group-by",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/subquery/manifest#subquery15",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/subquery/manifest#subquery16",
            // Our scoping of variables introduced by GRAPH is wrong
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/negation/manifest#graph-minus",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/aggregates/manifest#agg-empty-group-count-graph",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest#graph",
            // Our property path handling is wrong
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/property-path/manifest#zero_or_more_set_start",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/property-path/manifest#zero_or_more_set_end",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/property-path/manifest#zero_or_one_set_start",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/property-path/manifest#zero_or_one_set_end",
            // Our GROUP_CONCAT is wrong
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/aggregates/manifest#agg-groupconcat-04",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/aggregates/manifest#agg-groupconcat-06",
        ],
    )
}

#[test]
fn sparql11_federation_w3c_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql11/manifest-sparql11-fed.ttl",
        &[
            // Problem during service evaluation order
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/service/manifest#service5",
        ],
    )
}

#[test]
fn sparql11_update_w3c_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql11/manifest-sparql11-update.ttl",
        &[],
    )
}

#[test]
fn sparql11_json_w3c_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql11/json-res/manifest.ttl",
        &[],
    )
}

#[test]
fn sparql11_tsv_w3c_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql11/csv-tsv-res/manifest.ttl",
        &[
            // rdf-array: rdf-tests-array is newer than the rdf-tests v0.5.13 pinned, and these tests are new or changed
            // upstream since; rdf-tests-array left the SPARQL suites as they are
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/csv-tsv-res/manifest#csv04",
            // We do not run CSVResultFormatTest tests yet
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/csv-tsv-res/manifest#csv01",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/csv-tsv-res/manifest#csv02",
            "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/csv-tsv-res/manifest#csv03",
        ],
    )
}

#[cfg(feature = "rdf-12")]
#[test]
fn sparql12_w3c_testsuite() -> Result<()> {
    check_testsuite(
        "https://w3c.github.io/rdf-tests/sparql/sparql12/manifest.ttl",
        &[
            // rdf-array: rdf-tests-array is newer than the rdf-tests v0.5.13 pinned, and these tests are new or changed
            // upstream since; rdf-tests-array left the SPARQL suites as they are
            "https://w3c.github.io/rdf-tests/sparql/sparql12/grouping/manifest#select-variable-reuse",
            "https://w3c.github.io/rdf-tests/sparql/sparql12/codepoint-escapes/manifest#codepoint-esc-01-bad",
            "https://w3c.github.io/rdf-tests/sparql/sparql12/codepoint-escapes/manifest#codepoint-esc-03-bad",
            "https://w3c.github.io/rdf-tests/sparql/sparql12/codepoint-escapes/manifest#codepoint-esc-08",
            "https://w3c.github.io/rdf-tests/sparql/sparql12/codepoint-escapes/manifest#codepoint-esc-09",
            // TODO: https://github.com/w3c/sparql-query/issues/282
            "https://w3c.github.io/rdf-tests/sparql/sparql12/syntax-triple-terms-negative/manifest#tripleterm-subject-03",
            "https://w3c.github.io/rdf-tests/sparql/sparql12/syntax-triple-terms-negative/manifest#tripleterm-subject-06",
            // We do not prevent nested aggregate functions
            "https://w3c.github.io/rdf-tests/sparql/sparql12/syntax/manifest#nested-aggregate-functions",
        ],
    )
}

// rdf-array: SPARQL-array, in rdf-tests-contrib

#[test]
fn rdf_array_contrib_sparql_evaluation_testsuite() -> Result<()> {
    check_testsuite(
        "https://rdf-array.github.io/rdf-tests-contrib/sparql/eval/manifest.ttl",
        &[],
    )
}

#[test]
fn rdf_array_contrib_sparql_update_testsuite() -> Result<()> {
    check_testsuite(
        "https://rdf-array.github.io/rdf-tests-contrib/sparql/update/manifest.ttl",
        &[],
    )
}

#[test]
fn rdf_array_contrib_sparql_syntax_testsuite() -> Result<()> {
    check_testsuite(
        "https://rdf-array.github.io/rdf-tests-contrib/sparql/syntax-query/manifest.ttl",
        &[],
    )
}

#[test]
fn rdf_array_contrib_sparql_results_testsuite() -> Result<()> {
    check_testsuite(
        "https://rdf-array.github.io/rdf-tests-contrib/sparql/results/manifest.ttl",
        &[
            // This runner has no CSV results reader (CSV is write-only here, as for the W3C csv tests);
            // sparesults' tests compare its CSV output with this fixture
            "https://rdf-array.github.io/rdf-tests-contrib/sparql/results/manifest.ttl#results-csv",
        ],
    )
}
