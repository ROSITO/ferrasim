"""Built export example must not publish a bundle after invalid input or CG failure."""
from pathlib import Path
import json
import subprocess
import sys
import tempfile

ROOT=Path(__file__).resolve().parents[1]
BINARY=ROOT/'target/debug/examples'/('export_model.exe' if sys.platform=='win32' else 'export_model')
with tempfile.TemporaryDirectory(prefix='ferrasim S07 é ') as name:
    directory=Path(name)
    (directory/'mesh.msh').write_bytes((ROOT/'examples/uniaxial/mesh.msh').read_bytes())
    original=json.loads((ROOT/'examples/uniaxial/model.json').read_text(encoding='utf-8'))
    for scenario in ['invalid','limit','existing']:
        model=json.loads(json.dumps(original))
        if scenario=='invalid': model['material_assignments']=[]
        if scenario=='limit': model['solver_settings']['max_iterations']=1
        model_path=directory/'model.json';model_path.write_text(json.dumps(model),encoding='utf-8')
        output=directory/scenario
        if scenario=='existing':output.mkdir();(output/'sentinel').write_text('keep',encoding='utf-8')
        result=subprocess.run([str(BINARY),str(model_path),str(output),'10','2'],capture_output=True,text=True,encoding="utf-8",timeout=30)
        assert result.returncode!=0 and not result.stdout,(scenario,result)
        if scenario=='existing':assert list(output.iterdir())==[output/'sentinel'] and (output/'sentinel').read_text()=='keep'
        else: assert not output.exists(),scenario
print('Export failures publish neither completion nor result files; existing data is preserved.')
