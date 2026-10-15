# ADR 0005 — Assemblage CSR et élimination des prescriptions

Date : 15 octobre 2026. Statut : implémenté, vérifications numériques locales ; revue scientifique indépendante ouverte.

## Décision

S04 utilise les opérateurs TET4 S03 pour assembler une matrice CSR via `nalgebra-sparse` déjà verrouillé au workspace. La numérotation suit les IDs de nœuds triés et les composantes xyz. Chaque élément reçoit exactement une loi. Le modèle et les liaisons sont vérifiés avant assemblage.

Les déplacements imposés sont éliminés avec `Ff − Kfc uc`, en conservant K et F originaux pour les réactions. Aucun remplacement artificiel des diagonales ni pénalisation. Les contraintes identiques sont fusionnées ; les contradictoires sont rejetées. Le système totalement prescrit accepte zéro inconnue libre.

Le stockage et le produit CSR appartiennent à `ferrasim-solvers` ; la numérotation, l’assemblage et la cinématique à `ferrasim-fem`. Le solveur dense reste limité aux petits tests ; CG/Jacobi est réservé à S05. La construction des forces à partir des charges JSON reste à S06.

## Conséquences

La conservation d’une copie CSR du système original et l’accumulation séquentielle par triplets consomment de la mémoire supplémentaire, sans allouer de matrice dense globale. La référence rationnelle indépendante du chemin Rust et le patch affine vérifient le contrat avant l’introduction du solveur itératif. Les problèmes sous-contraints ne sont pas encore diagnostiqués par un solveur.

Voir le [contrat et le protocole](../ASSEMBLY.md) et le [rapport S04](../sprints/S04.md). La réussite numérique ne constitue pas une qualification scientifique indépendante.
