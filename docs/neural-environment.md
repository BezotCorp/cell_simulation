# Hérédité neuronale et interactions spatiales

## Hérédité

Une division copie les gènes, les connexions et le biais de la mère.
Les mutations s'appliquent uniquement à la fille, avec le générateur de la simulation : une même graine rejoue aussi les mutations neuronales.
L'activation instantanée est remise à zéro ; ce n'est pas un caractère héréditaire.

Le capteur actuel est la température. Le réseau primitif conserve son poids de 0,8 et son biais de −20, donc son seuil de 25 °C.
L'activation est calculée aussi pour ce réseau primitif et peut être observée dans l’inspecteur de la cellule sélectionnée.
La faim reste un réflexe physiologique indépendant, pour préserver la recherche de nourriture même sans activation thermique.

La complexité compte les unités de calcul, hors capteur. Le graphe est une chaîne orientée sans cycle, avec propagation linéaire intermédiaire et `tanh` en sortie. La propagation s'effectue sans tampon intermédiaire, avec le même résultat binaire que l'accumulation par nœud. Aucune borne arbitraire n'est imposée à la complexité : seule la cohérence entre complexité et connexions est vérifiée à la restauration.
Une mutation de complexité ajoute ou retire une unité, sans passer sous un.
L'ajout insère une connexion de gain un : agrandir le réseau ne détruit pas à lui seul le signal hérité.
La réduction supprime les connexions vers les unités retirées, sans laisser d'indices invalides.
Les poids et le biais mutent ensuite selon la probabilité portée par les gènes, avec les amplitudes déjà présentes dans le modèle (±0,2 et ±1 respectivement).

## Monde colonisable, sites et habitats

Trois notions sont distinctes :

