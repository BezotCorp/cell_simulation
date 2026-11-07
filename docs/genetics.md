# Génome, expression et phénotype

Un organisme n'a plus de paramètres directs (métabolisme, adaptation…). Il porte un **génotype**
héritable ; son **expression** dans un milieu fabrique des **produits** ; des **lois physiques** en
déduisent son **phénotype** : forme, taille, réserves, capacités, dépenses. Chaque étape est un type
distinct et la suivante ne lit jamais le génome directement.

```text
Génome ──(signaux du milieu)──▶ Levels (13 produits, 0…1) ──▶ Morphology  (à la naissance)
  gènes, câblage neuronal              expression                └▶ Anatomy → rendu
                                                ──▶ Physiology (chaque seconde de vie)
```

## Génotype (`genetics`)

Un **gène** (`Gene`) est un petit objet entier (résolution 1/1024, hérédité exacte, ≈ 64 octets) :

- une **expression de base** (`bias`) ;
- des **sensibilités** (`inputs`) à sept _signaux_ du milieu (lumière, chaleur, eau, nourriture,
  densité, énergie, maturité) et aux treize _produits_ déjà fabriqués ;
- des **effets** (`effects`) sur chacun des treize produits.

Son activité vaut `clamp(0,5 + base + Σ sensibilité × entrée, 0, 1)` : constante sans sensibilité (une
expression de base de 0,5 en fait un gène **constitutif**, actif sans signal), interrupteur ou gradient
sinon. Les
effets d'un gène sont `activité × effet` et **s'additionnent** : plusieurs gènes contribuent à un
produit (**polygénie**), un gène agit sur plusieurs produits (**pléiotropie**), une copie de plus d'un
gène double son effet (**dosage**). Les gènes se lisent les uns les autres par les produits : une
cascade de régulation se propage en trois passes (`Genome::express`).

Le génome (`Genome`) est une liste de 1 à 24 gènes et le câblage du réseau neuronal (poids de huit
unités, biais, capteur). Il ne dit ni la taille, ni la mobilité, ni la couleur.

## Produits (`Product`)

Métabolisme · Croissance · Fécondité · Réparation · Hydratation · Cuticule · Tissu nerveux ·
Digestion · Pigments · Thermosynthèse · Prédation · Appareil moteur · Adhésion. Une quantité est
toujours dans `[0, 1]`. Ajouter un produit = une variante + ses effets dans les lois ci-dessous.

## Expression : trois niveaux

1. **Développement** : à la naissance, les signaux du _milieu de naissance_ (conservés dans le fichier
   de sauvegarde) fixent la **morphologie** pour la vie.
2. **Physiologie** : à chaque seconde de vie, les gènes sont ré-exprimés avec les signaux _courants_ ;
   les capacités et les dépenses suivent le milieu (un gène sensible à la lumière produit des pigments
   le jour, pas la nuit).
3. **Sauvegarde** : le génome, le milieu de naissance et les produits courants suffisent ; forme et
   fonctionnement sont reconstruits.

## Phénotype

