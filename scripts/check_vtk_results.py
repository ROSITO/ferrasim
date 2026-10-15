"""Open exported VTU in native VTK; optionally render its unsmoothed cell field."""
import argparse
from pathlib import Path
import vtk
from check_result_exports import check

parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('directory',type=Path)
parser.add_argument('--render',type=Path)
args=parser.parse_args()
report=check(args.directory)
errors=[]
reader=vtk.vtkXMLUnstructuredGridReader()
reader.AddObserver('ErrorEvent',lambda *_:errors.append('VTK reader error'))
reader.SetFileName(str(args.directory/'results.vtu'))
reader.Update()
grid=reader.GetOutput()
assert not errors and reader.GetErrorCode()==0
assert grid.GetNumberOfPoints()==len(report['nodes'])
assert grid.GetNumberOfCells()==len(report['elements'])
for i,n in enumerate(report['nodes']):
    assert grid.GetPointData().GetArray('NodeId').GetValue(i)==n['id']
    assert list(grid.GetPoint(i))==n['coordinates']
    assert list(grid.GetPointData().GetArray('Displacement').GetTuple(i))==n['displacement']
for i,e in enumerate(report['elements']):
    assert grid.GetCellType(i)==vtk.VTK_TETRA
    assert grid.GetCellData().GetArray('ElementId').GetValue(i)==e['id']
    assert list(grid.GetCellData().GetArray('StressVoigt').GetTuple(i))==e['stress']
    assert list(grid.GetCellData().GetArray('StrainEngineering').GetTuple(i))==e['strain']
    assert grid.GetCellData().GetArray('VonMises').GetValue(i)==e['von_mises']
    assert grid.GetPointData().GetArray('StressVoigt') is None
if args.render:
    warp=vtk.vtkWarpVector();warp.SetInputData(grid)
    warp.SetInputArrayToProcess(0,0,0,vtk.vtkDataObject.FIELD_ASSOCIATION_POINTS,'Displacement');warp.SetScaleFactor(10.0)
    mapper=vtk.vtkDataSetMapper();mapper.SetInputConnection(warp.GetOutputPort())
    mapper.SetScalarModeToUseCellFieldData();mapper.SelectColorArray('VonMises');mapper.SetScalarRange(0.0,12.0)
    actor=vtk.vtkActor();actor.SetMapper(mapper);actor.GetProperty().EdgeVisibilityOn();actor.GetProperty().SetEdgeColor(0.1,0.1,0.15)
    renderer=vtk.vtkRenderer();renderer.SetBackground(0.96,0.97,0.99);renderer.AddActor(actor)
    bar=vtk.vtkScalarBarActor();bar.SetLookupTable(mapper.GetLookupTable());bar.SetTitle('von Mises (Pa)');bar.SetNumberOfLabels(5);bar.GetTitleTextProperty().SetColor(0.1,0.1,0.15);bar.GetLabelTextProperty().SetColor(0.1,0.1,0.15);renderer.AddViewProp(bar)
    text=vtk.vtkTextActor();text.SetInput('FerraSim S07 | VTK native reader\n6 TET4 | deformation x10 | raw cell stress');text.SetPosition(20,20);text.GetTextProperty().SetColor(0.1,0.1,0.15);text.GetTextProperty().SetFontSize(18);renderer.AddViewProp(text)
    renderer.ResetCamera();renderer.GetActiveCamera().Azimuth(25);renderer.GetActiveCamera().Elevation(20);renderer.ResetCameraClippingRange()
    window=vtk.vtkRenderWindow();window.SetOffScreenRendering(1);window.SetSize(1100,700);window.AddRenderer(renderer);window.Render()
    capture=vtk.vtkWindowToImageFilter();capture.SetInput(window);capture.ReadFrontBufferOff();capture.Update()
    args.render.parent.mkdir(parents=True,exist_ok=True)
    writer=vtk.vtkPNGWriter();writer.SetFileName(str(args.render));writer.SetInputConnection(capture.GetOutputPort());writer.Write();window.Finalize()
print(f"VTK {vtk.vtkVersion.GetVTKVersion()} read all fields and connectivity"+(' and rendered the cell field.' if args.render else '.'))
