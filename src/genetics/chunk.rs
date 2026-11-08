use crate::genetics::{ChunkAnalysis, Genetics};
use std::{borrow::Cow, sync::OnceLock};

/// Fragment de génome (quelques séquences de 8 bits), immuable et partagé par tous les génomes qui
/// le contiennent. Il porte son empreinte et, une fois calculée, son interprétation.
#[derive(Debug)]
pub(crate) struct Chunk {
    bytes: Box<[u8]>,
    hash: u64,
    analysis: OnceLock<ChunkAnalysis>,
}

impl Chunk {
    pub(crate) fn new(bytes: &[u8]) -> Self {
        // FNV-1a : une empreinte rapide qui sert à retrouver les fragments identiques.
        let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        });
        Self {
            bytes: bytes.into(),
            hash,
            analysis: OnceLock::new(),
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(crate) fn hash(&self) -> u64 {
        self.hash
    }

    /// Interprétation du fragment, calculée une seule fois pour toutes les règles identiques.
    pub(crate) fn analysis(&self, genetics: &Genetics) -> Cow<'_, ChunkAnalysis> {
        let cached = self.analysis.get_or_init(|| ChunkAnalysis::of(genetics, &self.bytes));
        if cached.rules() == genetics.id() {
            Cow::Borrowed(cached)
        } else {
            Cow::Owned(ChunkAnalysis::of(genetics, &self.bytes))
        }
    }

}
