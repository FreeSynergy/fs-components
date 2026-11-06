import json
def spec(rows, enc, mark="point", **kw):
    s = {"width": 300, "height": 200, "data": {"values": rows}, "mark": mark, "encoding": enc}; s.update(kw); return s
Q=lambda f,**k: dict({"field":f,"type":"quantitative"},**k)
cases = {
 "l01_small": [(0.1,0.3),(0.9,0.5)],
 "l02_unit": [(1,1),(2,2)],
 "l03_tiny": [(0.01,0.02),(0.09,0.06)],
 "l04_big": [(1e15,1e15),(2e15,3e15)],
 "l05_big_frac": [(1e15,1e15+0.125),(2e15,1e15)],
 "l06_thousands": [(1000,12345),(2500000,30000)],
 "l07_neg": [(-5,-0.5),(5,0.5)],
 "l08_dec5": [(0.0005,0.0015),(0.0025,0.0035)],
 "l09_1e21": [(1e21,1e22),(3e21,5e22)],
 "l10_1e7": [(1e-7,2e-7),(5e-7,9e-7)],
 "l11_halves": [(0,0),(2.5,2.5)],
}
S={}
for n,pts in cases.items():
    S[n]=spec([{"x":a,"y":b} for a,b in pts],{"x":Q("x"),"y":Q("y")})
json.dump(S,open("labels_specs.json","w"),indent=1)
