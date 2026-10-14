use std::collections::BTreeSet;

use ferrasim_mesh::{ElementId, Mesh, MeshErrors, PhysicalGroup, Tet4, Triangle, ValidatedMesh};
use ferrasim_model::{GroupId, NodeId};
use thiserror::Error;

/// Categories of Gmsh file rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MshCode {
    /// Malformed or truncated input.
    Syntax,
    /// Version, binary encoding or data size is unsupported.
    UnsupportedFormat,
    /// Topology other than TRI3 or TET4.
    UnsupportedElement,
    /// Section not implemented; never ignored silently.
    UnsupportedSection,
    /// Partition/extra tag metadata is unsupported.
    UnsupportedTags,
    /// Node or element identifier occurs twice.
    DuplicateId,
}

/// Structured syntax/format or geometric import failure.
#[derive(Debug, Error)]
pub enum MshError {
    /// File-level failure with source line.
    #[error("MSH import failed at line {line}: {message}")]
    Format {
        /// Failure category.
        code: MshCode,
        /// One-based input line.
        line: usize,
        /// Diagnostic detail.
        message: String,
    },
    /// Geometric and topological validation failures after parsing.
    #[error(transparent)]
    Mesh(#[from] MeshErrors),
}

fn error(code: MshCode, line: usize, message: impl Into<String>) -> MshError {
    MshError::Format {
        code,
        line,
        message: message.into(),
    }
}

struct Cursor<'a> {
    lines: Vec<(usize, &'a str)>,
    index: usize,
    eof: usize,
}
impl<'a> Cursor<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            lines: text
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    let line = line.trim();
                    (!line.is_empty()).then_some((i + 1, line))
                })
                .collect(),
            index: 0,
            eof: text.lines().count() + 1,
        }
    }
    fn next(&mut self) -> Result<(usize, &'a str), MshError> {
        let value = self
            .lines
            .get(self.index)
            .copied()
            .ok_or_else(|| error(MshCode::Syntax, self.eof, "unexpected end of file"))?;
        self.index += 1;
        Ok(value)
    }
    fn expect(&mut self, expected: &str) -> Result<(), MshError> {
        let (line, value) = self.next()?;
        if value != expected {
            return Err(error(
                MshCode::Syntax,
                line,
                format!("expected {expected}, found {value}"),
            ));
        }
        Ok(())
    }
    fn count(&mut self) -> Result<usize, MshError> {
        let (line, value) = self.next()?;
        value.parse().map_err(|_| {
            error(
                MshCode::Syntax,
                line,
                "expected a nonnegative section count",
            )
        })
    }
}

fn integer(value: &str, line: usize) -> Result<u64, MshError> {
    value.parse().map_err(|_| {
        error(
            MshCode::Syntax,
            line,
            format!("invalid unsigned integer {value:?}"),
        )
    })
}
fn format_header(text: &str, line: usize) -> Result<(), MshError> {
    let fields: Vec<_> = text.split_whitespace().collect();
    if fields != ["2.2", "0", "8"] {
        return Err(error(
            MshCode::UnsupportedFormat,
            line,
            "only Gmsh MSH 2.2 ASCII with data-size 8 is supported",
        ));
    }
    Ok(())
}

