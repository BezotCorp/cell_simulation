# Cell Lab — Fyrox 3D

Laboratoire d’évolution cellulaire en Rust. La scène, les matériaux PBR, les lumières,
la caméra scriptée et l’interface sont natifs Fyrox. La biologie repose sur un monde
torique continu : une grille technique de sites indexe les ressources et les calculs,
mais les organismes ont une position réelle et se déplacent dans toutes les directions. La sélection est un lancer de rayon analytique (plan du biotope et
sphères des organismes) : aucun corps ni collision physique n’existe dans la scène,
et rien n’impose d’exclusion entre cellules.

## Lancer

Depuis ce dossier, avec Rust et les bibliothèques système nécessaires à Fyrox :

```sh
cargo run --locked
cargo run --locked -- --seed 7
cargo run --locked -- --load runtime/session.bin
cargo run --locked -- --headless --seed 7 --steps 450000 --save runtime/experiment.bin
```

Le mode sans fenêtre exécute exactement le même modèle biologique et écrit un vrai
checkpoint BIN. `--load` restaure une expérience et `--steps` indique les pas
supplémentaires à exécuter en mode sans fenêtre.

## Manipuler l’expérience

- Clic gauche : sélectionner un organisme ou un habitat et lire l’inspecteur.
- Clic droit maintenu : orbiter ; Maj + clic droit : saisir et faire glisser le sol ; molette : zoomer.
- Espace : pause/reprise ; N : avancer de 16 ms ; F ou Centrer : cadrer la sélection, ou tout le biotope
  si rien n'est sélectionné (le cadrage tient compte des proportions de la fenêtre).
- Boutons − / + : vitesse de 0,25 à 64 ; Vue : écosystème, température, adaptation à l’eau.
  La pastille « Cadence » indique la vitesse réellement obtenue ; si la machine ne suit pas,
  elle affiche « limité » et le retard accumulé (voir *Temps et charge*).
- Graine puis Recommencer : nouvelle expérience reproductible.
- Relire RON : valider les définitions éditées et commencer une nouvelle expérience.
- Ctrl+S / Sauvegarder : checkpoint ; Ctrl+O / Charger : restaurer ce checkpoint.

Avant un redémarrage ou un rechargement RON, l’expérience courante est conservée
dans `runtime/session.previous.bin`. À la fermeture normale, une sauvegarde distincte
`runtime/session.autosave.bin` conserve le dernier état sans écraser le checkpoint manuel.
Les erreurs sont affichées dans le pied de fenêtre ; une opération échouée ne remplace
pas la scène courante. Les chemins dépendent de `--save` (voir `--help`).

## Temps et charge

La biologie avance exclusivement par pas de 16 ms. `Pacing` (module `application`) règle la
cadence d'exécution face à la machine ; trois grandeurs sont à distinguer :

