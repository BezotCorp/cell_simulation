use fyrox::resource::texture::{
    Texture, TextureKind, TexturePixelKind, TextureResource,
};
use std::collections::HashMap;
use uuid::Uuid;

/// Textures 1×1 partagées : une par valeur de rugosité ou de métallicité distincte, et une
/// texture blanche qui active l'émission des matériaux (sans elle, l'émission est nulle).
#[derive(Default)]
pub(super) struct PixelCache {
    scalars: HashMap<u8, TextureResource>,
    white: Option<TextureResource>,
}

impl PixelCache {
    fn texture(kind: TexturePixelKind, bytes: Vec<u8>) -> TextureResource {
        let texture = Texture::from_bytes(
            TextureKind::Rectangle {
                width: 1,
                height: 1,
            },
            kind,
            bytes,
        )
        .expect("une texture 1×1 est toujours valide");
        TextureResource::new_ok(Uuid::now_v7(), Default::default(), texture)
    }

    /// Valeur scalaire (canal rouge) dans [0, 1].
    pub(super) fn scalar(&mut self, value: f32) -> TextureResource {
        let byte = (value.clamp(0.0, 1.0) * 255.0) as u8;
        self.scalars
            .entry(byte)
            .or_insert_with(|| Self::texture(TexturePixelKind::R8, vec![byte]))
            .clone()
    }

    pub(super) fn white(&mut self) -> TextureResource {
        self.white
            .get_or_insert_with(|| Self::texture(TexturePixelKind::RGBA8, vec![255; 4]))
            .clone()
    }
}
