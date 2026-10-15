use ferrasim_fem::SimulationResults;
use ferrasim_model::OutputFormat;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use thiserror::Error;

/// Caller-supplied source locations, recorded as labels beside exact parsed snapshots.
#[derive(Debug, Clone)]
pub struct ResultProvenance {
    /// Model source label; typically the canonical source file path.
    pub model_source: String,
    /// Mesh source label; typically the resolved mesh file path.
    pub mesh_source: String,
    /// Optional caller-declared build revision; None is recorded explicitly.
    pub declared_code_revision: Option<String>,
}
/// Serialization, output location or filesystem failure.
#[derive(Debug, Error)]
pub enum ResultExportError {
    /// JSON encoding failure.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Dataset dimensions cannot be represented by the VTU index type.
    #[error("VTU index range exceeded")]
    IndexRange,
    /// Filesystem operation failed; existing destinations are never overwritten.
    #[error("cannot export results at {path}: {source}")]
    Io {
        /// Affected location.
        path: PathBuf,
        /// Underlying operating-system error.
        #[source]
        source: std::io::Error,
    },
}
/// Completed new-directory export; a manifest is published only after every selected file.
#[derive(Debug, Clone)]
pub struct ExportedResults {
    /// Destination directory.
    pub directory: PathBuf,
    /// Relative filenames, including manifest.json last.
    pub files: Vec<String>,
}
fn materials(results: &SimulationResults) -> BTreeMap<&ferrasim_model::MaterialId, usize> {
    let mut ids: Vec<_> = results.model().materials.iter().map(|m| m.id()).collect();
    ids.sort();
    ids.into_iter().enumerate().map(|(i, id)| (id, i)).collect()
}
fn array(
    out: &mut String,
    kind: &str,
    name: &str,
    components: usize,
    values: impl IntoIterator<Item = String>,
) {
    writeln!(out,"<DataArray type=\"{kind}\" Name=\"{name}\" NumberOfComponents=\"{components}\" format=\"ascii\">").unwrap();
    let mut first = true;
    for value in values {
        if !first {
            out.push(' ');
        }
        out.push_str(&value);
        first = false;
    }
    out.push_str("\n</DataArray>\n");
}
/// Encode an ASCII XML unstructured grid. Raw stresses/strains are CellData only.
/// UInt64 IDs remain separate from zero-based Int64 connectivity.
pub fn results_to_vtu(results: &SimulationResults) -> Result<String, ResultExportError> {
    i64::try_from(results.nodes().len()).map_err(|_| ResultExportError::IndexRange)?;
    i64::try_from(
        results
            .elements()
            .len()
            .checked_mul(4)
            .ok_or(ResultExportError::IndexRange)?,
    )
    .map_err(|_| ResultExportError::IndexRange)?;
    let indices: BTreeMap<_, _> = results
        .nodes()
        .iter()
        .enumerate()
        .map(|(i, n)| (&n.id, i))
        .collect();
    let ids = materials(results);
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<VTKFile type=\"UnstructuredGrid\" version=\"0.1\" byte_order=\"LittleEndian\">\n<UnstructuredGrid>\n",
    );
    writeln!(
        out,
        "<Piece NumberOfPoints=\"{}\" NumberOfCells=\"{}\">",
        results.nodes().len(),
        results.elements().len()
    )
    .unwrap();
    out.push_str("<PointData Vectors=\"Displacement\">\n");
    array(
        &mut out,
        "UInt64",
        "NodeId",
        1,
        results.nodes().iter().map(|n| n.id.0.to_string()),
    );
    array(
        &mut out,
        "Float64",
        "Displacement",
        3,
        results
            .nodes()
            .iter()
            .flat_map(|n| n.displacement.map(|x| x.to_string())),
    );
    array(
        &mut out,
        "Float64",
        "AppliedForce",
        3,
        results
            .nodes()
            .iter()
            .flat_map(|n| n.applied_force.map(|x| x.to_string())),
    );
    array(
        &mut out,
        "Float64",
        "Reaction",
        3,
        results
            .nodes()
            .iter()
            .flat_map(|n| n.reaction.map(|x| x.unwrap_or(0.0).to_string())),
    );
    array(
        &mut out,
        "UInt8",
        "PrescribedMask",
        3,
        results
            .nodes()
            .iter()
            .flat_map(|n| n.prescribed.map(|x| u8::from(x.is_some()).to_string())),
    );
    array(
        &mut out,
        "Float64",
        "PrescribedDisplacement",
        3,
        results
            .nodes()
            .iter()
            .flat_map(|n| n.prescribed.map(|x| x.unwrap_or(0.0).to_string())),
    );
    out.push_str("</PointData>\n<CellData Scalars=\"VonMises\">\n");
    array(
        &mut out,
        "UInt64",
        "ElementId",
        1,
        results.elements().iter().map(|e| e.id.0.to_string()),
    );
    array(
        &mut out,
        "UInt64",
        "MaterialIndex",
        1,
        results
            .elements()
            .iter()
            .map(|e| ids[&e.material].to_string()),
    );
    array(
        &mut out,
        "Float64",
        "StrainEngineering",
        6,
        results
            .elements()
            .iter()
            .flat_map(|e| e.strain.map(|x| x.to_string())),
    );
    array(
        &mut out,
        "Float64",
        "StressVoigt",
        6,
        results
            .elements()
            .iter()
            .flat_map(|e| e.stress.map(|x| x.to_string())),
    );
    array(
        &mut out,
        "Float64",
        "VonMises",
        1,
        results.elements().iter().map(|e| e.von_mises.to_string()),
    );
    array(
        &mut out,
        "Float64",
        "Volume",
        1,
        results.elements().iter().map(|e| e.volume.to_string()),
    );
    array(
        &mut out,
        "Float64",
        "ElasticEnergy",
        1,
        results.elements().iter().map(|e| e.energy.to_string()),
    );
    out.push_str("</CellData>\n<Points>\n");
    array(
        &mut out,
        "Float64",
        "Coordinates",
        3,
        results
            .nodes()
            .iter()
            .flat_map(|n| n.coordinates.map(|x| x.to_string())),
    );
    out.push_str("</Points>\n<Cells>\n");
    array(
        &mut out,
        "Int64",
        "connectivity",
        1,
        results
            .elements()
            .iter()
            .flat_map(|e| e.nodes.iter().map(|id| indices[id].to_string())),
    );
    array(
        &mut out,
        "Int64",
        "offsets",
        1,
        (1..=results.elements().len()).map(|i| (i * 4).to_string()),
    );
    array(
        &mut out,
        "UInt8",
        "types",
        1,
        results.elements().iter().map(|_| "10".into()),
    );
    out.push_str("</Cells>\n</Piece>\n</UnstructuredGrid>\n</VTKFile>\n");
    Ok(out)
}
fn csv_cell(s: &str) -> String {
    if s.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}
