use crate::definitions::{AncestorDefinition, BiologyRules, FounderDefinition, GeneticsDefinition, PartDefinition};
use crate::genetics::{Genetics, Genome};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpeciesDefinition {
    id: Uuid,
    name: String,
    /// Génome de l'ancêtre : il ne fixe que le point de départ, la suite relève de l'évolution.
    ancestor: AncestorDefinition,
    /// Motifs, règles d'expression, mutations et coûts : toute la génétique du jeu.
    genetics: GeneticsDefinition,
    rules: BiologyRules,
    /// Composants visibles et leurs conditions d'apparition : la morphologie en 3D, en données.
    #[serde(default)]
    parts: Vec<PartDefinition>,
    /// Cellules présentes au départ ; vide : une seule, fille de l'ancêtre.
    #[serde(default)]
    founders: Vec<FounderDefinition>,
    /// Règles compilées, construites au premier usage et partagées par toutes les cellules.
    #[serde(skip)]
    compiled: OnceLock<Arc<Genetics>>,
}

impl SpeciesDefinition {
    pub(crate) fn id(&self) -> Uuid {
        self.id
    }
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn rules(&self) -> &BiologyRules {
        &self.rules
    }

    /// Prépare une modification : les règles compilées seront reconstruites.
    pub(crate) fn edit(&mut self) -> &mut Self {
        self.compiled = OnceLock::new();
        self
    }

    pub(crate) fn name_mut(&mut self) -> &mut String {
        &mut self.name
    }

    pub(crate) fn ancestor_mut(&mut self) -> &mut AncestorDefinition {
        &mut self.ancestor
    }

    pub(crate) fn genetics_mut(&mut self) -> &mut GeneticsDefinition {
        &mut self.genetics
    }

    pub(crate) fn rules_mut(&mut self) -> &mut BiologyRules {
        &mut self.rules
    }

    pub(crate) fn parts_mut(&mut self) -> &mut Vec<PartDefinition> {
        &mut self.parts
    }

    pub(crate) fn founders_mut(&mut self) -> &mut Vec<FounderDefinition> {
        &mut self.founders
    }

    pub(crate) fn founders(&self) -> &[FounderDefinition] {
        &self.founders
    }

    /// Génome d'un groupe de fondateurs.
    pub(crate) fn founder_genome(&self, group: &FounderDefinition) -> Result<Genome, String> {
        let genetics = self.genetics();
        let mut bytes = match &group.sequences {
            Some(texts) => AncestorDefinition::parse(texts)?,
            None => self.ancestor.bytes()?,
        };
        for motif in &group.add_motifs {
            bytes.push(genetics.carrier(motif).ok_or_else(|| format!("Motif inconnu : {motif}."))?);
        }
        let limits = genetics.mutation();
        if bytes.len() < limits.min_length || bytes.len() > limits.max_length {
            return Err("Génome de fondateurs hors des bornes de longueur.".into());
        }
        Ok(Genome::from_bytes(genetics, &bytes))
    }

    pub(crate) fn parts(&self) -> &[PartDefinition] {
        &self.parts
    }

    pub(crate) fn genetics_definition(&self) -> &GeneticsDefinition {
        &self.genetics
    }

    /// Règles génétiques compilées. Le catalogue a été validé avant tout usage.
    pub(crate) fn genetics(&self) -> &Arc<Genetics> {
        self.compiled.get_or_init(|| {
            Arc::new(Genetics::compile(&self.genetics).expect("catalogue génétique validé"))
        })
    }

    /// Génome de l'ancêtre, dont les fragments sont partagés par toutes les cellules fondatrices.
    #[cfg(test)]
    pub(crate) fn ancestor_genome(&self) -> Genome {
        let bytes = self.ancestor.bytes().expect("ancêtre validé");
        Genome::from_bytes(self.genetics(), &bytes)
    }

    /// Longueur de référence du génome (celle de l'ancêtre), qui fixe l'entretien de base.
    pub(crate) fn reference_length(&self) -> usize {
        self.ancestor.bytes().map_or(0, |bytes| bytes.len())
    }

    /// Vérifie la génétique et l'ancêtre sans rien supposer : compile les règles, lit l'ancêtre.
    pub(crate) fn validate_genetics(&self) -> Result<(), String> {
        let genetics = Genetics::compile(&self.genetics)?;
        let bytes = self.ancestor.bytes()?;
        let mutation = genetics.mutation();
        if bytes.len() < mutation.min_length || bytes.len() > mutation.max_length {
            return Err("La longueur de l'ancêtre doit respecter les bornes de longueur du génome.".into());
        }
        for group in &self.founders {
            self.founder_genome(group)?;
            if !(0.0..=1.0).contains(&group.energy) || group.energy <= 0.0 {
                return Err("Réserve initiale des fondateurs hors de ]0, 1].".into());
            }
        }
        self.validate_parts()
    }

    /// Les composants doivent avoir des dimensions, un nombre et des variantes cohérents.
    fn validate_parts(&self) -> Result<(), String> {
        for part in &self.parts {
            let sane = part.size.iter().flatten().all(|v| v.is_finite() && *v > 0.0)
                && part.reach.is_finite()
                && part.reach > 0.0
                && part.count.per_level.is_finite()
                && part.count.per_level > 0.0
                && part.count.max >= 1
                && part.appears_at.is_finite()
                && part.variants.windows(2).all(|pair| pair[0].from <= pair[1].from)
                && part.variants.iter().all(|v| v.from.is_finite() && v.scale.is_finite() && v.scale > 0.0)
                && part.when.iter().all(|range| range.at_least.is_finite());
            if !sane || part.name.trim().is_empty() {
                return Err(format!("Composant « {} » invalide.", part.name));
            }
        }
        Ok(())
    }
}