- la **vitesse demandée** (× 0,25 à × 64), choisie par l'utilisateur ;
- le **retard** : temps simulé demandé mais pas encore exécuté parce que le budget de calcul d'une
  image (≈ 9 ms, adapté au coût mesuré d'un pas) était épuisé. Il est rendu à l'image suivante, borné à
  quatre fois la demande d'une image : au-delà, la machine ralentit l'expérience au lieu de
  s'endetter indéfiniment. Aucun pas n'est sauté ni comprimé, l'état biologique est identique à celui
  d'une machine rapide ; seul le rythme change. Une pause efface le retard ;
- la **vitesse effective**, mesurée sur une demi-seconde. La pastille « Cadence » affiche
  « limité · retard x s » dès qu'elle tombe sous 90 % de la demande.

Ces grandeurs dépendent de la machine : elles ne sont pas sauvegardées. Une suspension de la fenêtre de
plus de 250 ms n'est pas rattrapée. L'historique des courbes, lui, est échantillonné en temps simulé
(une fois par seconde simulée, puis condensé) : il ne dépend ni de la cadence d'affichage ni du retard.

## Données et architecture

`data/definitions.ron` est l’unique catalogue humain : espèce, génome initial,
réseau ancestral, budgets énergétiques, environnement et matériaux. Les placements
explicites permettent de définir murs, minéraux, nourriture ou sources thermiques
sans modifier les règles biologiques. Les minéraux sont des substrats observables ;
ils ne procurent pas d’énergie, comme dans le modèle biologique existant.

Les définitions sont partagées par `Arc`, les références persistantes utilisent
UUIDv7. Les cellules ne dupliquent pas les paramètres de leur espèce. Une sauvegarde
contient le modèle runtime, la filiation, l’état complet du générateur ChaCha,
l’accumulateur temporel, la sélection, la caméra, la pause et la vitesse. La scène
Fyrox et ses caches graphiques sont reconstruits à partir de cet état.

Les fichiers BIN utilisent `bincode-next` avec un en-tête versionné et une empreinte
de la partie biologique du catalogue (identité, espèce et monde). L’apparence — titre,
matériaux, rayon et espacement de rendu — n’en fait pas partie : la retoucher n’invalide
aucune sauvegarde. Le format est en version 5 : l'en-tête, le corps de la session, une section
« historique de population » (effectifs, naissances, décès et énergie moyenne au fil du temps simulé)
une section « positions fines » des organismes dans leur site, puis une section « eau »
(relief, eau, sédiments, humidité, nuages, courants : voir [le cycle de l'eau](docs/water-cycle.md)).
Les versions 1 à 4 restent lisibles, leur corps est identique : l'historique est alors reconstruit
depuis la filiation (l'énergie n'est connue qu'à partir de la restauration) et la position fine de
chaque organisme est dérivée de son identité et l'hydrologie est reconstruite d'après le niveau d'eau
global enregistré. Leur reprise est valide mais ne prolonge pas
bit à bit l'ancien déplacement cardinal. Le chargement refuse un catalogue modifié, une version inconnue,
un fichier tronqué, les données supplémentaires et les états incohérents.
L’écriture passe par un fichier temporaire synchronisé puis un renommage atomique.
Les index de filiation, de sites alimentaires et d’identités d’habitats sont dérivés : ils
sont reconstruits à la restauration. Toute opération de chargement, de redémarrage ou de
relecture prépare et valide la nouvelle expérience (et sa scène) avant de remplacer l’ancienne ;
une erreur laisse l’expérience courante intacte et l’écriture de `previous.bin` est atomique.
Le catalogue n’est pas recopié dans chaque checkpoint. L’empreinte détecte les
changements ; elle n’est pas une signature cryptographique.

Les responsabilités se répartissent entre `biology`, `habitat`, `ecosystem`,
`definitions`, `persistence`, `world_scene`, `interface` et `application`.
Les `mod.rs` ne contiennent que déclarations et exports. Les tests unitaires sont
dans les sous-modules `tests` des domaines ; les tests d’intégration sont dans
`tests/` et utilisent uniquement l’interface publique du programme : la CLI et les fichiers BIN.
Le projet reste un exécutable, sans `lib.rs`. Les types et opérations partagés sont
limités à `pub(crate)` ; tous les champs restent privés et les mutations passent
par les méthodes des propriétaires de l’état.

## Interface et rendu

- **Mise en page** : barre d’outils et pastilles de statistiques à retour à la ligne, colonne latérale
  proportionnelle (300 à 480 px), échelle de l’interface selon la densité de pixels de l’écran. La zone
  3D, le picking et la cible de rendu utilisent la zone réellement calculée par l’interface ; la cible
  n’est recréée qu’une fois le redimensionnement stabilisé (120 ms).
- **Courbes** : effectifs, naissances cumulées, décès cumulés et énergie moyenne sur toute la durée de
  l’expérience, sauvegardés avec elle. L’historique se condense (un point sur deux) au lieu de s’interrompre.
- **Sol** : un unique plan texturé, cuit une fois à partir du monde (substrat sable/terre/eau/roche aux
  frontières gauchies, relief par carte de normales, halo des sources de chaleur, vue
  thermique interpolée). Il remplace des milliers de dalles ; les substrats se fondent sans bordure visible et chaque site reste sélectionnable. L'eau,
  l'humidité et le relief qui évolue sont peints par-dessus (voir *Eau* ci-dessous).
- **Eau** : cycle complet (évaporation, vent, nuages, pluie locale, infiltration, écoulement, érosion
  et dépôt) décrit dans [docs/water-cycle.md](docs/water-cycle.md). Surcouche translucide interpolée
  (nappes, humidité, relief ombré), voile nuageux, gouttes là où il pleut et traînées le long des
  courants. Les organismes subissent les courants selon leur adaptation à l'eau et leur métabolisme.
- **Objets** : roches en amas déformés par du bruit, baies lumineuses qui grossissent avec la réserve
  du site, sources thermiques qui pulsent, bulles d'oxygène, cristaux de minéraux, flaques. Les matériaux
  émissifs brillent réellement (bloom) ; les formes et matériaux sont partagés par toute la scène.
- **Organismes** : membranes légèrement déformées, noyau et vésicules en surface, couleur et lueur selon
  l'énergie, la température ou l'adaptation. Ils respirent, s'orientent et s'étirent en se déplaçant,
  naissent du parent en grandissant et rétrécissent à leur mort. Chaque organisme est dessiné à sa position
  réelle ; sa taille diminue doucement dans un site très peuplé, sans jamais le masquer. Au-delà de 2 500 individus les organelles ne sont plus dessinées, au-delà de 2 000 ils ne
  projettent plus d'ombre, et au-delà de 1 200 habitats garnis chaque ressource se réduit à un seul
  élément : ces seuils ne changent que le dessin, jamais la simulation.
- **Éclairage** : soleil chaud avec ombres en cascade, contre-jour froid proportionnel au monde,
  12 lumières thermiques au plus (les autres sources restent émissives), HDR et bloom.
- **Sélection** : lancer de rayon analytique (plan du biotope et sphères des organismes), anneau lumineux
  autour de l'organisme choisi, anneau posé sur le site choisi.
- **Mouvement** : lissage en temps réel, indépendant de la cadence d'affichage, de la vitesse de
  simulation et du pas biologique. La scène n'est resynchronisée que lorsque la simulation a avancé ou
  que la vue change.

## Vérifier

```sh
cargo check --locked --offline --all-targets
cargo test --locked --offline
```

Le scénario prolongé vérifie 16 graines pendant deux heures simulées chacune,
sans extinction ni dépassement de l’enveloppe de régression, ainsi que la poursuite
des naissances et des mutations à l’échelle de l’ensemble. La survie universelle
n’est pas garantie. Voir [le cycle de vie](docs/life-cycle.md), [le cycle de l'eau](docs/water-cycle.md) et
[le réseau neuronal et l’environnement](docs/neural-environment.md).

Le contrôle graphique s’effectue depuis l’extérieur de l’application : lancer le
binaire, manipuler les commandes natives, inspecter le rendu, puis vérifier la
reprise du fichier sauvegardé. Aucun scénario de test, injection de messages UI
ou rapport de test n’est embarqué dans le code de production.
