#!/usr/bin/env python3
"""Deterministic TET4 cantilever convergence study; Python standard library only."""
import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import sys
import subprocess
import time


def generate(root, m, rtol=1e-10):
    nx, ny, nz = 20*m, m, m
    root.mkdir(parents=True)
    def nid(i, j, k):
        return 1 + (i*(ny+1)+j)*(nz+1)+k
    nodes = [(nid(i,j,k), i/nx, .05*j/ny, .05*k/nz)
             for i in range(nx+1) for j in range(ny+1) for k in range(nz+1)]
    elements = []
    for i in range(nx):
        for j in range(ny):
            for k in range(nz):
                c = [nid(i+a,j+b,k+d) for a,b,d in
                     [(0,0,0),(1,0,0),(1,1,0),(0,1,0),(0,0,1),(1,0,1),(1,1,1),(0,1,1)]]
                for t in [(0,1,2,6),(0,2,3,6),(0,3,7,6),(0,7,4,6),(0,4,5,6),(0,5,1,6)]:
                    elements.append((4,3,[c[a] for a in t]))
    tip = []
    for i, tag in [(0,1),(nx,2)]:
        for j in range(ny):
            for k in range(nz):
                a,b,c,d = nid(i,j,k),nid(i,j+1,k),nid(i,j+1,k+1),nid(i,j,k+1)
                for tri in [(a,b,c),(a,c,d)]:
                    elements.append((2,tag,tri))
                    if tag == 2:
                        tip.append(tri)
    lines = ['$MeshFormat','2.2 0 8','$EndMeshFormat','$PhysicalNames','3',
             '2 1 "clamp"','2 2 "tip"','3 3 "solid"','$EndPhysicalNames',
             '$Nodes',str(len(nodes))]
    lines += [' '.join([str(n),*(format(v,'.17g') for v in xyz)]) for n,*xyz in nodes]
    lines += ['$EndNodes','$Elements',str(len(elements))]
    lines += [f'{idx} {kind} 2 {tag} {tag} '+ ' '.join(map(str,ns))
              for idx,(kind,tag,ns) in enumerate(elements,1)]
    lines += ['$EndElements']
    (root/'mesh.msh').write_text('\n'.join(lines)+'\n',encoding='utf-8',newline='\n')
    model = json.loads((Path(__file__).resolve().parents[1]/'examples/uniaxial/model.json').read_text())
    model['boundary_conditions'] = [{'node_group':'clamp','components':dict(x=0.,y=0.,z=0.)}]
    model['loads'] = [{'type':'surface_traction','surface_group':'tip','components':dict(x=0.,y=0.,z=-40000.)}]
    model['mesh']['groups'] = dict(nodes=['clamp'],surfaces=['clamp','tip'],volumes=['solid'])
    model['materials'][0].update(young=210e9,poisson=.3)
    model['solver_settings'].update(rtol=rtol,atol=1e-12,max_iterations=30000)
    model['outputs']['formats'] = ['json']
    (root/'model.json').write_text(json.dumps(model,indent=2)+'\n',encoding='utf-8',newline='\n')
    return nodes, tip


