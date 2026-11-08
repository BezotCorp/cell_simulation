# Génome, expression et phénotype

Un organisme porte un **génome** : une suite ordonnée de **séquences de 8 bits**. Des **motifs**
binaires, cherchés dans ces séquences, déterminent ce qu'il sait faire. Leur présence, leur nombre,
leur ordre, leur position et leurs combinaisons donnent des **quantités de produits** ; des lois
physiques en déduisent le **phénotype** : forme, taille, réserves, capacités, dépenses. Tout ce qui
relève de la génétique du jeu (motifs, règles, mutations, coûts) est défini dans
`data/definitions.ron` : le moteur n'en connaît aucun.

```text
Génome ─ motifs ─▶ Interpretation ─ milieu ─▶ Levels (15 produits) ─▶ Morphology (naissance) ─▶ Anatomy ─▶ rendu
(séquences)         (par génome)      (signaux)                        └▶ Physiology (chaque seconde)
```

## Génome

- `Genome` est une liste de fragments immuables (`Chunk`, quelques séquences chacun) **partagés** :
  cloner un génome ne copie rien, modifier en crée un nouveau qui réutilise tous les fragments
  intacts. Une division sans mutation donne à la fille le _même objet_ que sa mère.
- Un registre de fragments (`ChunkPool`, dans `Genetics`) dédoublonne ce qui est identique : deux
  fragments de mêmes octets ne sont qu'un objet, avec une seule interprétation. Il ne retient aucun
  fragment (références faibles).
- L'**identité génétique** (`Genome::id`) est l'empreinte des séquences, indépendante du découpage.

## Motifs et conditions (données du jeu)