- **le site** est la maille de la grille technique. Il indexe les ressources, la température et le calcul de la compétition alimentaire ; il porte une identité persistante (UUID v7) pour la sélection et la reprise. Il n'enferme personne ;
- **les conditions** (substrat, température, eau et courants du cycle de l'eau, nourriture) sont des champs sur le monde. La température est interpolée au point exact de l'organisme, sans palier à la frontière de deux sites ;
- **l'habitat réel** est l'étendue contiguë de sites de même milieu (eau ou terre) et de même traversabilité. Il est mesuré à la demande (inspecteur) et n'est jamais une case prédéfinie.

Un organisme possède une position continue dans un monde torique. Le site qui le contient et son décalage dans ce site forment cette position ; le décalage est écrit dans une section dédiée du fichier BIN (version 4).
Le modèle ne possède ni volume ni rayon physique des organismes : lui ajouter une exclusion « une cellule par case » introduirait une capacité sans fondement dans ce modèle.
La disponibilité alimentaire limite les habitants sans plafond d'effectif.

Les roches et les murs sont solides et infranchissables : c'est la seule restriction physique de mouvement.
Le sable et le sol sont des substrats traversables, même si le conteneur historique de ces matériaux s'appelle `Entity::Obstacle`.
Les sources thermiques, l'eau et les ressources restent accessibles ; leurs autres effets demeurent ceux du cycle de vie.
Aucun milieu n'est interdit à un organisme mal adapté : il peut s'y aventurer, en subir le surcoût d'entretien (voir `water_adaptation` et `land_adaptation`), s'y adapter par mutation au fil des générations ou y mourir.
Le fondateur apparaît sur le site accessible le plus proche du centre, sans supprimer d'obstacle.
S'il n'existe aucun site accessible, aucune cellule n'est créée.

Une décision de déplacement (au plus une par seconde simulée) évalue huit caps régulièrement répartis et d'orientation aléatoire, à une portée d'un site : aucune direction n'est privilégiée.
Le trajet ne traverse pas de solide ; la position actuelle et son site ne peuvent pas devenir une destination.
Sans destination, le coût de déplacement n'est pas prélevé ; le métabolisme continue normalement.
Une fille naît à moins d'un demi-site de sa mère, dans n'importe quelle direction (le site voisin n'est pas interdit ; un solide la ramène auprès de sa mère) : l'espèce gagne le voisinage de proche en proche, sans plafond ni repeuplement artificiel.

Pour chaque destination possible, le moteur estime l'énergie accessible avant la décision suivante : réserve présente + renouvellement d'une seconde.
Cette énergie est rapportée à l'entretien total des occupants, en ajoutant le candidat.
L'estimation emploie le taux d'entretien actuel du candidat ; les coûts réels continuent d'utiliser l'environnement de la cellule à chaque pas.

L'ordre de décision est mélangé pour ne pas avantager une lignée.
Les arrivées et les départs sont réservés au fil des décisions : les cellules suivantes voient leur effet sur l'entretien du site.
Cette réservation ne bloque pas une case occupée ; elle en estime le budget alimentaire.

Une cellule reste sur place si son habitat couvre son entretien collectif et si aucun voisin n'offre un meilleur rapport.
Si l'entretien n'est plus couvert, l'exploration demeure possible, même vers un voisin encore vide.
Ces règles évitent à la fois l'abandon coûteux d'une réserve viable et la migration de toute une famille vers un site déjà surchargé.
Les égalités entre destinations sont départagées aléatoirement.

## Compétition alimentaire

Tous les déplacements précèdent l'ingestion. À chaque site, on somme les déficits énergétiques des occupants vivants puis on retire une seule fois la quantité réellement disponible.
Chaque cellule reçoit la même fraction de son déficit :

`ingestion = déficit × min(1, nourriture disponible / somme des déficits)`

L'ordre du vecteur ne crée aucune priorité alimentaire. Une cellule pleine ne prélève rien ; une cellule morte ne reçoit rien et ne se reproduit pas.
Les naissances ont lieu après le partage et les filles commencent à se nourrir au pas suivant.
Une fille naît près de sa mère avec la moitié de son énergie ; elle participe ensuite à la compétition du site où elle se trouve.

Le rendu place chaque organisme à sa position réelle ; la taille des individus diminue doucement dans les sites très peuplés, sans jamais en masquer un.
L'inspecteur expose la position, le site technique, l'habitat réel, les gènes, l'âge et les paramètres neuronaux.

## Présentation Fyrox et définitions partagées

Le réseau ancestral (poids thermique et biais) est défini dans le catalogue RON, avec le génome et les règles énergétiques.
Une modification du catalogue démarre une nouvelle expérience ; les checkpoints restent liés à leur catalogue d’origine.
Les déplacements, la compétition et l’hérédité utilisent le même modèle que les expériences sans fenêtre.
La scène Fyrox ne contient ni corps ni collider : organismes et habitats sont sélectionnés par un lancer de rayon analytique (plan du biotope, sphères des organismes). La représentation ne modifie pas la compétition biologique.

Le sable, la terre et les habitats vides sont des couleurs du sol ; les roches (amas de blocs) et murs ont une géométrie volumique, les ressources une représentation et un matériau propres (baies, bulles, cristaux, flaques).
Les placements RON peuvent instancier tous les types existants, notamment murs et minéraux.
Aucun métabolisme minéral nouveau n’est introduit. Les nouvelles ressources et les naissances sont répercutées dans la scène ; les morts retirent leur représentation.

## Validation

Les tests unitaires restent rattachés aux domaines via leurs `mod.rs`, sans tests dans les fichiers de production.
Les tests d’intégration dans `tests/` vérifient le lancement sans fenêtre, la reprise via la CLI publique et le refus des sauvegardes invalides.
Aucun élément n’est rendu public uniquement pour tester.

```sh
cargo check --locked --offline --all-targets
cargo test --locked --offline
```

Le scénario prolongé conserve 16 graines, deux heures par graine et le pas de 16 ms.
Il contrôle la survie à chaque pas, l’enveloppe de croissance, des naissances dans la deuxième heure à l’échelle de l’ensemble et une variation neuronale héritée.
Le modèle ne force ni mortalité ni naissance pour rompre un équilibre sans surplus.
Un test distinct vérifie deux divisions successives lorsque les ressources le permettent.

La vérification graphique s’effectue sur le binaire lancé normalement, depuis l’extérieur du programme.
Le runtime ne contient aucun scénario de test ni commande de vérification automatisée.
Les opérations de caméra, de pause, de vitesse et de sélection modifient leurs états via des méthodes dédiées ;
les champs sont privés et aucun accesseur ne renvoie une référence mutable.

### Résultats de migration

La compilation de toutes les cibles est sans avertissement. Les 46 tests unitaires et les deux tests d’intégration passent.
Le scénario prolongé retrouve les effectifs finaux de 60 à 86 cellules et les pics de 130 à 168 cellules, sans extinction sur les 16 graines.
Les graines 3 et 13 atteignent un équilibre sans naissance dans la deuxième heure ; les autres continuent de se reproduire.

La fenêtre native et une population développée sont inspectées visuellement sur le binaire normal.
La sauvegarde de fermeture et sa reprise sont vérifiées par la CLI, indépendamment des tests unitaires.
