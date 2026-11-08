# Laboratoire : la fenêtre d'administration

La fenêtre 3D reste celle que l'on montre en direct. Une **seconde fenêtre**, séparée, pilote
l'expérience : on la place sur un autre écran, et elle n'apparaît pas dans la capture de la fenêtre 3D.

```
cell_simulation --with-panel        # simulation 3D + laboratoire
cell_simulation                     # simulation seule (le lien local reste ouvert)
cell_simulation --panel             # laboratoire seul, pour une simulation déjà lancée
--admin-port PORT                   # port du lien (47821 par défaut, 127.0.0.1 uniquement)
```

Le laboratoire est un second processus (egui) relié à la simulation par un lien TCP local : messages
RON d'une ligne (`src/admin/`). Il se reconnecte seul si la simulation redémarre. Une fenêtre à la fois.

## Brouillon et application

Le laboratoire travaille sur un **brouillon** des définitions (le contenu de `definitions.ron`). La
barre du haut dit s'il diffère de ce que la simulation utilise :

- **Appliquer en direct** : la simulation adopte le brouillon sans perdre ses cellules. Elles gardent
  leur génome et se réexpriment avec les nouvelles règles (capacités, composants 3D, matériaux,
  coûts, paramètres du monde). La scène 3D est reconstruite. Refus, sans rien modifier, si le brouillon
  est invalide (motif inconnu, composant incohérent…) ou si la taille du monde change.
- **Redémarrer avec le brouillon** : nouvelle expérience (graine facultative), cellules initiales du
  brouillon, monde régénéré. L'expérience précédente est conservée dans `previous.bin`.
- **Annuler** : revient aux définitions de la simulation.

## Les onglets

- **Pilotage** : pause, un pas, vitesse, sauvegarde ; créer des cellules (nombre, énergie, motifs
  ajoutés au génome, réparties ou non) ; en tuer (part, nombre, les plus faibles, toutes) ; déclencher
  un orage, former des nuages ; ajouter nourriture, sources de chaleur, oxygène, flaques, minéraux ;
  remplir la nourriture. Le recensement des produits exprimés s'affiche en direct.
- **Capacités** : les motifs (bits, orientation) et les capacités. Une capacité est un mécanisme du
  moteur (liste ci-dessous, avec sa description) et des contributions : un montant, des conditions
  (nombre d'occurrences, même séquence, ordre, séquences identiques, position, signal du milieu), une
  dose par copie. Créer, dupliquer, modifier, supprimer. Plus : mutations, coûts, réseau neuronal.
- **Rendus 3D** : les composants visibles. Chacun dit ce qui le développe (un produit, les unités
  nerveuses, ou toujours), à partir de quel niveau il apparaît, les conditions sur le milieu
  d'acclimatation (eau, lumière…), combien d'exemplaires, où ils se placent, leur forme, leur matériau,
  leurs dimensions, et des **variations** selon le développement (autre matériau, échelle, modèle GLB).
  Les matériaux (couleur, rugosité, métal, émission) se règlent aussi. Les modèles GLB se placent dans
  `data/assets/` ; sans modèle chargé, la forme procédurale s'affiche.
- **Départ** : nom de l'espèce, groupes de cellules initiales (nombre, énergie, motifs ajoutés ou
  génome complet), génome de l'ancêtre, règles d'énergie et de cadence, paramètres du monde.
- **Configurations** : enregistre le brouillon sous un nom dans `data/configs/<nom>.ron`, le retrouve,
  le supprime. Une configuration chargée devient le brouillon ; on l'applique ensuite en direct ou par
  redémarrage (une configuration d'une autre espèce demande un redémarrage).

## Mécanismes du moteur disponibles pour les capacités

Les quinze produits de `Product` (`src/genetics/product.rs`) : métabolisme, croissance, fécondité,
réparation, hydratation, cuticule, tissu nerveux, digestion, pigments, thermosynthèse, prédation,
appareil moteur, adhésion, conjugaison, recombinaison. L'éditeur les liste depuis `Product::ALL` avec
`description()` : un nouveau mécanisme y apparaît sans toucher au laboratoire.

## Inventer un mécanisme entièrement nouveau (en Rust)

1. `src/genetics/product.rs` : ajouter la variante, l'insérer dans `ALL`, incrémenter `COUNT`, écrire
   `label()` et `description()`. Le compilateur signale ensuite chaque tableau à compléter
   (les tableaux de `Physiology` sont de taille `Product::COUNT`).
2. `src/biology/physiology.rs` : donner au produit son effet (une loi physique) et, si besoin, son
   entretien par rapport à l'ancêtre.
3. Ailleurs si le mécanisme agit sur le monde (`src/ecosystem/`), comme la prédation ou la conjugaison.
4. Facultatif : un composant visible dans l'onglet Rendus 3D (aucun code), ou une nouvelle forme dans
   `PartShape`.

Le laboratoire propose alors le nouveau produit aux capacités, aux composants et au recensement, et les
motifs qui le fabriquent se définissent dans l'éditeur.

## Interventions

`Intervention` (`src/ecosystem/intervention.rs`) est l'unique entrée des actions de l'expérimentateur :
`Simulation::intervene` les applique entre deux pas en gardant filiation, liaisons et sauvegardes
cohérentes (cellules créées sans parent, mortes enterrées immédiatement). Ajouter une action, c'est
ajouter une variante et son traitement dans `src/ecosystem/simulation/interventions.rs`, puis un bouton
dans `src/panel/live_tab.rs`.
