# oxigraph-array: Oxigraph with sequences as terms

rdf-array is RDF whose data model has one more kind of term, the **sequence**: `(1 2 3)` in Turtle
is one term holding its elements in order, not a ladder of `rdf:first`/`rdf:rest` blank nodes, and
`()` is the empty sequence, not `rdf:nil`. Triples that spell out `rdf:first`/`rdf:rest` stay
ordinary triples and `rdf:nil` an ordinary IRI; they are just no longer what `( … )` means. See
[rdf-array.md](https://rdf-array.github.io/rdf-array/docs/rdf-array.html) for the model and the proposed
SPARQL and ShEx syntax.

This fork of [Oxigraph](https://github.com/oxigraph/oxigraph) is the Rust stack for that model: the
term, every syntax Oxigraph reads and writes, the SPARQL parser, optimizer and evaluator, the
results formats, the RocksDB and in-memory store, and the Python and JavaScript bindings. All the
crates live in this workspace, so there are no dependency forks to pin.

| crate | what it does here | upstream version |
|---|---|---|
| `oxrdf` | `Sequence`, `Term::Sequence`, `TermRef::Sequence`, and `Subject`/`SubjectRef` (an IRI, a blank node or a sequence) as the subject of `Triple` and `Quad`; `Dataset` interns and canonicalizes sequences; `Term::from_str` reads `( … )` | 0.3.5 |
| `oxttl` | Turtle, TriG and N3 collections are one term; N-Triples and N-Quads read and write `( … )`; N3's `N3Term::Sequence` may hold variables | 0.2.5 |
| `oxrdfxml` | `rdf:parseType="Collection"` is one term and is written back for a sequence of IRIs and blank nodes; `rdf:li` stays `rdf:li` | 0.2.5 |
| `oxjsonld` | `@list` is one term and a sequence is written as `@list` | 0.2.7 |
| `oxrdfio` | the format-independent parser and serializer carry sequences, N3 collections included | 0.2.7 |
| `sparesults` | `{"type": "seq", "value": [ … ]}`, `<seq>`, the CSV cell and the TSV spelling, read and written | 0.3.5 |
| `spargebra` | a collection in a pattern, template or update is one sequence pattern; the position patterns `((e))[i]`, `[?i]`, `[*]`; `isSEQ`, `SEQLEN`, `SEQ`, `x[i]` | 0.4.8 |
| `sparopt` | carries the patterns; type inference knows that a variable may hold a sequence | 0.3.9 |
| `spareval` | matches sequence and position patterns, evaluates the functions, orders sequences, builds them in CONSTRUCT and update templates | 0.2.9 |
| `oxigraph` | stores sequences in RocksDB and in memory; `quads_for_pattern` takes a sequence subject | 0.5.13 |
| `pyoxigraph` | `pyoxigraph.Sequence` | 0.5.13 |
| `oxigraph` (npm) | `oxigraph.sequence()`, `termType: "Sequence"` as in [types-array](https://github.com/rdf-array/types-array) | 0.5.13 |

The branch is `rdf-sequences`, from the tag v0.5.13. The other rdf-array implementations:
[jena-array](https://github.com/rdf-array/jena-array) (Java),
[comunica-array](https://github.com/rdf-array/comunica-array) and
[shex.js-array](https://github.com/rdf-array/shex.js-array) (JavaScript),
[rdflib-array](https://github.com/rdf-array/rdflib-array) and
[PyShEx-array](https://github.com/rdf-array/PyShEx-array) (Python).

## Setting up

```bash
git clone --recurse-submodules -b rdf-sequences git@github.com:rdf-array/oxigraph-array.git
cd oxigraph-array
cargo test --workspace --exclude oxigraph-cli   # RocksDB is built from the submodule
cargo test -p oxigraph-cli                        # see "Tests" for why it runs on its own
(cd python && uv venv -q .venv && uv pip install -q -p .venv maturin && .venv/bin/maturin develop && cd tests && ../.venv/bin/python -m unittest)
(cd js && npm install && npm test)
```

`--no-default-features` builds the store without RocksDB, in memory only.

## What a sequence does here

```rust
use oxrdf::{Literal, NamedNode, Sequence, Term, Triple};
let seq = Sequence::new([Term::from(Literal::from(1)), Sequence::empty().into()]);
assert_eq!(seq.to_string(), "(\"1\"^^<http://www.w3.org/2001/XMLSchema#integer> ())");
let t = Triple::new(seq.clone(), NamedNode::new("http://ex/p")?, seq); // a sequence may be a subject
```

```sparql
PREFIX : <http://example/>
SELECT ?x ?o WHERE { ?s :p (?x (2 3) ?o) }          # binds ?x and ?o element by element
SELECT ?s WHERE { ?s :p () }                          # the empty sequence is a term
SELECT ?e WHERE { ?s :p ((?e))[2] }                   # the 2nd element
SELECT ?i ?e WHERE { ?s :p ((?e))[?i] }               # each element and its 1-based position
SELECT ?e WHERE { ?s :p ((((?e))[1]))[2] }            # nested: the 1st element of the 2nd
SELECT ?n WHERE { ?s :p ?q FILTER(isSEQ(?q)) BIND(SEQLEN(?q) AS ?n) BIND(?q[2] AS ?second) }
CONSTRUCT { ?s :r (?x [ :k ?x ]) } WHERE { ?s :p (?x 2) }
DELETE { ?s :tags ?seq } INSERT { ?s :tag ?t } WHERE { ?s :tags ?seq . ?s :tags ((?t))[*] }
SELECT * WHERE { ?s ?p ?o } ORDER BY ?o               # sequences last, element by element
```

- **Data.** `Term::Sequence(Sequence)` and `TermRef::Sequence(&Sequence)`; a sequence holds any
  terms, sequences and triple terms included. `Triple::subject` and `Quad::subject` are a `Subject`
  (`NamedNode`, `BlankNode` or `Sequence`), which replaces the deprecated `Subject` alias of
  `NamedOrBlankNode`; `NamedOrBlankNode` stays for graph names and RDF 1.2 reifiers, which cannot be
  sequences. Equality and hashing are by elements. `Dataset` canonicalization reaches blank nodes
  inside sequences (in subject, object and triple-term positions).
- **Syntaxes.** Turtle, TriG and N3 collections are one term wherever a subject or an object may
  stand, blank node property lists inside them still give their triples, and `()` is the empty
  sequence. N-Triples and N-Quads spell a sequence `(e1 e2 …)` on one line and refuse it as a
  predicate or a graph name. The Turtle writer writes `( e1 e2 )`. RDF/XML reads
  `rdf:parseType="Collection"` as a sequence and writes one back when every element is an IRI or a
  blank node; a sequence of literals or sequences, or a sequence subject, is an error there. JSON-LD
  reads `@list` (nested, empty, through `"@container": "@list"`) as a sequence and writes `@list`; a
  sequence subject is an error. `rdf:li` in RDF/XML stays `rdf:li` and is no longer numbered into
  `rdf:_n`.
- **Store.** `EncodedTerm::Sequence` holds the encoded elements; the binary form is the type byte
  52 (in the block of structured terms beside triple terms), a big-endian `u32` count and the
  elements. Strings inside sequences are stored as any term's are. A database written by this fork
  cannot be opened by upstream Oxigraph if it holds sequences.
- **Matching.** A sequence pattern whose elements are all constants is a constant and uses the
  indexes. One with variables, blank nodes or position patterns inside is matched like a triple term
  holding variables: the quads are fetched with that position open, then each candidate is taken
  apart element by element, and a repeated variable must match equal elements. A position pattern
  with `?i` or `*` gives one solution per matching position. In a property path's end points, a
  position pattern that matches several positions is an error (`UnsupportedSequencePattern`).
- **Expressions.** `isSEQ(x)`; `SEQLEN(x)` (an `xsd:integer`); `SEQ(e1, …)` builds a sequence;
  `x[i]` is the element at the 1-based position `i`, an error out of range. `=` compares sequences
  element by element with `=`'s own rules. `ORDER BY` puts sequences after literals and triple terms
  and compares them element by element, a prefix first.
- **Results.** JSON `{"type": "seq", "value": [ …one term object per element… ]}`, XML `<seq>` with
  one child per element, CSV one cell with literals quoted (`("1" "2")`, CSV-escaped once), TSV
  `(e1 e2 …)` in the TSV spellings; JSON, XML and TSV are read back.
- **Python.** `pyoxigraph.Sequence(elements)` is a frozen Python sequence of terms (`len`,
  indexing, iteration, `elements`, `str` in the N-Triples-array spelling, pickling); `Triple`,
  `Quad` and `Store.quads_for_pattern` take it as a subject.
- **JavaScript.** `oxigraph.sequence(elements)` returns a `Sequence` with `termType: "Sequence"`,
  `elements` and `value` (the N-Triples-array spelling), as types-array defines it; `fromTerm` reads
  plain `{termType: "Sequence", elements}` objects, and `Store.match` takes a sequence subject.

## Tests

- **Added**, each crate's `tests/sequences.rs` (oxttl 10, oxrdfxml 5, oxjsonld 4, sparesults 4,
  spargebra 6, spareval 5, oxigraph 4) and `lib/oxrdf/src/sequence.rs` (3); `python/tests`
  (`TestSequence`, `TestStoreSequences`); `js/test/sequence.test.ts`. sparesults' tests compare its
  four formats with [rdf-tests-contrib](https://github.com/rdf-array/rdf-tests-contrib)'s
  `sparql/results` fixtures.
- **Suites.** The `testsuite` submodules point at [rdf-tests-array](https://github.com/rdf-array/rdf-tests-array),
  [json-ld-api-array](https://github.com/rdf-array/json-ld-api-array) and
  [json-ld-streaming-array](https://github.com/rdf-array/json-ld-streaming-array) (branch
  `rdf-sequences`), and two new ones hold [rdf-tests-contrib](https://github.com/rdf-array/rdf-tests-contrib)
  and [json-ld-api-contrib](https://github.com/rdf-array/json-ld-api-contrib); `testsuite/src/files.rs`
  maps `https://rdf-array.github.io/` onto them. The manifest reader takes `mf:entries ( … )` and
  `mf:include ( … )` as the sequences they now are. New suite functions: `rdf_array_contrib_*` in
  `tests/parser.rs` (Turtle, TriG, N-Triples, N-Quads, JSON-LD toRdf and fromRdf) and
  `tests/sparql.rs` (evaluation, update, syntax, results). Every suite passes: parser 25, SPARQL 12,
  canonicalization 1, Oxigraph's own 9, serd 3.
- **Flagged** (skipped with the reason in the test list):
  - w3c/N3, which has no -array fork: 14 TurtleTests whose expected results spell collections as
    ladders (the same tests pass, re-recorded, in `rdf11_turtle_w3c_testsuite`) and
    `cwm_syntax_path2.n3`.
  - Suite drift, not sequences: rdf-tests-array is 73 commits newer than the rdf-tests Oxigraph
    v0.5.13 pins, and its SPARQL, N-Triples and RDF/XML directories carry no rdf-array change. New
    or changed upstream since: `ntriples-version-1`; `xml-canon-test001`/`002` and
    `rdf12-xml-an-13`/`14` (XML literal serialization); SPARQL 1.0 `graph-nested-2`/`3`,
    `dawg-dataset-11`/`12b`, `case-insensitive-booleans`; SPARQL 1.1 `graph-no-var`,
    `graph-from-named-1`/`2`/`3`, `values-construct-group-by`, `subquery15`/`16`, `csv04`; SPARQL
    1.2 `select-variable-reuse`, `codepoint-esc-01-bad`/`03-bad`/`08`/`09`.
  - `results-csv` in rdf-tests-contrib: this runner reads no CSV results (as for the W3C csv tests).
- **Re-recorded**: the four serd fixtures that spelled collections as ladders
  (`test-a-without-whitespace`, `test-blank-in-list`, `test-list-in-blank`, `test-list-subject`),
  folded with rdf-array's `tools/rdfarray.py` and checked against jena-array's reading of the input.
- **Numbers**: `cargo test --workspace --all-targets --exclude oxigraph-cli` 281 passed; doctests
  314 passed; `cargo clippy --workspace --all-targets` clean; pyoxigraph 212; JavaScript 53. The CLI's
  64 tests pass when run alone; run with the rest of the workspace, some time out, because each
  spawns `cargo run` and waits on cargo's locks and on macOS starting the fresh binary.
- **Contrib fixes made on the way**: rdf-tests-contrib's JSON results fixture lost comunica's
  `"metadata"` member, which SPARQL Results JSON does not define; json-ld-api-contrib's fromRdf
  `seq04` input names its blank node `_:b0`, as the expected output does.

## Decisions

- **The subject type.** Oxigraph 0.5 made a triple's subject `NamedOrBlankNode`. A sequence may be a
  subject in rdf-array, so `Subject` became an enum of its own rather than a third variant of
  `NamedOrBlankNode`, which also names graphs and reifiers.
- **Not behind a feature.** Sequences are always on, unlike RDF 1.2's `rdf-12` feature: they are
  the point of the fork and change what the Turtle parser produces.
- **Position patterns in the algebra.** `TermPattern::SequencePosition` and
  `GroundTermPattern::SequencePosition` carry `((e))[i]` to the evaluator rather than rewriting it
  into filters, because `[?i]` and `[*]` give several solutions per quad. The parser refuses them in
  CONSTRUCT and update templates.
- **N3.** `N3Term::Sequence(Vec<N3Term>)` rather than `Sequence`, so that elements may be variables.
- **Spelling.** `Display` gives `(e1 e2)` with no inner spaces (N-Triples-array, as rdf-array's
  tools write it); the Turtle and SPARQL writers give `( e1 e2 )`.

## Not done

- A sequence inside a Turtle or N-Triples triple term (`<<( ( 1 ) :p :o )>>`) is not parsed, though
  the model, the store, N-Triples' writer and SPARQL hold it (rdf-array issue #5).
- Sequences as subjects of `DELETE DATA` (`GroundQuad` keeps an IRI subject).
- The GeoSPARQL functions and the HTTP server's HTML pages know nothing of sequences beyond printing
  them.
- No GitHub Pages site; the Python stubs (`generate_stubs.py`) were not regenerated.
