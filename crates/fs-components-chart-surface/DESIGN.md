# fs-components-chart-surface - Design

A screen of the window shell (ADR 0065) that shows a chart. A program gives data and the trait `ChartSource` (a Vega-Lite spec as text, the subset `fs-components-charts` validates); this crate draws it. No render code and no look in the program repo (ADR 0066).

| Where | Pattern | Why |
|---|---|---|
| `ChartSource` | Strategy | the program decides the data and the spec, the screen only draws |
| `ChartSurface` | Adapter | `ChartSource` + `Labels` to the shell's `Surface` |

State: done, tested offscreen (drawn pixels, new data at the next build, an unusable spec gives a notice with the reason, never a panic).

Unsupported: no click/hover on marks (the chart widget has none); no refresh signal, the spec is read each time the screen is built (the shell builds again when the screen is shown or resized).
