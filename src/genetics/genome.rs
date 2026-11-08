use crate::genetics::{Chunk, Genetics, Interpretation};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{borrow::Cow, sync::{Arc, OnceLock}};

/// Longueur visée des fragments, en séquences.
const CHUNK_TARGET: usize = 12;

#[derive(Debug)]
struct GenomeData {
    chunks: Vec<Arc<Chunk>>,
    length: usize,
    id: u64,
    interpretation: OnceLock<Interpretation>,
}

impl GenomeData {
    fn new(chunks: Vec<Arc<Chunk>>) -> Self {
        let length = chunks.iter().map(|chunk| chunk.len()).sum();
        // L'identité ne dépend que des séquences, pas de la façon dont elles sont découpées en fragments.
        let id = chunks
            .iter()
            .flat_map(|chunk| chunk.bytes().iter())
            .fold(0xcbf2_9ce4_8422_2325_u64, |id, byte| {
                (id ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
            });
        Self {
            chunks,
            length,
            id,
            interpretation: OnceLock::new(),
        }
    }
}

/// Génome : une succession ordonnée de séquences de 8 bits, découpée en fragments partagés. Le cloner
/// ne copie rien ; modifier crée un nouveau génome qui réutilise tous les fragments intacts. L'identité
/// génétique (`id`) distingue les génomes de contenus différents.
#[derive(Clone, Debug)]
pub(crate) struct Genome {
    data: Arc<GenomeData>,
}

impl Genome {
    fn from_chunks(chunks: Vec<Arc<Chunk>>) -> Self {
        Self {
            data: Arc::new(GenomeData::new(chunks)),
        }
    }

    /// Génome de séquences `bytes`, dont les fragments passent par le registre de `genetics`.
    pub(crate) fn from_bytes(genetics: &Genetics, bytes: &[u8]) -> Self {
        Self::from_chunks(
            bytes
                .chunks(CHUNK_TARGET)
                .map(|piece| genetics.pool().intern(piece))
                .collect(),
        )
    }

    /// Même contenu, fragments dédoublonnés par le registre (après une lecture de fichier).
    pub(crate) fn interned(&self, genetics: &Genetics) -> Self {
        Self::from_bytes(genetics, &self.bytes())
    }

    pub(crate) fn len(&self) -> usize {
        self.data.length
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.data.length == 0
    }

    /// Identité génétique : empreinte du contenu, identique pour deux génomes de mêmes séquences.
    pub(crate) fn id(&self) -> u64 {
        self.data.id
    }

    pub(crate) fn chunks(&self) -> &[Arc<Chunk>] {
        &self.data.chunks
    }

    /// Vrai si les deux génomes sont le même objet en mémoire.
    pub(crate) fn is_same_object(&self, other: &Genome) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    /// Nombre de fragments que les deux génomes partagent en mémoire.
    pub(crate) fn shared_chunks(&self, other: &Genome) -> usize {
        self.data
            .chunks
            .iter()
            .filter(|chunk| other.data.chunks.iter().any(|other| Arc::ptr_eq(chunk, other)))
            .count()
    }

    pub(crate) fn bytes(&self) -> Vec<u8> {
        self.slice(0, self.len())
    }

    /// Séquences `start..end` (bornées au génome).
    pub(crate) fn slice(&self, start: usize, end: usize) -> Vec<u8> {
        let (start, end) = (start.min(self.len()), end.min(self.len()));
        let mut bytes = Vec::with_capacity(end.saturating_sub(start));
        let mut offset = 0;
        for chunk in &self.data.chunks {
            let (from, to) = (offset, offset + chunk.len());
            if to > start && from < end {
                bytes.extend_from_slice(&chunk.bytes()[start.saturating_sub(from)..chunk.len().min(end - from)]);
            }
            offset = to;
        }
        bytes
    }

    /// Nouveau génome où les séquences `start..end` sont remplacées par `insert`. Seuls les fragments
    /// touchés sont recréés ; tous les autres sont partagés avec ce génome.
    pub(crate) fn splice(&self, genetics: &Genetics, start: usize, end: usize, insert: &[u8]) -> Genome {
        let start = start.min(self.len());
        let end = end.clamp(start, self.len());
        let (mut head, mut tail) = (Vec::new(), Vec::new());
        let (mut left, mut right) = (Vec::new(), Vec::new());
        let mut offset = 0;
        for chunk in &self.data.chunks {
            let (from, to) = (offset, offset + chunk.len());
            if to <= start {
                head.push(chunk.clone());
            } else if from >= end {
                tail.push(chunk.clone());
            } else {
                if from < start {
                    left.extend_from_slice(&chunk.bytes()[..start - from]);
                }
                if to > end {
                    right.extend_from_slice(&chunk.bytes()[end - from..]);
                }
            }
            offset = to;
        }
        let middle: Vec<u8> = left.into_iter().chain(insert.iter().copied()).chain(right).collect();
        head.extend(middle.chunks(CHUNK_TARGET).map(|piece| genetics.pool().intern(piece)));
        head.extend(tail);
        Genome::from_chunks(head)
    }

    /// Interprétation du génome, calculée une fois puis partagée par toutes les cellules qui l'ont.
    pub(crate) fn interpretation(&self, genetics: &Genetics) -> Cow<'_, Interpretation> {
        let cached = self
            .data
            .interpretation
            .get_or_init(|| Interpretation::of(genetics, &self.data.chunks));
        if cached.rules() == genetics.id() {
            Cow::Borrowed(cached)
        } else {
            Cow::Owned(Interpretation::of(genetics, &self.data.chunks))
        }
    }

    /// Vrai si l'interprétation de ce génome est déjà calculée.
    pub(crate) fn is_interpreted(&self) -> bool {
        self.data.interpretation.get().is_some()
    }
}

impl PartialEq for Genome {
    fn eq(&self, other: &Self) -> bool {
        self.is_same_object(other) || (self.data.id == other.data.id && self.bytes() == other.bytes())
    }
}

impl Serialize for Genome {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.bytes().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Genome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        Ok(Self::from_chunks(
            bytes.chunks(CHUNK_TARGET).map(|piece| Arc::new(Chunk::new(piece))).collect(),
        ))
    }
}
