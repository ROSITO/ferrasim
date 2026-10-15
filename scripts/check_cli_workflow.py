"""Run the native CLI and independent Rust API example from a fresh working directory."""
from pathlib import Path
import subprocess
import sys
import tempfile
from check_result_exports import check

ROOT=Path(__file__).resolve().parents[1]
suffix='.exe' if sys.platform=='win32' else ''
cli=ROOT/'target/debug'/('ferrasim'+suffix)
api=ROOT/'target/debug/examples'/('run_simulation'+suffix)
with tempfile.TemporaryDirectory(prefix='ferrasim S08 é ') as name:
    root=Path(name);inputs=root/'input model';inputs.mkdir();working=root/'other cwd';working.mkdir()
    (inputs/'model.json').write_bytes((ROOT/'examples/uniaxial/model.json').read_bytes())
    (inputs/'mesh.msh').write_bytes((ROOT/'examples/uniaxial/mesh.msh').read_bytes())
    model=Path('..')/'input model/model.json'
    def run(binary,args):
        result=subprocess.run([str(binary),*map(str,args)],cwd=working,capture_output=True,text=True,encoding='utf-8',timeout=30)
        assert result.returncode==0,(result.returncode,result.stderr)
        return result
    run(cli,['validate',model]);assert not (working/'cli results').exists()
    run(cli,['solve',model,'--output','cli results'])
    run(api,[model,'api results'])
    for file in (working/'cli results').iterdir():assert file.read_bytes()==(working/'api results'/file.name).read_bytes(),file.name
    report=check(working/'cli results')
    assert report['solver']['status']=='Converged'
    assert report['balance']['relative_force_error']<1e-6
    assert report['balance']['relative_moment_error']<1e-6
print('CLI/API workflow agrees from a fresh Unicode working directory, including all field files and manifest.')
