use fyrox::{core::pool::Handle, scene::mesh::Mesh};

/// Apparence courante d'une dalle ; sert à ne toucher au graphe de scène que si elle change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileState {
    Ground,
    Water,
    Temperature(u8),
}

pub(crate) struct HabitatVisual {
    tile: Handle<Mesh>,
    tile_state: Option<TileState>,
    object: Option<Handle<Mesh>>,
    /// Palier de hauteur de la nourriture déjà appliqué à l'objet.
    food_bucket: Option<u8>,
}

impl HabitatVisual {
    pub(crate) fn new(tile: Handle<Mesh>, object: Option<Handle<Mesh>>) -> Self {
        Self {
            tile,
            tile_state: None,
            object,
            food_bucket: None,
        }
    }

    pub(crate) fn tile(&self) -> Handle<Mesh> {
        self.tile
    }

    pub(crate) fn object(&self) -> Option<Handle<Mesh>> {
        self.object
    }

    pub(crate) fn set_object(&mut self, object: Handle<Mesh>) {
        self.object = Some(object);
    }

    /// Enregistre `state` et indique s'il diffère de l'apparence déjà appliquée.
    pub(crate) fn update_tile_state(&mut self, state: TileState) -> bool {
        self.tile_state.replace(state) != Some(state)
    }

    /// Enregistre le palier de nourriture et indique s'il a changé.
    pub(crate) fn update_food_bucket(&mut self, bucket: u8) -> bool {
        self.food_bucket.replace(bucket) != Some(bucket)
    }
}
