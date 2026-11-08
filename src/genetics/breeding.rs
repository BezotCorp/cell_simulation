use crate::genetics::{Genetics, Genome};
use rand::RngExt;

/// Plus grand nombre d'événements d'une même sorte à une division : une borne de sécurité.
const MAX_EVENTS: usize = 64;

/// Nombre d'événements d'une loi de Poisson de moyenne `mean`, tiré avec `rng`.
fn events(rng: &mut impl rand::Rng, mean: f64) -> usize {
    let mut remaining = mean.max(0.0);
    let mut count = 0;
    while remaining > 0.0 && count < MAX_EVENTS {
        if rng.random::<f64>() < remaining.min(1.0) {
            count += 1;
        }
        remaining -= 1.0;
    }
    count
}

impl Genetics {
    /// Taux de mutation d'une division : facteur du milieu (rayonnement, chaleur) et de la réparation.
    fn error_scale(&self, repair: f32, exposure: f32) -> f64 {
        f64::from(exposure.max(0.0)) / (1.0 + self.mutation().repair_gain * f64::from(repair.clamp(0.0, 1.0)))
    }

    /// Nombre moyen de mutations d'une division : il croît avec la longueur du génome (plus de bits à
    /// copier) et avec l'exposition, et décroît avec la réparation.
    pub(crate) fn expected_mutations(&self, genome: &Genome, repair: f32, exposure: f32) -> f64 {
        let m = self.mutation();
        let scale = self.error_scale(repair, exposure);
        scale
            * (m.bit_flip * genome.len() as f64 * 8.0
                + m.duplication
                + m.deletion
                + m.insertion
                + m.inversion
                + m.translocation)
    }

    /// Génome d'une fille : copie de celui de sa mère à laquelle s'appliquent des mutations en
    /// nombre aléatoire : erreurs de copie de bits, duplications, suppressions, insertions,
    /// inversions et déplacements de fragments. Chaque mutation est tirée sans tenir compte de son
    /// utilité. Sans mutation, la fille partage le génome de sa mère.
    pub(crate) fn reproduce(
        &self,
        parent: &Genome,
        rng: &mut impl rand::Rng,
        repair: f32,
        exposure: f32,
    ) -> Genome {
        let m = self.mutation();
        let scale = self.error_scale(repair, exposure);
        let mut genome = parent.clone();
        for _ in 0..events(rng, m.bit_flip * parent.len() as f64 * 8.0 * scale) {
            let bit = rng.random_range(0..genome.len() * 8);
            let (index, flipped) = (bit / 8, genome.slice(bit / 8, bit / 8 + 1)[0] ^ (0x80 >> (bit % 8)));
            genome = genome.splice(self, index, index + 1, &[flipped]);
        }
        for _ in 0..events(rng, m.duplication * scale) {
            let run = rng.random_range(1..=m.run_max.min(genome.len()));
            if genome.len() + run <= m.max_length {
                let from = rng.random_range(0..=genome.len() - run);
                let copy = genome.slice(from, from + run);
                let at = rng.random_range(0..=genome.len());
                genome = genome.splice(self, at, at, &copy);
            }
        }
        for _ in 0..events(rng, m.deletion * scale) {
            let run = rng.random_range(1..=m.run_max.min(genome.len()));
            if genome.len() >= m.min_length + run {
                let from = rng.random_range(0..=genome.len() - run);
                genome = genome.splice(self, from, from + run, &[]);
            }
        }
        for _ in 0..events(rng, m.insertion * scale) {
            let run = rng.random_range(1..=m.run_max);
            if genome.len() + run <= m.max_length {
                let fresh: Vec<u8> = (0..run).map(|_| rng.random()).collect();
                let at = rng.random_range(0..=genome.len());
                genome = genome.splice(self, at, at, &fresh);
            }
        }
        for _ in 0..events(rng, m.inversion * scale) {
            let run = rng.random_range(1..=m.run_max.min(genome.len()));
            let from = rng.random_range(0..=genome.len() - run);
            let mut piece = genome.slice(from, from + run);
            piece.reverse();
            // Une inversion peut aussi retourner les bits de chaque séquence : les motifs changent de sens.
            if rng.random_bool(0.5) {
                piece.iter_mut().for_each(|byte| *byte = byte.reverse_bits());
            }
            genome = genome.splice(self, from, from + run, &piece);
        }
        for _ in 0..events(rng, m.translocation * scale) {
            let run = rng.random_range(1..=m.run_max.min(genome.len().saturating_sub(1)).max(1));
            if genome.len() > run {
                let from = rng.random_range(0..=genome.len() - run);
                let piece = genome.slice(from, from + run);
                let rest = genome.splice(self, from, from + run, &[]);
                let at = rng.random_range(0..=rest.len());
                genome = rest.splice(self, at, at, &piece);
            }
        }
        genome
    }

    /// Transfert horizontal : un fragment du donneur est inséré dans le receveur (la longueur reste
    /// bornée : au besoin il remplace un fragment de même taille).
    pub(crate) fn transfer(&self, recipient: &Genome, donor: &Genome, rng: &mut impl rand::Rng) -> Genome {
        let run = rng.random_range(1..=self.mutation().run_max.min(donor.len()));
        let from = rng.random_range(0..=donor.len() - run);
        let fragment = donor.slice(from, from + run);
        let at = rng.random_range(0..=recipient.len());
        if recipient.len() + run <= self.mutation().max_length {
            recipient.splice(self, at, at, &fragment)
        } else {
            let at = at.min(recipient.len() - run);
            recipient.splice(self, at, at + run, &fragment)
        }
    }

    /// Recombinaison : début du génome de `a`, fin de celui de `b`, coupés à des points homologues
    /// (même proportion de leur longueur). Les fragments d'un parent restent partagés avec lui.
    pub(crate) fn recombine(&self, a: &Genome, b: &Genome, rng: &mut impl rand::Rng) -> Genome {
        if a.len() < 2 || b.is_empty() {
            return a.clone();
        }
        let cut = rng.random_range(1..a.len());
        let other = (cut * b.len() / a.len()).min(b.len());
        a.splice(self, cut, a.len(), &b.slice(other, b.len()))
    }

    /// Fusion : le génome de `b` (borné par la longueur maximale) est ajouté à la suite de celui de `a`.
    #[allow(dead_code)] // Point d'extension : le catalogue de départ ne définit pas de mécanisme de fusion cellulaire.
    pub(crate) fn fuse(&self, a: &Genome, b: &Genome) -> Genome {
        let room = self.mutation().max_length.saturating_sub(a.len());
        a.splice(self, a.len(), a.len(), &b.slice(0, room.min(b.len())))
    }
}
