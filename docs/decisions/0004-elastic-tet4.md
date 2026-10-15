# ADR 0004 — Loi isotrope et opérateurs TET4

Date : 15 octobre 2026. Statut : adopté pour l’implémentation ; qualification scientifique indépendante ouverte.

- Matériau immutable, contrôlé à sa construction, tenseur 6×6 et unités cohérentes conservées.
- Cisaillement d’ingénieur dans l’ordre xx, yy, zz, xy, yz, xz de S00.
- Avertissement de quasi-incompressibilité à partir de ν=0,49, sans modification du matériau ni formulation mixte implicite.
- Réutilisation de la géométrie validée S02 ; aucun second seuil d’orientation/dégénérescence divergent.
- Gradients via Jacobien normalisé et transformation inverse transposée ; opérateurs locaux immutables.
- Rigidité normalisée, sans symétrisation artificielle ; erreurs explicites sur résultats hors plage.
- Référence dense séparée en Python/Fraction, par polynômes affines et contraction tensorielle, sans appeler Rust ni reproduire B.
- Comparaisons analytiques et invariances complémentaires ; aucune assimilation du test affine local au patch test global.

Le champ artificiel de démonstration ne résout pas les prescriptions/charges du JSON. L’assemblage, la réduction du système et les solveurs restent à S04/S05. La revue humaine indépendante n’est pas présumée réalisée par la production d’une seconde implémentation.
