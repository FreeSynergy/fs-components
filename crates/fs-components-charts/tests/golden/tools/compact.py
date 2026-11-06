import json, os, sys
specs = json.load(open(sys.argv[2] if len(sys.argv) > 2 else 'specs.json')); out = json.load(open(sys.argv[3] if len(sys.argv) > 3 else 'out.json'))
dest = sys.argv[1]; os.makedirs(dest, exist_ok=True)
for name, spec in specs.items():
    o = out[name]
    if 'error' in o: print('skip', name, o['error'][:80]); continue
    W, H = o['width'], o['height']
    marks = []
    for m in o['marks']:
        items = []
        for i in m['items']:
            it = {}
            if m['type'] in ('rect',): it = dict(x=i['x']/W, y=i['y']/H, w=i['w']/W, h=i['h']/H)
            elif m['type'] == 'arc': it = dict(a0=min(i['a0'], i['a1']), a1=max(i['a0'], i['a1']))
            elif m['type'] == 'area': it = dict(x=i['x']/W, y=i['y']/H, y2=(i['y']+i['h'])/H)
            else: it = dict(x=i['x']/W, y=i['y']/H)
            for k in ('opacity','fillOpacity','dash'):
                if i.get(k) is not None: it[k] = i[k]
            items.append(it)
        marks.append({'type': m['type'], 'items': items})
    g = {'note': 'Generated with Vega-Lite %s (headless, vega 6) by scratch/oracle/oracle.mjs; fractions of the plot area' % o['vl'],
         'spec': spec, 'colour_domain': o['domains'].get('color'), 'marks': marks, 'axis_labels': [a['texts'] for a in o.get('axes', []) if a['texts']]}
    json.dump(g, open(os.path.join(dest, name + '.json'), 'w'), indent=None, separators=(',', ':'))
print(len(os.listdir(dest)), 'goldens')
