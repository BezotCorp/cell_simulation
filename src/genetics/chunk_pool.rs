use crate::genetics::Chunk;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicU64, Ordering},
    },
};

/// Seuil initial d'entrées avant un nettoyage des fragments disparus.
const PURGE_FROM: usize = 8192;

/// Dédoublonnage des fragments : deux fragments de mêmes octets sont un seul objet en mémoire, avec une
/// seule interprétation. Le registre ne retient aucun fragment : il ne garde que des références faibles.
#[derive(Debug, Default)]
pub(crate) struct ChunkPool {
    entries: Mutex<(HashMap<u64, Vec<Weak<Chunk>>>, usize)>,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl ChunkPool {
    pub(crate) fn intern(&self, bytes: &[u8]) -> Arc<Chunk> {
        let candidate = Chunk::new(bytes);
        let mut guard = self.entries.lock().expect("registre des fragments");
        let (map, limit) = &mut *guard;
        if let Some(found) = map
            .get(&candidate.hash())
            .and_then(|list| list.iter().filter_map(Weak::upgrade).find(|c| c.bytes() == bytes))
        {
            self.hits.fetch_add(1, Ordering::Relaxed);
            return found;
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        let chunk = Arc::new(candidate);
        map.entry(chunk.hash()).or_default().push(Arc::downgrade(&chunk));
        if map.len() > (*limit).max(PURGE_FROM) {
            map.retain(|_, list| {
                list.retain(|weak| weak.strong_count() > 0);
                !list.is_empty()
            });
            *limit = 2 * map.len();
        }
        chunk
    }

    /// (fragments retrouvés, fragments créés) depuis la création du registre.
    pub(crate) fn statistics(&self) -> (u64, u64) {
        (self.hits.load(Ordering::Relaxed), self.misses.load(Ordering::Relaxed))
    }

    /// Nombre de fragments distincts encore vivants.
    pub(crate) fn alive(&self) -> usize {
        let guard = self.entries.lock().expect("registre des fragments");
        guard
            .0
            .values()
            .flatten()
            .filter(|weak| weak.strong_count() > 0)
            .count()
    }
}
