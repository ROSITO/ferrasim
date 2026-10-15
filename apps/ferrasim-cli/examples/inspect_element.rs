//! Element-local verification field using a model's geometry and material only.

use ferrasim_fem::{ElementDisplacements, IsotropicElastic, Tet4Element};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: inspect_element <model.json>")?;
    let loaded = ferrasim_io::load_meshed_model(path)?;
    let id = *loaded
        .mesh
        .mesh()
        .tetrahedra
        .keys()
        .next()
        .ok_or("no tetrahedra")?;
    let assignment = loaded
        .model
        .simulation
        .material_assignments
        .iter()
        .find(|assignment| {
            loaded
                .mesh
                .volume_groups()
                .get(&assignment.region)
                .is_some_and(|ids| ids.contains(&id))
        })
        .ok_or("element has no material")?;
    let material = loaded
        .model
        .simulation
        .materials
        .iter()
        .find(|material| material.id() == &assignment.material)
        .ok_or("material is missing")?;
    let law = IsotropicElastic::try_from(material)?;
    for warning in law.warnings() {
        println!("Material advisory: {warning:?}");
    }
    let element = Tet4Element::from_mesh(&loaded.mesh, id)?;
    let displacements = ElementDisplacements::from_fn(|index, _| {
        let axis = index % 3;
        let factor = if axis == 0 {
            0.001
        } else {
            -law.poisson() * 0.001
        };
        factor * element.coordinates()[index / 3][axis]
    });
    let k = element.stiffness(&law)?;
    let response = element.response(&law, &displacements)?;
    println!(
        "Element {}: volume={:.12e}, stiffness symmetry error={:.4e}",
        id.0,
        element.volume(),
        (k - k.transpose()).norm() / k.norm()
    );
    println!("Artificial uniaxial strain field, using geometry/material only.");
    println!(
        "Local stress [xx yy zz xy yz xz]: {:?}",
        response.stress.as_slice()
    );
    println!("Integrated elastic energy: {:.12e}", response.energy);
    println!("Model loads and boundary conditions have not been solved.");
    Ok(())
}
