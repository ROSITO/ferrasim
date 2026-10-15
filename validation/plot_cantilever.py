"""Render archived cantilever metrics (optional matplotlib dependency)."""
import argparse
import json
from pathlib import Path
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('summary',type=Path)
parser.add_argument('output',type=Path)
args=parser.parse_args()
data=json.loads(args.summary.read_text())
runs=data['runs']
fig,axes=plt.subplots(1,2,figsize=(10,4.4),layout='constrained')
x=[r['tetrahedra'] for r in runs]
axes[0].semilogx(x,[-r['tip_displacement_m']*1000 for r in runs],'o-',label='TET4, diagnostic CG')
axes[0].axhline(-data['reference_m']*1000,color='#555555',ls='--',label='Euler–Bernoulli (approx.)')
axes[0].set(ylabel='Amplitude de flèche moyenne (mm)',xlabel='Nombre de TET4')
axes[0].legend(fontsize=8)
axes[1].loglog(x,[r['relative_error']*100 for r in runs],'o-',color='#a34b21')
axes[1].axhline(5,color='#555555',ls='--',label='Seuil physique 5 %')
axes[1].set(ylabel='Écart relatif à Euler–Bernoulli (%)',xlabel='Nombre de TET4')
axes[1].legend(fontsize=8)
for ax in axes:
    ax.grid(True,which='both',alpha=.2)
fig.suptitle('FerraSim S09 — diagnostic ; acceptation du protocole suspendue',fontsize=12)
fig.savefig(args.output,dpi=160)
