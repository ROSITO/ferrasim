"""Audit b-Ax with 100-digit Decimal arithmetic, independently of Rust summation.

Input: JSON from the audit_residual Rust example (a diagnostic, not a result bundle).
No solve or conversion of the original system is performed by this script.
"""
import argparse
import decimal
import hashlib
import json
import math
import os
import subprocess
from pathlib import Path


def audit(path):
    data=json.loads(path.read_text(encoding='utf-8'))
    with decimal.localcontext() as context:
        context.prec=100
        convert=decimal.Decimal.from_float
        residual=[convert(float(b)) for b in data['rhs']]
        naive=[0.]*len(residual)
        for i,j,a in data['entries']:
            residual[i]-=convert(a)*convert(data['x'][j])
            naive[i]+=a*data['x'][j]
        reference=sum(r*r for r in residual).sqrt()
        row=max(range(len(residual)),key=lambda i:abs(residual[i]))
        # Sensitivity scale, NOT a rigorous impossibility bound or condition number.
        ulp_scale=math.hypot(*(abs(a)*math.ulp(data['x'][j])
                              for i,j,a in data['entries'] if i==row))
        return dict(precision_decimal_digits=100,system_sha256=hashlib.sha256(path.read_bytes()).hexdigest(),
                    size=len(residual),reference_residual_norm=str(reference),
                    compensated_residual_norm=data['residual_norm'],
                    naive_residual_norm=math.hypot(*(b-v for b,v in zip(data['rhs'],naive))),
                    worst_row=row,worst_row_residual=str(residual[row]),
                    worst_row_one_ulp_product_scale=ulp_scale)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input',type=Path)
    parser.add_argument('--run-model',type=Path,help='run native audit example and save its system to input')
    parser.add_argument('--binary',type=Path,default=Path('target/release/examples/audit_residual'+('.exe' if os.name=='nt' else '')))
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    if args.run_model is not None:
        with args.input.open('xb') as stream:
            subprocess.run([str(args.binary.resolve()),str(args.run_model.resolve())],stdout=stream,check=True)
    result=audit(args.input)
    reference=float(result['reference_residual_norm'])
    mismatch=abs(result['compensated_residual_norm']-reference)/max(reference,1e-300)
    result['relative_norm_disagreement']=mismatch
    print(json.dumps(result,indent=2))
    if args.check and (not math.isfinite(mismatch) or mismatch>1e-6):
        raise SystemExit('Compensated residual disagrees with 100-digit reference')
