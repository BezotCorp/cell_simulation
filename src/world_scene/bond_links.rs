use crate::definitions::MaterialKind;
use crate::world_scene::{
    geometry::{self, mesh},
    materials::Materials,
    organism_visual::OrganismVisual,
};
use fyrox::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::Handle,
    },
    scene::{graph::Graph, mesh::Mesh},
};
use std::collections::HashMap;
use uuid::Uuid;

/// Nombre de liaisons dessinées au plus.
const LINKS: usize = 600;

/// Filaments entre organismes liés : un lot de barres réutilisées, placées entre les deux membranes.
pub(super) struct BondLinks {
    nodes: Vec<Handle<Mesh>>,
    shown: usize,
    spacing: f32,
}

impl BondLinks {
    pub(super) fn new(graph: &mut Graph, materials: &Materials, spacing: f32) -> Self {
        let nodes = (0..LINKS)
            .map(|_| {
                let node = mesh(
                    graph,
                    "Liaison",
                    Uuid::now_v7(),
                    Vector3::zeros(),
                    Vector3::new(0.1, 0.02, 0.02) * spacing,
                    geometry::cube(),
                    materials.get(MaterialKind::Nucleus),
                );
                graph[node].set_cast_shadows(false);
                graph[node].set_visibility(false);
                node
            })
            .collect();
        Self {
            nodes,
            shown: 0,
            spacing,
        }
    }

    /// Place chaque liaison entre les visuels de ses deux membres ; celles dont un membre n'est pas
    /// encore dessiné, ou qui traversent la couture du monde torique, ne sont pas tracées.
    pub(super) fn update(
        &mut self,
        graph: &mut Graph,
        pairs: &[(Uuid, Uuid)],
        cells: &HashMap<Uuid, OrganismVisual>,
        visible: bool,
    ) {
        let mut used = 0;
        if visible {
            for (a, b) in pairs {
                if used == LINKS {
                    break;
                }
                let (Some(from), Some(to)) = (cells.get(a), cells.get(b)) else {
                    continue;
                };
                let (pa, pb) = (from.position(), to.position());
                let offset = pb - pa;
                let length = offset.norm();
                if length > 2.0 * self.spacing || length < 1e-4 {
                    continue;
                }
                let node = &mut graph[self.nodes[used]];
                node.set_visibility(true);
                node.local_transform_mut()
                    .set_position((pa + pb) * 0.5)
                    .set_rotation(UnitQuaternion::rotation_between(&Vector3::x(), &offset)
                        .unwrap_or_else(UnitQuaternion::identity))
                    .set_scale(Vector3::new(length, 0.025 * self.spacing, 0.025 * self.spacing));
                used += 1;
            }
        }
        for slot in used..self.shown {
            graph[self.nodes[slot]].set_visibility(false);
        }
        self.shown = used;
    }
}
