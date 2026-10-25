import assert from "node:assert";
import { webcrypto } from "node:crypto";
import { describe, it, vi } from "vitest";
import oxigraph from "../pkg/oxigraph.js";

vi.stubGlobal("crypto", webcrypto);

// rdf-array: the sequence term
describe("Sequence", () => {
    const a = oxigraph.namedNode("http://example.com/a");
    const p = oxigraph.namedNode("http://example.com/p");

    it("is a term with elements", () => {
        const seq = oxigraph.sequence([oxigraph.literal("1"), a, oxigraph.sequence([])]);
        assert.strictEqual(seq.termType, "Sequence");
        assert.strictEqual(seq.elements.length, 3);
        assert.strictEqual(seq.value, '("1" <http://example.com/a> ())');
        assert.strictEqual(seq.toString(), seq.value);
        assert(seq.elements[1].equals(a));
        assert(seq.equals(oxigraph.sequence([oxigraph.literal("1"), a, oxigraph.sequence([])])));
        assert(!seq.equals(oxigraph.sequence([])));
    });

    it("is read from plain RDF/JS-array objects", () => {
        const term = oxigraph.fromTerm({ termType: "Sequence", value: "", elements: [a] });
        assert(term.equals(oxigraph.sequence([a])));
    });

    it("stands as subject and object", () => {
        const seq = oxigraph.sequence([a]);
        const quad = oxigraph.quad(seq, p, oxigraph.sequence([]));
        assert(quad.subject.equals(seq));
        assert.strictEqual(quad.toString(), "(<http://example.com/a>) <http://example.com/p> ()");
    });

    it("is parsed, stored and queried", () => {
        const store = new oxigraph.Store();
        store.load("@prefix : <http://example.com/> . :s :p (1 (2) ()) . (:a) :p :o .", {
            format: "text/turtle",
        });
        assert.strictEqual(store.size, 2);
        assert.strictEqual(store.match(oxigraph.sequence([a]), null, null, null).length, 1);
        const results = store.query("SELECT ?e WHERE { ?s <http://example.com/p> ((?e))[*] }");
        assert.strictEqual(results.length, 3);
        const rows = store.query("SELECT ?seq WHERE { <http://example.com/s> ?p ?seq }");
        assert.strictEqual(rows[0].get("seq").termType, "Sequence");
    });
});