def execute(binary, model, output, log):
    start = time.perf_counter()
    with log.open('w') as stream:
        proc = subprocess.Popen([str(binary),'solve',str(model),'--output',str(output),
                                 '--force-scale','100','--length-scale','1'],stdout=stream,stderr=stream)
        rss = None
        if hasattr(os,'wait4'):
            _, status, usage = os.wait4(proc.pid,0)
            proc.returncode = os.waitstatus_to_exitcode(status)
            rss = usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024)
        else:
            proc.wait()
    if proc.returncode:
        raise RuntimeError(f'CLI failed ({proc.returncode}); see {log}')
    return time.perf_counter()-start, rss


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary',type=Path,default=Path('target/release/ferrasim'+('.exe' if os.name=='nt' else '')))
    parser.add_argument('--output',type=Path,required=True,help='new study directory')
    parser.add_argument('--protocol',choices=['v1','v2'],default='v2')
    parser.add_argument('--generate-only',action='store_true',help='generate inputs without invoking Rust or claiming validation')
    parser.add_argument('--diagnostic-rtol',type=float,help='diagnostic only: override protocol rtol, never qualifies S09')
    parser.add_argument('--levels',type=int,nargs='+')
    args = parser.parse_args()
    if args.levels is None:
        args.levels=[2,4,8] if args.protocol=='v1' else [2,4,8,12]
    primary_rtol=1e-10 if args.protocol=='v1' else 1e-7
    tight_rtol=1e-12 if args.protocol=='v1' else 1e-8
    if args.protocol=='v2' and args.diagnostic_rtol is None and not args.generate_only and args.levels!=[2,4,8,12]:
        parser.error('adopted v2 fixes levels to 2 4 8 12')
    if len(args.levels)<3 or any(m<1 for m in args.levels) or any(b<=a for a,b in zip(args.levels,args.levels[1:])):
        parser.error('require at least three strictly increasing positive levels')
    if args.diagnostic_rtol is not None and (not math.isfinite(args.diagnostic_rtol) or not 0<args.diagnostic_rtol<=1e-8):
        parser.error('diagnostic rtol must be finite, positive and <=1e-8')
    root=args.output.resolve(); root.mkdir(parents=True,exist_ok=False)
    if args.generate_only:
        for m in args.levels:
            generate(root/f'm{m}',m,rtol=args.diagnostic_rtol if args.diagnostic_rtol is not None else primary_rtol)
        print('Inputs generated; no simulation or qualification performed.')
        return
    binary=args.binary.resolve()
    reference=-100/(3*210e9*(.05*.05**3/12))
    summary=dict(protocol='S09-'+args.protocol,environment=platform.platform(),machine=platform.machine(),
                 driver_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                 python=sys.version, rustc=subprocess.check_output(['rustc','--version'],text=True).strip(),
                 source_revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
                 cpu_count=os.cpu_count(),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                 qualification='suspended',reference_m=reference,diagnostic_only=args.diagnostic_rtol is not None,runs=[])
    (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    for m in args.levels:
        nodes,tip=generate(root/f'm{m}',m,rtol=args.diagnostic_rtol if args.diagnostic_rtol is not None else primary_rtol)
        try:
            elapsed,rss=execute(binary,root/f'm{m}/model.json',root/f'm{m}/results',root/f'm{m}/run.log')
        except RuntimeError:
            summary['failure']=dict(m=m,diagnostic=(root/f'm{m}/run.log').read_text())
            (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
            raise
        r=json.loads((root/f'm{m}/results/report.json').read_text())
        values={n['id']:n['displacement'][2] for n in r['nodes']}
        # Equal triangle areas for this structured square; integral of linear u is A*sum(u)/3.
        displacement=sum(sum(values[n] for n in tri)/3 for tri in tip)/len(tip)
        if not math.isfinite(displacement) or displacement>=0:
            raise RuntimeError('Nonfinite or wrong-sign tip displacement')
        if summary['runs'] and abs(displacement/reference-1)>=summary['runs'][-1]['relative_error']:
            raise RuntimeError('Refinement does not improve the displacement error')
        row=dict(m=m,nodes=len(nodes),degrees_of_freedom=3*len(nodes),tetrahedra=120*m**3,tip_displacement_m=displacement,
                 relative_error=abs(displacement/reference-1),elapsed_seconds=elapsed,peak_rss_bytes=rss,
                 solver=r['solver'],balance=r['balance'],
                 input_sha256={p:hashlib.sha256((root/f'm{m}'/p).read_bytes()).hexdigest() for p in ['model.json','mesh.msh']})
        summary['runs'].append(row)
        (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
        print(f'm={m}: u={displacement:.12g} m; error={row["relative_error"]:.4%}; {elapsed:.2f}s',flush=True)
        for metric in ['relative_force_error','relative_moment_error','relative_free_residual','relative_energy_error']:
            if not math.isfinite(r['balance'][metric]) or r['balance'][metric]>1e-6:
                raise RuntimeError(f'Balance failed: {metric}')
    if args.diagnostic_rtol is not None:
        print('Diagnostic only; S09 acceptance suspended.',flush=True)
        return
    errors=[r['relative_error'] for r in summary['runs']]
    if any(not math.isfinite(e) for e in errors) or len(errors)<3 or any(b>=a for a,b in zip(errors,errors[1:])) or errors[-1]>.05:
        raise RuntimeError('Convergence gate failed: require >=3 improving levels and final error <=5%')
    m=args.levels[-1]
    _,tip=generate(root/'tight',m,rtol=tight_rtol)
    elapsed,rss=execute(binary,root/'tight/model.json',root/'tight/results',root/'tight/run.log')
    r=json.loads((root/'tight/results/report.json').read_text())
    values={n['id']:n['displacement'][2] for n in r['nodes']}
    tight=sum(sum(values[n] for n in tri)/3 for tri in tip)/len(tip)
    variation=abs(tight/summary['runs'][-1]['tip_displacement_m']-1)
    primary_values={n['id']:n['displacement'] for n in json.loads((root/f'm{m}/results/report.json').read_text())['nodes']}
    field_difference=math.hypot(*(v-primary_values[n['id']][i] for n in r['nodes'] for i,v in enumerate(n['displacement'])))
    field_norm=math.hypot(*(v for u in primary_values.values() for v in u))
    field_variation=field_difference/field_norm
    energy_variation=abs(r['balance']['strain_energy']/summary['runs'][-1]['balance']['strain_energy']-1)
    summary['algebraic_check']=dict(rtol=tight_rtol,relative_field_variation=field_variation,relative_energy_variation=energy_variation,tip_displacement_m=tight,relative_variation=variation,
                                    solver=r['solver'],balance=r['balance'],elapsed_seconds=elapsed,peak_rss_bytes=rss,
                                    input_sha256={p:hashlib.sha256((root/'tight'/p).read_bytes()).hexdigest() for p in ['model.json','mesh.msh']})
    (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    for metric in ['relative_force_error','relative_moment_error','relative_free_residual','relative_energy_error']:
        if not math.isfinite(r['balance'][metric]) or r['balance'][metric]>1e-6:
            raise RuntimeError(f'Tight balance failed: {metric}')
    if any(not math.isfinite(v) or v>1e-6 for v in [variation,field_variation,energy_variation]):
        raise RuntimeError('Algebraic tolerance check failed')
    summary['qualification']='numerically_verified_pending_independent_review'
    (root/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(f'Convergence gate passed for {args.protocol}; status: {summary["qualification"]}.')

if __name__=='__main__':
    main()
