"""Analytic uniaxial solid and exact consistent surface forces, independent of Rust.

The fixture uses a 2x1x1 block split into six TET4s. No stiffness or solver is
called: the reference follows sigma_xx=10, E=1000, nu=1/4 and uniform traction.
"""
import argparse
from fractions import Fraction as F
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def artifacts():
    ids = [90, 7, 400, 23, 61, 800, 51, 12]
    coordinates = [[0,0,0],[2,0,0],[2,1,0],[0,1,0],
                   [0,0,1],[2,0,1],[2,1,1],[0,1,1]]
    tetrahedra = [[0,1,2,6],[0,2,3,6],[0,3,7,6],
                  [0,7,4,6],[0,4,5,6],[0,5,1,6]]
    surfaces = {"x0":[[0,3,7],[0,7,4]], "x1":[[1,2,6],[1,6,5]],
                "y0":[[0,1,5],[0,5,4]], "y1":[[3,2,6],[3,6,7]],
                "z0":[[0,1,2],[0,2,3]], "z1":[[4,5,6],[4,6,7]]}
    forces = [[F(0) for _ in range(3)] for _ in ids]
    for triangle in surfaces["x1"]:
        # Each right triangle has known area 1/2; exact integral of N_a is A/3.
        for node in triangle:
            forces[node][0] += F(10,6)
    epsilon = F(1,100)
    displacements = [[epsilon*x, -epsilon*F(1,4)*y, -epsilon*F(1,4)*z]
                     for x,y,z in coordinates]
    reference = dict(reference_version=1, method="Analytic uniform axial stress; rational surface shape integrals",
                     node_ids=ids, coordinates=coordinates, tetrahedra=tetrahedra,
                     surfaces=surfaces, forces=[[str(x) for x in row] for row in forces],
                     displacements=[[str(x) for x in row] for row in displacements],
                     stress=["10","0","0","0","0","0"], strain_energy="1/10",
                     resultant=["10","0","0"], moment_origin=[0,0,0], moment=["0","5","-5"])
    msh = ["$MeshFormat","2.2 0 8","$EndMeshFormat","$PhysicalNames","7"]
    for tag,name in enumerate(surfaces,1): msh.append(f'2 {tag} "{name}"')
    msh += ['3 1 "solid"',"$EndPhysicalNames","$Nodes","8"]
    for node,point in zip(ids,coordinates): msh.append(f"{node} {' '.join(map(str,point))}")
    msh += ["$EndNodes","$Elements","18"]
    element=1
    for tag,triangles in enumerate(surfaces.values(),1):
        for triangle in triangles:
            msh.append(f"{element} 2 2 {tag} {tag} "+" ".join(str(ids[i]) for i in triangle)); element+=1
    for tetrahedron in tetrahedra:
        msh.append(f"{element} 4 2 1 1 "+" ".join(str(ids[i]) for i in tetrahedron)); element+=1
    msh += ["$EndElements"]
    model = dict(schema_version="0.1", units="SI", mesh=dict(path="mesh.msh",groups=dict(nodes=["x0","y0","z0"],surfaces=list(surfaces),volumes=["solid"])),
                 materials=[dict(id="elastic",type="isotropic_linear_elastic",young=1000.0,poisson=0.25)],
                 material_assignments=[dict(region="solid",material="elastic")],
                 boundary_conditions=[dict(node_group=name,components={axis:0.0}) for name,axis in [("x0","x"),("y0","y"),("z0","z")]],
                 loads=[dict(type="surface_traction",surface_group="x1",components=dict(x=10.0,y=0.0,z=0.0))],
                 analysis=dict(type="linear_static_3d"),solver_settings=dict(method="cg",preconditioner="jacobi",rtol=1e-12,atol=1e-14,max_iterations=1000),outputs=dict(formats=["vtu","csv","json"]))
    def encoded(value): return json.dumps(value,indent=2,sort_keys=True)+"\n"
    return {ROOT/"validation/fixtures/uniaxial-reference.json": encoded(reference),
            ROOT/"examples/uniaxial/model.json": encoded(model),
            ROOT/"examples/uniaxial/mesh.msh": "\n".join(msh)+"\n"}


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    args=parser.parse_args()
    for path, content in artifacts().items():
        if args.check:
            if path.read_text(encoding="utf-8") != content:
                raise SystemExit(f"Load reference artifact differs: {path}")
        else:
            path.parent.mkdir(parents=True,exist_ok=True)
            path.write_text(content,encoding="utf-8")
    print("Analytic uniaxial reference, mesh and model match." if args.check else "Uniaxial artifacts generated.")
