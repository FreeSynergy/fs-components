"""Writers of tokens.json (one literal per line, stable order) and the summary. Pattern: Template method."""
from __future__ import annotations

import json


def line(obj):
    return json.dumps(obj, sort_keys=True, ensure_ascii=False, separators=(", ", ": "))


def dump(data):
    out = ["{"]
    keys = sorted(data)
    for i, key in enumerate(keys):
        tail = "," if i < len(keys) - 1 else ""
        value = data[key]
        if isinstance(value, list):
            out.append(f' "{key}": [')
            out.extend(f"  {line(item)}{',' if n < len(value) - 1 else ''}" for n, item in enumerate(value))
            out.append(f" ]{tail}")
        elif key == "scales":
            out.append(f' "{key}": {{')
            for n, (kind, scale) in enumerate(value.items()):
                out.append(f'  "{kind}": {line(scale)}{"," if n < len(value) - 1 else ""}')
            out.append(f" }}{tail}")
        else:
            out.append(f' "{key}": {line(value)}{tail}')
    out.append("}")
    return "\n".join(out) + "\n"


def summary(data):
    rows = [f"counted {data['counted']}"] + [f"new tokens ({k}): {v}" for k, v in data["new_count"].items()]
    for kind, scale in data["scales"].items():
        rows.append(f"{kind}: base {scale['base']} new {[(n['token'], n['value']) for n in scale['new']]}")
    rows.append(f"new colours: {data['new_colors']}")
    return "\n".join(rows)
