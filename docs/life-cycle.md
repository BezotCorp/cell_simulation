# Cycle de vie et budget énergétique

Le moteur avance au pas fixe de 16 ms. Le rendu ne détermine ni la consommation, ni l'âge, ni le rythme des déplacements.
Le domaine reste réparti entre `Cell` (physiologie), `CellGenes` (hérédité), `Grid` (ressources et voisinage) et `Simulation` (ordre des opérations et naissances/morts).

Les valeurs ci-dessous sont les valeurs du catalogue `data/definitions.ron`.
Les instances partagent ces définitions ; les checkpoints stockent leurs états, pas une copie du catalogue par organisme.

## Paramètres et règles

- Réserve cellulaire : 100 unités d'énergie ; métabolisme ancestral : 1 unité/s.
- La pénalité d'adaptation multiplie ce métabolisme, comme précédemment.
- Maturité et délai entre divisions : 30 secondes simulées, borne incluse.
- Une division partage l'énergie disponible à parts égales. Elle ne crée pas d'énergie ; la fille commence à l'âge zéro et hérite des gènes mutés.
- La faim déclenche la recherche de nourriture même en eau froide.
  Au maximum un déplacement par seconde, vers un voisin traversable offrant un meilleur budget alimentaire par unité d'entretien, ou pour explorer un habitat insuffisant.
  Le bouclage de la grille est conservé.
- Un déplacement effectif coûte l'équivalent d'une seconde de métabolisme. Passer de 5 à 1 unité pour le génotype ancestral réduit le coût d'exploration :
  le premier essai à 5 unités perdait 4 fondateurs sur 8 graines en 30 minutes.
- Chaque site alimentaire stocke au plus 20 unités, le repas déjà présent dans le modèle.
  Il se renouvelle en 10 secondes (2 unités/s), indépendamment de l'effectif des cellules. Un site épuisé reste un site alimentaire.
- L'ingestion retire au plus la somme des déficits des occupants. Si la réserve est insuffisante, chaque occupant reçoit la même fraction de son déficit.
- L'ordre des décisions de déplacement est mélangé à chaque pas ; les arrivées et départs sont réservés. L'ingestion ne donne pas de priorité liée à cet ordre.
- L'eau et l'oxygène sont placés à l'initialisation. Leur ajout répété remplissait définitivement les cases et empêchait le renouvellement alimentaire.
  Les tentatives existantes de colonisation alimentaire des cases vides continuent (5 positions, probabilité de 10 % par seconde).

L'entretien minimal est fixé à 0,5 unité/s (la moitié du taux ancestral). C'est une hypothèse explicite de ce modèle simplifié, pas une constante biologique.
Elle empêche l'évolution vers un organisme sans aucune dépense énergétique.
Le seuil de division reste compris entre l'énergie nécessaire à deux cellules à cet entretien minimal pendant une maturation (30 unités), et la capacité cellulaire (100 unités).
Les probabilités restent dans [0, 1], les adaptations dans [0, 1] et la complexité neuronale reste positive, sans plafond.

La régulation vient des ressources finies, du métabolisme et de la famine.
Aucun plafond d'effectif, repeuplement automatique ou apport de nourriture conditionné à la population n'est appliqué.
La température continue d'influencer le neurone ; la faim permet aussi de chercher de la nourriture.

## Vérification et limites

Les tests unitaires vérifient séparément le métabolisme, la faim, le mouvement, l'ingestion, le renouvellement, les bornes des mutations, la maturité et la conservation d'énergie à la division.
Les tests de `Simulation` vérifient son comportement sur la durée et la reproductibilité par graine.
Ce sont des tests du module, pas des tests d'intégration utilisant des accès privés détournés. Aucune API n'est rendue publique pour permettre les tests.

Le scénario prolongé utilise une grille de 10 × 10, un fondateur, les graines 0 à 15 et deux heures simulées par graine (450 000 pas).
Il exige une population non vide à chaque pas et un effectif sous l'enveloppe de régression de 400 cellules.
Cette enveloppe correspond au débit maximal théorique de 100 sites à 2 unités/s, divisé par l'entretien minimal ;
elle n'est pas une limite imposée à l'exécution ni une preuve mathématique de borne instantanée en présence de réserves accumulées.

```sh
cargo check --locked --offline
cargo test --locked --offline
# Vérifications courtes, sans le scénario prolongé :
cargo test --locked --offline -- --skip population_remains_viable_with_finite_resources
```

La survie n'est pas garantie pour toute graine, taille de grille ou durée. Une grille dépourvue de nourriture reste mortelle.
L'essai prolongé couvre aussi le passage eau/terre autour de 6 931 secondes, mais ne constitue pas une validation écologique universelle.
L'hérédité neuronale, le partage des habitats, les déplacements et l'audit des avertissements sont décrits dans [neural-environment.md](neural-environment.md).

Les résultats de validation actualisés sont consignés dans [neural-environment.md](neural-environment.md).
La survie n'est pas forcée et les naissances ne sont pas imposées à chaque population pendant un intervalle donné.

## Performances sans effet sur les résultats

Les optimisations du domaine ne modifient ni l'ordre des tirages aléatoires, ni les additions flottantes, ni les règles :
une graine donne exactement les mêmes populations, mutations et énergies qu'avant (vérifié sur plusieurs graines jusqu'à 150 000 pas, avec reprise de sauvegardes de format 1).
La filiation est conservée intégralement ; ses index (identité → position) et le compteur de décès évitent les recherches linéaires.
La régénération parcourt la liste des sites alimentaires, la chaleur d'une source ne balaie que son rayon d'action, la propagation neuronale n'alloue plus, et les tampons de compétition et d'entretien sont réutilisés d'un pas à l'autre.