/// Import the supported MSH 2.2 ASCII subset and validate its mesh integrity.
///
/// Only triangle type 2, tetrahedron type 4 and up to two element tags are
/// accepted. PhysicalNames is optional; unknown sections are rejected.
pub fn parse_msh22(text: &str) -> Result<ValidatedMesh, MshError> {
    let mut cursor = Cursor::new(text);
    cursor.expect("$MeshFormat")?;
    let (header_line, header) = cursor.next()?;
    format_header(header, header_line)?;
    cursor.expect("$EndMeshFormat")?;
    let mut mesh = Mesh::default();
    let mut sections = BTreeSet::new();
    let mut element_ids = BTreeSet::new();
    while cursor.index < cursor.lines.len() {
        let (line, section) = cursor.next()?;
        if !sections.insert(section) {
            return Err(error(MshCode::Syntax, line, "duplicate section"));
        }
        match section {
            "$PhysicalNames" => {
                let count = cursor.count()?;
                for _ in 0..count {
                    let (line, value) = cursor.next()?;
                    let (prefix, quoted) = value.split_once('"').ok_or_else(|| {
                        error(MshCode::Syntax, line, "physical name must be quoted")
                    })?;
                    let fields: Vec<_> = prefix.split_whitespace().collect();
                    if fields.len() != 2 {
                        return Err(error(
                            MshCode::Syntax,
                            line,
                            "expected dimension, tag and quoted name",
                        ));
                    }
                    let dimension = integer(fields[0], line)?;
                    if !matches!(dimension, 2 | 3) {
                        return Err(error(
                            MshCode::UnsupportedElement,
                            line,
                            "only physical dimensions 2 and 3 are supported",
                        ));
                    }
                    let name =
                        serde_json::from_str::<String>(&format!("\"{quoted}")).map_err(|_| {
                            error(MshCode::Syntax, line, "invalid physical name string")
                        })?;
                    mesh.physical_groups.push(PhysicalGroup {
                        dimension: dimension as u8,
                        tag: integer(fields[1], line)?,
                        name: GroupId(name),
                    });
                }
                cursor.expect("$EndPhysicalNames")?;
            }
            "$Nodes" => {
                let count = cursor.count()?;
                for _ in 0..count {
                    let (line, value) = cursor.next()?;
                    let fields: Vec<_> = value.split_whitespace().collect();
                    if fields.len() != 4 {
                        return Err(error(
                            MshCode::Syntax,
                            line,
                            "node record needs ID and three coordinates",
                        ));
                    }
                    let id = NodeId(integer(fields[0], line)?);
                    let mut coordinates = [0.0; 3];
                    for (index, value) in fields[1..].iter().enumerate() {
                        coordinates[index] = value
                            .parse()
                            .map_err(|_| error(MshCode::Syntax, line, "invalid node coordinate"))?;
                    }
                    if mesh.nodes.insert(id.clone(), coordinates).is_some() {
                        return Err(error(
                            MshCode::DuplicateId,
                            line,
                            format!("duplicate node {}", id.0),
                        ));
                    }
                }
                cursor.expect("$EndNodes")?;
            }
            "$Elements" => {
                let count = cursor.count()?;
                for _ in 0..count {
                    let (line, value) = cursor.next()?;
                    let fields: Vec<_> = value.split_whitespace().collect();
                    if fields.len() < 3 {
                        return Err(error(MshCode::Syntax, line, "element header is truncated"));
                    }
                    let id = ElementId(integer(fields[0], line)?);
                    let kind = integer(fields[1], line)?;
                    let count = match kind {
                        2 => 3,
                        4 => 4,
                        _ => {
                            return Err(error(
                                MshCode::UnsupportedElement,
                                line,
                                format!("unsupported element type {kind} at element {}", id.0),
                            ));
                        }
                    };
                    let tags = integer(fields[2], line)?;
                    if tags > 2 {
                        return Err(error(
                            MshCode::UnsupportedTags,
                            line,
                            "partition/extra tags are not supported",
                        ));
                    }
                    let tags = tags as usize;
                    if fields.len() != 3 + tags + count {
                        return Err(error(
                            MshCode::Syntax,
                            line,
                            "element tag/connectivity length mismatch",
                        ));
                    }
                    if !element_ids.insert(id) {
                        return Err(error(
                            MshCode::DuplicateId,
                            line,
                            format!("duplicate element {}", id.0),
                        ));
                    }
                    let physical_tag = if tags >= 1 {
                        Some(integer(fields[3], line)?).filter(|tag| *tag != 0)
                    } else {
                        None
                    };
                    let entity_tag = if tags >= 2 {
                        Some(integer(fields[4], line)?).filter(|tag| *tag != 0)
                    } else {
                        None
                    };
                    let nodes: Vec<_> = fields[3 + tags..]
                        .iter()
                        .map(|value| integer(value, line).map(NodeId))
                        .collect::<Result<_, _>>()?;
                    if kind == 2 {
                        mesh.triangles.insert(
                            id,
                            Triangle {
                                id,
                                nodes: [nodes[0].clone(), nodes[1].clone(), nodes[2].clone()],
                                physical_tag,
                                entity_tag,
                            },
                        );
                    } else {
                        mesh.tetrahedra.insert(
                            id,
                            Tet4 {
                                id,
                                nodes: [
                                    nodes[0].clone(),
                                    nodes[1].clone(),
                                    nodes[2].clone(),
                                    nodes[3].clone(),
                                ],
                                physical_tag,
                                entity_tag,
                            },
                        );
                    }
                }
                cursor.expect("$EndElements")?;
            }
            _ => {
                return Err(error(
                    MshCode::UnsupportedSection,
                    line,
                    format!("unsupported section {section:?}"),
                ));
            }
        }
    }
    if !sections.contains("$Nodes") || !sections.contains("$Elements") {
        return Err(error(
            MshCode::Syntax,
            cursor.eof,
            "Nodes and Elements sections are required",
        ));
    }
    Ok(mesh.into_validated()?)
}

/// Inspect the text header before UTF-8 decoding, so binary files fail clearly.
pub fn parse_msh22_bytes(bytes: &[u8]) -> Result<ValidatedMesh, MshError> {
    let mut lines = bytes
        .split(|byte| *byte == b'\n')
        .enumerate()
        .filter(|(_, line)| !line.iter().all(u8::is_ascii_whitespace));
    let (first_line, first) = lines
        .next()
        .ok_or_else(|| error(MshCode::Syntax, 1, "empty mesh file"))?;
    if std::str::from_utf8(first).map(str::trim) != Ok("$MeshFormat") {
        return Err(error(
            MshCode::Syntax,
            first_line + 1,
            "expected MeshFormat header",
        ));
    }
    let (header_line, header) = lines
        .next()
        .ok_or_else(|| error(MshCode::Syntax, 2, "missing mesh format"))?;
    let header = std::str::from_utf8(header).map_err(|_| {
        error(
            MshCode::UnsupportedFormat,
            header_line + 1,
            "non-text mesh header",
        )
    })?;
    format_header(header, header_line + 1)?;
    let text = std::str::from_utf8(bytes).map_err(|source| {
        error(
            MshCode::Syntax,
            bytes[..source.valid_up_to()]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count()
                + 1,
            "ASCII mesh metadata must be valid UTF-8",
        )
    })?;
    parse_msh22(text)
}
