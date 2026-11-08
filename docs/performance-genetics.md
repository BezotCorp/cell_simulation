# Mesures de performance de la génétique

Mesures (CPU, profil release) : `cargo test --release measure -- --ignored --nocapture`,
population de 20 000 cellules obtenue par divisions successives avec mutations.

| Mesure                                       | Résultat                      |
| -------------------------------------------- | ----------------------------- |
| génomes distincts / cellules                 | 2 809 / 20 000                |
| séquences si chaque cellule avait son génome | 481 463 octets                |
| fragments réellement stockés                 | 25 367 octets (19 fois moins) |
| registre de fragments                        | 2 464 retrouvés, 2 445 créés  |
| interprétation d'un génome à froid           | 2,18 µs (15 produits compris) |
| interprétation à chaud (cache)               | 0,023 µs                      |
| `interpret_all`, 1 / 2 / 4 / 8 fils          | 44,9 / 22,2 / 12,1 / 6,7 ms   |

## Répartition CPU / GPU

L'interprétation coûte quelques microsecondes par génome distinct et rien pour les génomes déjà
vus ; la simulation entière à 20 000 cellules reste dominée par le déplacement et l'hydrologie,
pas par la génétique. Le travail est donc réparti ainsi :

- interprétation par génome (table d'octets, cache dans les fragments et dans le génome) : CPU, mono-fil
  en régime courant, car 99 % des appels sont des succès de cache ;
- échauffement des génomes distincts au chargement : CPU parallèle (au-delà de 256 génomes),
  mise à l'échelle quasi linéaire mesurée jusqu'à 8 fils ;
- GPU : non utilisé. Fyrox 1.0 n'expose pas de calcul général (compute) ; le GPU sert au rendu
  (eau, pluie, nuages, foudre). Le coût mesuré ne justifierait de toute façon pas un transfert
  CPU↔GPU. Aucune mesure GPU n'a été faite.
