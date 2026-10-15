"""Check geometry, area integration and refinement independently of the Rust solver."""
import hashlib
import json
from pathlib import Path
import tempfile
from run_cantilever import generate


def main():
    with tempfile.TemporaryDirectory() as temp:
        previous = set()
        for m in [1,2,4]:
            root = Path(temp)/str(m)
            nodes, tip = generate(root,m)
            coordinates = {n:xyz for n,*xyz in nodes}
            points = {tuple(xyz) for xyz in coordinates.values()}
            assert previous <= points
            previous = points
            lines=(root/'mesh.msh').read_text().splitlines()
            start=lines.index('$Elements')+2
            volume=0.
            count=0
            faces={}
            for line in lines[start:lines.index('$EndElements')]:
                fields=list(map(int,line.split()))
                if fields[1]!=4:
                    continue
                ids=fields[5:]
                a,b,c,d=[coordinates[n] for n in ids]
                u,v,w=[[q[i]-a[i] for i in range(3)] for q in [b,c,d]]
                det=sum(u[i]*(v[(i+1)%3]*w[(i+2)%3]-v[(i+2)%3]*w[(i+1)%3]) for i in range(3))
                assert det>0
                volume+=det/6
                count+=1
                for omitted in range(4):
                    face=tuple(sorted(n for i,n in enumerate(ids) if i!=omitted))
                    faces[face]=faces.get(face,0)+1
            assert count==120*m**3
            assert abs(volume-.0025)<1e-14
            assert set(faces.values())=={1,2}
            for face, incidence in faces.items():
                if incidence==1:
                    assert any(all(coordinates[n][axis]==edge for n in face)
                               for axis, extent in enumerate([1.,.05,.05]) for edge in [0.,extent])
            area=0.
            integral=0.
            for tri in tip:
                assert faces[tuple(sorted(tri))]==1
                a,b,c=[coordinates[n] for n in tri]
                assert all(p[0]==1 for p in [a,b,c])
                weight=abs((b[1]-a[1])*(c[2]-a[2])-(b[2]-a[2])*(c[1]-a[1]))/2
                area+=weight
                integral+=weight*sum(p[1]+2*p[2] for p in [a,b,c])/3
            assert abs(area-.0025)<1e-15
            assert abs(integral/area-.075)<1e-14
            assert abs(json.loads((root/'model.json').read_text())['loads'][0]['components']['z']*area+100)<1e-10
            first=(root/'mesh.msh').read_bytes(),(root/'model.json').read_bytes()
            other=Path(temp)/f'repeated{m}'
            generate(other,m)
            assert first==((other/'mesh.msh').read_bytes(),(other/'model.json').read_bytes())
    for name in ['protocol-v1','diagnostic']:
        data=json.loads((Path(__file__).parent/f'results/cantilever/{name}.json').read_text())
        assert data['qualification']=='suspended'
        with tempfile.TemporaryDirectory() as temp:
            for row in data['runs']:
                root=Path(temp)/str(row['m'])
                generate(root,row['m'],rtol=row['solver']['settings']['rtol'])
                for filename,digest in row['input_sha256'].items():
                    assert hashlib.sha256((root/filename).read_bytes()).hexdigest()==digest
                assert abs(row['relative_error']-abs(row['tip_displacement_m']/data['reference_m']-1))<1e-14
    print('Cantilever geometry, conformity, nested refinement, integration and deterministic inputs passed.')

if __name__=='__main__':
    main()
