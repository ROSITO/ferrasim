"""Compare exported S07 fields to the analytic uniaxial fixture, without Rust/VTK."""
from fractions import Fraction
import json
import math
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[1]
reference=json.loads((ROOT/'validation/fixtures/uniaxial-reference.json').read_text(encoding='utf-8'))
report=json.loads((Path(sys.argv[1])/'report.json').read_text(encoding='utf-8'))
expected={i:[float(Fraction(x)) for x in u] for i,u in zip(reference['node_ids'],reference['displacements'])}
def error(actual,expected,scale):
    return math.hypot(*(a-b for a,b in zip(actual,expected)))/max(math.hypot(*expected),scale)
errors=[error(n['displacement'],expected[n['id']],1e-3) for n in report['nodes']]
for element in report['elements']:
    errors.append(error(element['stress'],[10,0,0,0,0,0],1))
    errors.append(error(element['strain'],[0.01,-0.0025,-0.0025,0,0,0],1e-3))
    errors.append(abs(element['von_mises']-10)/10)
maximum=max(errors)
assert maximum<=1e-10,maximum
print(f'Analytic uniaxial result-field error: {maximum:.17e} (limit 1e-10).')
