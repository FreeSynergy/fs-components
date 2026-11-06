Golden values: mark geometry (as fractions of the plot area) and colour domains of real Vega-Lite
6.5.0 (headless, `vega` 6, no rendering) for small specs, one file per case. Made once with
`.scratch/t1-charts/oracle/{gen.py,oracle.mjs,compact.py}`; `cargo test` needs no node. We analyse
what Vega-Lite does and write our own code (ADR 0013); nothing of Vega-Lite is copied.

Tools: `tools/gen.py` and `tools/labels_gen.py` write the specs, `tools/oracle.mjs` runs them in
Vega-Lite (`npm install vega@6 vega-lite@6.5.0` in a scratch directory), `tools/compact.py` writes
the small files here.
