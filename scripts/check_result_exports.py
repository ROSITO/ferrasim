"""Cross-check an all-format result bundle with standard-library XML/CSV/JSON readers."""
import csv
import json
from pathlib import Path
import sys
import xml.etree.ElementTree as ET


def check(directory):
    directory=Path(directory)
    manifest=json.loads((directory/'manifest.json').read_text(encoding='utf-8'))
    assert manifest['status']=='complete'
    for file in manifest['files']:
        assert (directory/file['path']).stat().st_size==file['bytes']
    report=json.loads((directory/'report.json').read_text(encoding='utf-8'))
    assert report['analysis_status']=='converged'
    assert report['scientific_qualification']=='pending_independent_review'
    assert manifest['units']==report['units']
    tree=ET.parse(directory/'results.vtu')
    piece=tree.find('./UnstructuredGrid/Piece')
    nodes=report['nodes']; elements=report['elements']
    assert int(piece.attrib['NumberOfPoints'])==len(nodes)
    assert int(piece.attrib['NumberOfCells'])==len(elements)
    def arrays(section):
        return {a.attrib['Name']:a for a in piece.find(section).findall('DataArray')}
    points=arrays('PointData');cells=arrays('CellData');topology=arrays('Cells')
    def values(a):
        cast=int if a.attrib['type'].startswith(('Int','UInt')) else float
        return list(map(cast,(a.text or '').split()))
    def flatten(rows):return [v for row in rows for v in row]
    assert values(points['NodeId'])==[n['id'] for n in nodes]
    assert values(cells['ElementId'])==[e['id'] for e in elements]
    assert values(arrays('Points')['Coordinates'])==flatten(n['coordinates'] for n in nodes)
    for field,key in [('Displacement','displacement'),('AppliedForce','applied_force')]:
        assert values(points[field])==flatten(n[key] for n in nodes)
    assert values(points['PrescribedMask'])==flatten([int(x is not None) for x in n['prescribed']] for n in nodes)
    assert values(points['Reaction'])==flatten([0.0 if x is None else x for x in n['reaction']] for n in nodes)
    for field,key in [('StressVoigt','stress'),('StrainEngineering','strain')]:
        assert int(cells[field].attrib['NumberOfComponents'])==6
        assert values(cells[field])==flatten(e[key] for e in elements)
        assert field not in points
    for field,key in [('VonMises','von_mises'),('Volume','volume'),('ElasticEnergy','energy')]:
        assert values(cells[field])==[e[key] for e in elements]
    indices={n['id']:i for i,n in enumerate(nodes)}
    assert values(topology['connectivity'])==flatten([indices[n] for n in e['nodes']] for e in elements)
    assert values(topology['offsets'])==[4*(i+1) for i in range(len(elements))]
    assert values(topology['types'])==[10]*len(elements)
    material_indices={m['id']:m['index'] for m in report['material_indices']}
    assert values(cells['MaterialIndex'])==[material_indices[e['material']] for e in elements]
    with (directory/'nodes.csv').open(encoding='utf-8',newline='') as file: rows=list(csv.DictReader(file))
    assert len(rows)==len(nodes)
    for row,node in zip(rows,nodes):
        assert int(row['node_id'])==node['id']
        for keys,name in [('xyz','coordinates'),(['ux','uy','uz'],'displacement'),(['fx','fy','fz'],'applied_force')]:
            assert [float(row[k]) for k in keys]==node[name]
        for key,value in zip(['rx','ry','rz'],node['reaction']): assert (None if not row[key] else float(row[key]))==value
        for key,value in zip(['prescribed_x','prescribed_y','prescribed_z'],node['prescribed']): assert (None if not row[key] else float(row[key]))==value
    with (directory/'elements.csv').open(encoding='utf-8',newline='') as file: rows=list(csv.DictReader(file))
    assert len(rows)==len(elements)
    for row,e in zip(rows,elements):
        assert int(row['element_id'])==e['id']
        assert [int(row[f'n{i}']) for i in range(1,5)]==e['nodes']
        assert row['material_id']==e['material']
        for keys,key in [(['epsilon_xx','epsilon_yy','epsilon_zz','gamma_xy','gamma_yz','gamma_xz'],'strain'),(['sigma_xx','sigma_yy','sigma_zz','tau_xy','tau_yz','tau_xz'],'stress')]:
            assert [float(row[k]) for k in keys]==e[key]
        for key in ['volume','von_mises']:assert float(row[key])==e[key]
        assert float(row['elastic_energy'])==e['energy']
    return report


if __name__=='__main__':
    report=check(sys.argv[1])
    print(f"VTU/CSV/JSON/manifest agree: {len(report['nodes'])} nodes, {len(report['elements'])} TET4s.")