fn csv_row(out: &mut String, cells: impl IntoIterator<Item = String>) {
    let row: Vec<_> = cells.into_iter().map(|s| csv_cell(&s)).collect();
    out.push_str(&row.join(","));
    out.push('\n');
}
/// Encode one row per external node; free reaction/prescription components are empty.
pub fn results_to_nodes_csv(results: &SimulationResults) -> String {
    let mut out = String::from(
        "node_id,x,y,z,ux,uy,uz,fx,fy,fz,rx,ry,rz,prescribed_x,prescribed_y,prescribed_z\n",
    );
    for n in results.nodes() {
        let mut row = vec![n.id.0.to_string()];
        row.extend(
            n.coordinates
                .into_iter()
                .chain(n.displacement)
                .chain(n.applied_force)
                .map(|x| x.to_string()),
        );
        row.extend(
            n.reaction
                .into_iter()
                .chain(n.prescribed)
                .map(|x| x.map(|x| x.to_string()).unwrap_or_default()),
        );
        csv_row(&mut out, row);
    }
    out
}
/// Encode one row per TET4, retaining unsmoothed engineering strain and stress order.
pub fn results_to_elements_csv(results: &SimulationResults) -> String {
    let mut out = String::from(
        "element_id,n1,n2,n3,n4,material_id,volume,epsilon_xx,epsilon_yy,epsilon_zz,gamma_xy,gamma_yz,gamma_xz,sigma_xx,sigma_yy,sigma_zz,tau_xy,tau_yz,tau_xz,von_mises,elastic_energy\n",
    );
    for e in results.elements() {
        let mut row = vec![e.id.0.to_string()];
        row.extend(e.nodes.iter().map(|id| id.0.to_string()));
        row.push(e.material.0.clone());
        row.extend(
            [e.volume]
                .into_iter()
                .chain(e.strain)
                .chain(e.stress)
                .chain([e.von_mises, e.energy])
                .map(|x| x.to_string()),
        );
        csv_row(&mut out, row);
    }
    out
}
fn balance_json(results: &SimulationResults) -> Value {
    let b = results.balance();
    json!({"origin":b.origin,"force_scale":b.scales.force(),"length_scale":b.scales.length(),
        "external_force":b.external_force,"reaction_force":b.reaction_force,"force_imbalance":b.force_imbalance,
        "external_moment":b.external_moment,"reaction_moment":b.reaction_moment,"moment_imbalance":b.moment_imbalance,
        "relative_force_error":b.relative_force_error,"relative_moment_error":b.relative_moment_error,
        "relative_free_residual":b.relative_free_residual,"relative_prescription_error":b.relative_prescription_error,
        "strain_energy":b.strain_energy,"load_work":b.load_work,"prescription_work":b.prescription_work,"relative_energy_error":b.relative_energy_error})
}
/// Encode report version 1, including exact parsed snapshots, units, settings and balances.
/// Source labels and any declared revision are metadata, not cryptographic attestations.
pub fn results_to_json(
    results: &SimulationResults,
    source: &ResultProvenance,
) -> Result<String, ResultExportError> {
    let mesh = results.mesh().mesh();
    let solver = results.solver();
    let report = json!({"result_version":1,"analysis_status":"converged","scientific_qualification":"pending_independent_review",
        "provenance":{"engine":"FerraSim","engine_version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,
            "model_source":source.model_source,"mesh_source":source.mesh_source,"declared_code_revision":source.declared_code_revision,
            "model_snapshot":results.model(),"mesh_snapshot":{
                "nodes":mesh.nodes.iter().map(|(id,x)|json!({"id":id.0,"coordinates":x})).collect::<Vec<_>>(),
                "tetrahedra":mesh.tetrahedra.values().map(|e|json!({"id":e.id.0,"nodes":e.nodes.each_ref().map(|id|id.0),"physical_tag":e.physical_tag,"entity_tag":e.entity_tag})).collect::<Vec<_>>(),
                "triangles":mesh.triangles.values().map(|e|json!({"id":e.id.0,"nodes":e.nodes.each_ref().map(|id|id.0),"physical_tag":e.physical_tag,"entity_tag":e.entity_tag})).collect::<Vec<_>>(),
                "physical_groups":mesh.physical_groups.iter().map(|g|json!({"dimension":g.dimension,"tag":g.tag,"name":g.name.0})).collect::<Vec<_>>() }},
        "units":results.model().units,"conventions":{"frame":"global Cartesian xyz","voigt":["xx","yy","zz","xy","yz","xz"],"shear_strain":"engineering gamma=2*epsilon","stress_location":"element constant, unsmoothed","reaction_null":"free component","energy":"force*length","stress":"force/length^2"},
        "material_indices":materials(results).iter().map(|(id,index)|json!({"index":index,"id":id.0})).collect::<Vec<_>>(),
        "solver":{"status":format!("{:?}",solver.status),"iterations":solver.iterations,"residual_norm":solver.residual_norm,"relative_residual":solver.relative_residual,"threshold":solver.threshold,"settings":results.model().solver_settings},
        "balance":balance_json(results),
        "material_warnings":results.material_warnings().iter().map(|(id,w)|json!({"material":id.0,"warning":format!("{w:?}")})).collect::<Vec<_>>(),
        "nodes":results.nodes().iter().map(|n|json!({"id":n.id.0,"coordinates":n.coordinates,"displacement":n.displacement,"applied_force":n.applied_force,"reaction":n.reaction,"prescribed":n.prescribed})).collect::<Vec<_>>(),
        "elements":results.elements().iter().map(|e|json!({"id":e.id.0,"nodes":e.nodes.each_ref().map(|id|id.0),"material":e.material.0,"volume":e.volume,"strain":e.strain,"stress":e.stress,"von_mises":e.von_mises,"energy":e.energy})).collect::<Vec<_>>() });
    Ok(serde_json::to_string_pretty(&report)? + "\n")
}
/// Export selected model formats into a new directory (parent must already exist).
/// Encodes everything first; writes manifest.json last. Existing destinations are rejected.
/// On write failure, removes only files created by this call and attempts to remove its directory.
/// A surviving partial directory has no completed manifest and must not be treated as success.
pub fn export_results(
    results: &SimulationResults,
    source: &ResultProvenance,
    directory: impl AsRef<Path>,
) -> Result<ExportedResults, ResultExportError> {
    let directory = directory.as_ref();
    let mut encoded = Vec::new();
    for format in &results.model().outputs.formats {
        match format {
            OutputFormat::Vtu => encoded.push(("results.vtu", results_to_vtu(results)?)),
            OutputFormat::Csv => {
                encoded.push(("nodes.csv", results_to_nodes_csv(results)));
                encoded.push(("elements.csv", results_to_elements_csv(results)));
            }
            OutputFormat::Json => encoded.push(("report.json", results_to_json(results, source)?)),
        }
    }
    let manifest = json!({"result_version":1,"status":"complete","scientific_qualification":"pending_independent_review",
        "provenance":{"engine":"FerraSim","engine_version":env!("CARGO_PKG_VERSION"),"model_source":source.model_source,"mesh_source":source.mesh_source,"declared_code_revision":source.declared_code_revision},
        "units":results.model().units,"voigt":["xx","yy","zz","xy","yz","xz"],
        "shear_strain":"engineering gamma=2*epsilon","stress_location":"element constant, unsmoothed",
        "material_indices":materials(results).iter().map(|(id,index)|json!({"index":index,"id":id.0})).collect::<Vec<_>>(),
        "files":encoded.iter().map(|(name,text)|json!({"path":name,"bytes":text.len()})).collect::<Vec<_>>()});
    let manifest = serde_json::to_string_pretty(&manifest)? + "\n";
    fs::create_dir(directory).map_err(|source| ResultExportError::Io {
        path: directory.to_owned(),
        source,
    })?;
    let mut created = Vec::new();
    let outcome = (|| {
        for (name, text) in encoded
            .iter()
            .map(|(n, t)| (*n, t))
            .chain([(".manifest.tmp", &manifest)])
        {
            let path = directory.join(name);
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(|source| ResultExportError::Io {
                    path: path.clone(),
                    source,
                })?;
            created.push(path.clone());
            file.write_all(text.as_bytes())
                .and_then(|_| file.sync_all())
                .map_err(|source| ResultExportError::Io { path, source })?;
        }
        fs::rename(
            directory.join(".manifest.tmp"),
            directory.join("manifest.json"),
        )
        .map_err(|source| ResultExportError::Io {
            path: directory.join("manifest.json"),
            source,
        })?;
        Ok::<_, ResultExportError>(())
    })();
    if let Err(error) = outcome {
        for path in &created {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_dir(directory);
        return Err(error);
    }
    let mut files: Vec<_> = encoded.iter().map(|(n, _)| (*n).to_owned()).collect();
    files.push("manifest.json".into());
    Ok(ExportedResults {
        directory: directory.to_owned(),
        files,
    })
}