**Morphologie** (`Morphology`) : taille `0,6 + 0,8 × Croissance` (diamètres de l'ancêtre), réserve
maximale `100 × taille³`, unités nerveuses `⌊8 × Tissu nerveux⌋`, anatomie (ci-dessous).

**Physiologie** (`Physiology`) :

| Grandeur                      | Loi                                                                                                                                                                                      |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| métabolisme                   | `0,5 × 2^(2 × Métabolisme)` par seconde                                                                                                                                                  |
| entretien                     | métabolisme × (1 + inadaptation au milieu + produits) × `(0,4 + 0,6 × taille^2,25)`                                                                                                      |
| coût des produits             | écart à l'ancêtre × (Réparation 0,10 ; Digestion 0,04 ; Pigments, Thermosynthèse, Prédation 0,20 ; Moteur 0,06 ; Adhésion 0,08) + 0,03 par unité nerveuse + 0,01 par gène au-delà de six |
| adaptation eau / terre        | Hydratation / Cuticule                                                                                                                                                                   |
| ingestion                     | Digestion                                                                                                                                                                                |
| photosynthèse, thermosynthèse | produit × surface (`taille²`) : un grand corps capte plus                                                                                                                                |
| prédation                     | Prédation × taille                                                                                                                                                                       |
| nage contre le courant        | `0,12 × Hydratation × métabolisme × (0,3 + 0,7 × Moteur) / √taille`                                                                                                                      |
| division                      | à `capacité × (0,35 + 0,6 × Fécondité)`                                                                                                                                                  |

La taille crée de vrais compromis : un grand organisme stocke plus (volume), capte plus (surface),
mais coûte davantage, nage moins bien et met plus de temps à remplir ses réserves ; un petit coûte
moins mais ne tient pas une nuit ni un courant. Le socle fixe de l'entretien et le coût de chaque
gène empêchent les corps minuscules et les génomes gonflés d'être gratuits.

**Anatomie** (`Anatomy`) : une liste ouverte de composants (`PartKind`) déduits des produits qui les
justifient — vacuoles (croissance), chloroplastes (pigments), thermophores, épines (prédation),
flagelles (moteur), cils (hydratation), plaques (cuticule), ganglions (tissu nerveux), zones d'adhésion.
Le nombre d'exemplaires croît avec le développement et la taille. Ce que l'on voit est donc ce qui
agit : c'est la même quantité de produit qui fait fonctionner et qui dessine le composant.

## Mutations

À la division, la fille reçoit une copie du génome de sa mère à laquelle s'appliquent des mutations
en nombre aléatoire, de moyenne

`λ = 0,05 × (nombre de gènes) × exposition / (1 + 4 × Réparation)`

- **Plus de naissances, plus de mutations** : le tirage a lieu à chaque division.
- **Plus de gènes, plus de cibles** : un génome long mute davantage.
- **Exposition** (`1 + 0,6 × lumière + 0,8 × chaleur/30`) : le rayonnement ultraviolet (la lumière non
  filtrée par l'eau ni les nuages) et la chaleur abîment l'ADN ; la Réparation, produit comme les
  autres et coûteux, l'atténue. L'exposition est celle du site de la mère.
- **Jamais orientées** : chaque mutation est tirée uniformément, sans regard sur son utilité :
  une valeur d'un gène (expression, sensibilité ou effet), le câblage du réseau, la **duplication**, la
  **perte** ou l'**apparition** d'un gène. C'est la sélection, par l'énergie, la survie et la
  reproduction, qui trie.

Des produits absents de l'ancêtre (pigments, tissu nerveux, prédation, adhésion…) apparaissent par
duplication, dérive des effets et nouveaux gènes ; des combinaisons inédites donnent des anatomies
inédites. Rien n'est prévu pour qu'une forme précise apparaisse.

## Catalogue

`species.genome` (RON) décrit l'ancêtre : six gènes constitutifs qui reproduisent la cellule thermosensible
d'origine (voir les commentaires de `data/definitions.ron`). L'empreinte du catalogue ne le couvre pas :
il ne sert qu'à créer l'ancêtre d'une nouvelle expérience.

## Sauvegardes

Version 7 : chaque organisme enregistre son génome, ses signaux de naissance et ses produits courants.
Les versions 1 à 6 (paramètres directs) sont lues puis traduites : un gène constitutif par produit
exprimé, d'effet égal à l'ancienne valeur, donne le même phénotype à la résolution du génome près ;
le réseau neuronal, les mécanismes et les liaisons sont repris. Le taux de mutation propre à chaque
organisme n'existe plus (il découle du génome et du milieu).

## Rendu

Le génome ne contient aucun chemin d'asset et la biologie ne dépend pas du moteur graphique. Le
rendu lit `Anatomy` et assemble les composants (`world_scene/organism_assembly.rs`) :

- `data/assets/library.ron` associe un emplacement (`flagellum`, `spine`, `chloroplast`, …) à des
  modèles GLB, relatifs à `data/assets/` ; la variante (0 à 3) est choisie d'après le développement du
  composant ;
- un modèle est orienté (axe +Y vers l'extérieur du corps), mis à l'échelle (modèle de taille unitaire)
  et lié à la membrane ; l'organisme entier est déformé selon sa taille et sa mobilité ;
- sans modèle chargé pour un emplacement, une forme procédurale le remplace : le jeu fonctionne sans
  aucun asset ;
- la finesse dépend de la population : anatomie complète sous 300 organismes, noyau et deux familles
  les plus développées sous 2 500, membrane seule au-delà.
