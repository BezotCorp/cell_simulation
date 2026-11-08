# Mécanismes évolutifs, coopération et orages

Le génome ne décrit plus une cellule de type unique : c'est une suite de séquences binaires dont les motifs, lus dans un
milieu donné, fabriquent des **produits** (voir [genetics.md](genetics.md)). Rien n'est une catégorie : une « plante » serait seulement une
lignée qui a retenu la photosynthèse et perdu la motilité ; un « animal », une lignée qui chasse.
Rien de cela n'est programmé ni garanti.

## Mécanismes : des produits du génome

Les mécanismes ne sont plus des paramètres : ce sont des **produits** que les motifs du génome déterminent
(voir [genetics.md](genetics.md)). Chacun agit sur la physiologie, coûte de l'entretien et dessine un
composant visible.

| Produit         | Effet                                                                       | Entretien (part du métabolisme par unité) |
| --------------- | --------------------------------------------------------------------------- | ----------------------------------------- |
| Digestion       | prélève les réserves alimentaires des sites                                 | 0,04                                      |
| Pigments        | captent la lumière du site (jour, nuages, profondeur et turbidité de l'eau) | 0,20                                      |
| Thermosynthèse  | capte l'excès de température au-dessus de l'ambiante                        | 0,20                                      |
| Prédation       | prélève de l'énergie sur un voisin du même site (rendement 45 %)            | 0,20                                      |
| Appareil moteur | rythme des déplacements actifs ; sous 0,05 l'organisme est sessile          | 0,06                                      |
| Adhésion        | reste lié à ses filles et partage l'énergie avec elles                      | 0,08                                      |

L'ancêtre fabrique la digestion et l'appareil moteur ; son métabolisme de base les inclut déjà, si bien
que l'entretien se mesure à l'écart avec lui. Les règles (quels motifs, combien d'occurrences, dans quel ordre, avec quel signal du milieu) sont des données de
`definitions.ron` ; les motifs et leurs positions mutent avec le génome (bits, duplications, suppressions,
insertions, inversions, déplacements).

## Pressions de sélection

- **Lumière et chaleur partagées** : chaque site n'offre qu'une capacité finie (3 organismes à
  photosynthèse pleine, 2 à thermosynthèse pleine). Au-delà, les occupants se font de l'ombre : la
  croissance est bornée par l'énergie disponible, sans plafond arbitraire d'effectif.
- **Jour et nuit** (150 s) : la photosynthèse s'arrête la nuit, les réserves comptent.
- **Eau et nuages** : l'eau profonde ou boueuse éteint la lumière, les nuages la voilent ; les
  inondations et sécheresses du cycle de l'eau déplacent les terrains favorables.
- **Courants** : ils emportent qui ne peut les remonter.
- **Prédation** : la proie perd exactement ce que le prédateur prélève, il n'en garde que 45 %.
- **Foudre** : décharge d'un nuage chargé, qui blesse les organismes proches (le double dans l'eau,
  qui conduit) et fertilise le site touché.

## Neurones facultatifs

L'ancêtre n'a aucun tissu nerveux : le produit « tissu nerveux » est nul. Une mutation de ses effets en
produit, et ses unités (`⌊8 × produit⌋`) s'activent selon le câblage du génome (gain 0,8 et biais −20, données du catalogue). Un réseau lit un seul capteur (température, lumière,
nourriture ou énergie, selon le premier motif de capteur du génome) et son activation autorise l'exploration même quand
les réserves ne l'exigent pas. Il coûte de l'énergie (0,03 par unité) : la sélection décide s'il vaut ce prix.

## Coopération

Une cellule adhésive reste liée à sa fille avec une probabilité égale à son adhésion (au plus 4
liaisons par organisme). Les organismes liés :

- **partagent leur énergie**, de la plus riche vers la plus pauvre (`0,2 × écart × √(adhésion A × adhésion B)`
  par seconde), sans aucune perte ;
- **se rappellent élastiquement** à une distance de repos de 0,45 site, sans traverser de solide ;
- **ne se chassent pas** entre eux ;
- se séparent si la distance dépasse 1,8 site (courant, mouvement), si l'un n'exprime plus l'adhésion
  ou s'il meurt.

Une division du travail n'est pas codée : un organisme sessile qui capte la lumière peut nourrir un
voisin mobile s'ils sont liés et la sélection les retient ensemble.

## Orages

L'électrification suit le cycle de l'eau (`hydrology::Storm`) :

1. la **charge** d'un site croît avec la pluie qui y tombe et la masse du nuage (collisions de gouttes),
   voyage avec le vent comme le nuage et fuit (2 %/s, bien plus vite dans l'air clair) ;
2. au-delà du **seuil de claquage** une décharge devient possible, avec une probabilité qui croît avec
   l'excès (tirage par hachage de la graine, de l'instant et du site : rejeu exact sans toucher au
   générateur de la simulation) ;
3. l'éclair frappe le **point le plus haut** du voisinage et libère 85 % de la charge ;
4. conséquences : l'eau du site est **vaporisée** (elle rejoint l'air), les organismes à moins de
   1,5 site perdent de l'énergie (décroissante avec la distance, ×2 dans l'eau) et meurent s'ils
   en sont vidés ; la foudre **fixe l'azote** : un site nu devient site alimentaire, un site
   alimentaire est regarni ;
5. le **tonnerre** est l'onde qui suit l'éclair (visuel : anneau qui s'étend depuis l'impact).

## Persistance

Version 7 du fichier BIN : chaque organisme enregistre son génome (voir [genetics.md](genetics.md)) ;
la section du vivant ne contient plus que les liaisons et la charge électrique. Les versions 1 à 6 sont
lues puis traduites en génomes. L'empreinte du catalogue ne couvre pas le génome de l'ancêtre (il ne sert
qu'à créer l'ancêtre d'une nouvelle expérience), seulement l'identité, les règles de l'espèce et le monde.
