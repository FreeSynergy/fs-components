// Runs Vega-Lite specs (headless, no rendering) and prints mark geometry as fractions of the
// plot area plus the scale domains. Usage: node oracle.mjs specs.json out.json
import * as vega from 'vega';
import * as vl from 'vega-lite';
import fs from 'fs';

const [,, inFile, outFile] = process.argv;
const specs = JSON.parse(fs.readFileSync(inFile, 'utf8'));
const out = {};
function walk(node, f) { f(node); for (const it of node.items || []) if (it && it.items) walk(it, f); }
for (const [name, spec] of Object.entries(specs)) {
  try {
    const vg = vl.compile(spec).spec;
    const view = new vega.View(vega.parse(vg), { renderer: 'none' });
    await view.runAsync();
    const W = spec.width ?? 300, H = spec.height ?? 200;
    const marks = [];
    walk(view.scenegraph().root, (n) => {
      if (n.marktype && n.role === 'mark') {
        const items = (n.items || []).map((i) => ({
          x: i.x, y: i.y, x2: i.x2, y2: i.y2, w: i.width, h: i.height,
          a0: i.startAngle, a1: i.endAngle, r0: i.innerRadius, r1: i.outerRadius,
          fill: i.fill, stroke: i.stroke, opacity: i.opacity, fillOpacity: i.fillOpacity, dash: i.strokeDash,
          datum: i.datum && Object.fromEntries(Object.entries(i.datum).filter(([k]) => !k.startsWith('_') && k !== 'Symbol(vega_id)')),
        }));
        marks.push({ type: n.marktype, items });
      }
    });
    const axes = [];
    walk(view.scenegraph().root, (n) => {
      if (n.role === 'axis') {
        const g = n.items && n.items[0];
        if (!g) return;
        let texts = [];
        for (const m of g.items || []) if (m.role === 'axis-label') texts = (m.items || []).map((i) => i.text);
        axes.push({ orient: (n.items[0].datum && n.items[0].datum.orient) || null, texts });
      }
    });
    const dom = {};
    for (const s of ['x', 'y', 'color', 'theta']) { try { dom[s] = view.scale(s).domain(); } catch (e) {} }
    out[name] = { vl: vl.version, width: W, height: H, domains: dom, marks, axes };
  } catch (e) { out[name] = { error: String(e.message || e) }; }
}
fs.writeFileSync(outFile, JSON.stringify(out, null, 1));