Un motif est une suite de 1 à 8 bits avec une **orientation** : `Forward` (comme écrit), `Reverse`
(la fenêtre lue à l'envers) ou `Both`. Il est cherché dans **chaque séquence** (pas à cheval sur deux)
à toutes les positions : les occurrences peuvent se chevaucher et plusieurs motifs peuvent partager des bits.

Une **capacité** (un produit) somme des **contributions**. Chacune a un `amount` et des conditions qui
doivent toutes être vraies :

| Condition      | Vraie quand…                                                                                 |
| -------------- | -------------------------------------------------------------------------------------------- |
| `Count`        | le motif apparaît au moins `at_least` fois (et au plus `at_most` : un répresseur)            |
| `SameSequence` | plusieurs motifs se trouvent dans une même séquence (une combinaison)                        |
| `Ordered`      | une occurrence de `first` précède une occurrence de `then` (l'ordre compte)                  |
| `Repeated`     | au moins `copies` séquences complètes identiques portent le motif (une duplication)          |
| `Within`       | le motif apparaît dans les `sequences` premières séquences (la position compte)              |
| `Signal`       | le milieu (lumière, chaleur, eau, nourriture, densité, énergie, maturité) est dans une plage |

`per_copy` multiplie l'effet par le nombre de copies d'un motif, borné (une dose). Une même capacité
a ainsi une **forme primitive** (un motif courant) et des **formes développées** (copies
supplémentaires, ordre, combinaison, milieu). Le jeu de départ de `definitions.ron` en donne des
exemples : le métabolisme monte par paliers à chaque copie, les pigments demandent deux motifs
ordonnés _et_ de la lumière pour leur forme complète, la prédation une combinaison dans une séquence.

Les produits sont des canaux du moteur (`Product`) : métabolisme, croissance, fécondité, réparation,
hydratation, cuticule, tissu nerveux, digestion, pigments, thermosynthèse, prédation, appareil moteur,
adhésion, conjugaison, recombinaison. Le **réseau neuronal** a son gain et son biais dans les données,
et son capteur se choisit par le premier motif de capteur présent le long du génome.

## Interprétation partagée

- La table `ByteProfile` donne, pour chacune des 256 séquences possibles, les motifs qu'elle contient
  (nombre, première et dernière positions) : interpréter un fragment est une lecture de table.
- L'analyse d'un `Chunk` est mémorisée **dans le fragment** : tous les génomes qui le partagent la
  partagent. Une mutation ne fait recalculer que les fragments touchés.
- L'`Interpretation` d'un génome (occurrences, positions, combinaisons, séquences répétées, quantités
  qui ne dépendent pas du milieu) est calculée **une fois** et partagée par toutes les cellules qui
  le portent. Seules les contributions à condition de milieu sont réévaluées, une fois par seconde de vie.
- Au chargement d'une sauvegarde, les génomes distincts sont interprétés d'avance en parallèle sur
  plusieurs fils d'exécution (au-delà de 256 génomes) ; le résultat est identique à celui d'une
  interprétation séquentielle.

## Phénotype

**Morphologie** (développée dans le milieu d'acclimatation, voir ci-dessous) : taille `0,6 + 0,8 × Croissance`,
réserve maximale `100 × taille³`, unités nerveuses `⌊8 × Tissu nerveux⌋`, anatomie.

**Développement tout au long de la vie.** Le milieu d'acclimatation est posé à la naissance (lumière,
chaleur, eau, nourriture du site de naissance), puis se rapproche du milieu vécu au rythme de
1/(10 × maturation) par seconde. Chaque seconde, le génome est ré-exprimé dans ce milieu ; si un produit a
changé de plus de 0,05, le corps (taille, réserves, tissu nerveux, anatomie) est redéveloppé. Deux cellules
de même génome divergent donc selon leur milieu, un séjour bref ne laisse rien, un séjour long laisse une
forme durable qui ne s'efface que lentement (testé : pigment développé à la lumière, perdu après un long
séjour à l'ombre, retrouvé au retour de la lumière). La physiologie, elle, suit le milieu instantané.

**Physiologie** (ré-exprimée chaque seconde de vie avec le milieu courant) :

| Grandeur                      | Loi                                                                                                                                                                                         |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| métabolisme                   | `0,5 × 2^(2 × Métabolisme)` par seconde                                                                                                                                                     |
| entretien                     | métabolisme × (1 + inadaptation au milieu + produits) × `(0,4 + 0,6 × taille^2,25)`                                                                                                         |
| coût des produits             | écart à l'ancêtre × (Réparation 0,10 ; Digestion 0,04 ; Pigments, Thermosynthèse, Prédation 0,20 ; Moteur 0,06 ; Adhésion 0,08 ; Conjugaison, Recombinaison 0,05) + 0,03 par unité nerveuse |
| entretien du génome           | `0,004` × (longueur − longueur de l'ancêtre), en part du métabolisme                                                                                                                        |
| adaptation eau / terre        | Hydratation / Cuticule                                                                                                                                                                      |
| photosynthèse, thermosynthèse | produit × surface (`taille²`)                                                                                                                                                               |
| prédation                     | Prédation × taille                                                                                                                                                                          |
| nage contre le courant        | `0,12 × Hydratation × métabolisme × (0,3 + 0,7 × Moteur) / √taille`                                                                                                                         |
| division                      | à `capacité × (0,35 + 0,6 × Fécondité)`, après paiement de la réplication                                                                                                                   |

## Hérédité et mutations

À la division, la mère paie la **réplication** de son génome (`0,04` par séquence, majorée de 100 %
par unité de réparation : une copie fidèle coûte plus), puis la fille reçoit une copie à laquelle
s'appliquent des événements tirés au hasard, sans regard sur leur utilité :

- **erreurs de copie** : chaque bit peut être inversé (taux par bit) ;
- **duplication**, **suppression**, **insertion** de séquences aléatoires, **inversion** (ordre des
  séquences, et parfois des bits de chacune : les motifs changent de sens), **déplacement** d'un fragment.

Tous les taux (données du jeu) sont multipliés par l'**exposition** du site de la mère
(`1 + 0,6 × lumière + 0,8 × chaleur/30` : rayonnement ultraviolet et chaleur) et divisés par
`1 + 4 × Réparation`. Plus de naissances donne plus de mutations ; un génome plus long offre plus de
bits à copier de travers. Les bornes de longueur (4 à 96 séquences dans le jeu de départ) sont des
données. Un génome plus long coûte aussi plus d'entretien et de réplication : la sélection arbitre.

## Échanges de matériel génétique

- **Conjugation** (produit du génome) : un organisme lié à un voisin lui transfère un fragment de
  son génome, au prix d'énergie (`Genetics::transfer`).
- **Recombinaison** (produit du génome) : à la division, le génome de la fille mêle celui de la mère
  et celui d'une cellule liée, coupés à des points homologues (`Genetics::recombine`).
- **Fusion** : `Genetics::fuse` concatène deux génomes ; aucun mécanisme de fusion cellulaire n'existe
  dans le jeu de départ, l'opération est disponible pour les motifs et règles qui la définiront.

Les fragments transférés ou recombinés restent partagés avec leurs génomes d'origine. Chaque cellule
garde **son propre génome**, y compris au sein d'un organisme pluricellulaire (cellules liées) : leurs
différences se conservent et se comptent (`Simulation::multicellular`).

## Rendu

Le génome ne contient aucun chemin d'asset et la biologie ne dépend pas du moteur graphique. Le
rendu lit `Anatomy`, déduite des produits exprimés (vacuoles, chloroplastes, thermophores, épines,
flagelles, cils, plaques, ganglions, zones d'adhésion, pili) et assemble les composants
(`world_scene/organism_assembly.rs`) :

- `data/assets/library.ron` associe un emplacement (`flagellum`, `spine`, `chloroplast`, …) à des
  modèles GLB, relatifs à `data/assets/` ; la variante (0 à 3) est choisie d'après le développement du
  composant ;
- un modèle est orienté (axe +Y vers l'extérieur du corps), mis à l'échelle (modèle de taille unitaire)
  et lié à la membrane ; l'organisme entier est déformé selon sa taille et sa mobilité ;
- sans modèle chargé pour un emplacement, une forme procédurale le remplace ;
- la finesse dépend de la population : anatomie complète sous 300 organismes, noyau et deux familles
  les plus développées sous 2 500, membrane seule au-delà.

## Sauvegardes

Un seul format (version 1) : en-tête (magie, version, empreinte du catalogue, longueur) puis la session
entière, génomes compris. Les sauvegardes d'autres formats sont refusées. L'empreinte du catalogue couvre
l'identité, les règles de l'espèce, la génétique et le monde, pas le génome de l'ancêtre.

## Performances

Voir les mesures dans [docs/performance-genetics.md](performance-genetics.md).
