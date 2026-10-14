# Conventions numériques du MVP

Statut : contrat de travail S00, à relire indépendamment avant qualification S03.

## Unités et repères

SI pour les exemples : m, N, Pa, déplacements en m ; déformations sans dimension. Les modèles utilisent des unités cohérentes déclarées, sans conversion en 0.1. Repère cartésien global direct `(x,y,z)` ; aucune orientation matériau anisotrope dans le MVP. Traction positive en tension ; normale extérieure pour les frontières.

Tous les scalaires de calcul sont `f64`. Les coordonnées, matériaux et charges doivent être finis. Les validations documentent échelle et conditionnement ; la reproductibilité est dans les tolérances, sans promesse bit à bit entre plateformes.

## Tenseurs et cisaillement

Ordre de Voigt commun : `xx, yy, zz, xy, yz, xz`.

- Contrainte : `[σxx, σyy, σzz, τxy, τyz, τxz]`.
- Déformation d’ingénieur : `[εxx, εyy, εzz, γxy, γyz, γxz]`, avec `γij = 2 εij`.
- Énergie volumique : `0.5 * stress_voigt.dot(strain_engineering)`.
- Matrice isotrope : bloc normal avec `λ + 2μ` sur diagonale et `λ` hors diagonale ; bloc cisaillement diagonal `μ`.
- `μ = E / (2(1+ν))`, `λ = Eν / ((1+ν)(1−2ν))`.

Valider `E > 0` et `−1 < ν < 0.5`. Le matériau presque incompressible n’est pas garanti avec TET4 ; un avertissement sera spécifié en S03.

Von Mises : racine de `0.5*((σxx−σyy)^2 + (σyy−σzz)^2 + (σzz−σxx)^2) + 3*(τxy^2+τyz^2+τxz^2)`. La formule s’applique à la contrainte par élément non lissée.

## Élément et degrés de liberté

Tétraèdre de référence : `(0,0,0), (1,0,0), (0,1,0), (0,0,1)` ; fonctions `[1−r−s−t, r, s, t]`. Jacobien à colonnes `x₂−x₁, x₃−x₁, x₄−x₁`. Volume signé `det(J)/6`.

En 0.1, un élément d’orientation négative est rejeté avec son identifiant ; aucune permutation silencieuse. S02 fixe le critère initial `|det(J)| / Lmax³ ≤ 10⁻¹²`, où `Lmax` est la plus grande arête. Les tests vérifient ce critère à plusieurs échelles ; sa revue indépendante reste requise avant qualification S03. Il ne faut pas utiliser une tolérance de volume absolue identique pour toutes les unités et échelles. Voir les [règles de maillage et orientations de frontière](MESH-FORMAT.md).

Ordre local : `[u1x,u1y,u1z,u2x,u2y,u2z,u3x,u3y,u3z,u4x,u4y,u4z]`. Les identifiants de fichiers ne sont pas des indices de tableau ; la correspondance ID/index est explicite. Les contraintes et déformations sont constantes par élément.

## Charges, prescriptions et réactions

Force nodale : N par nœud explicitement identifié. Traction : N/m² ; pour un triangle linéaire de surface et une traction constante, force consistante `A*t/3` à chacun de ses trois nœuds. Aucun partage automatique d’une force totale n’est implicite.

Pour déplacements prescrits `u_c`, résoudre `K_ff u_f = F_f − K_fc u_c`. Réactions sur composantes contraintes : `R_c = (K_original*u − F_original)_c`. Conserver le système ou les opérations permettant ce calcul avant élimination. Équilibre : somme charges et réactions, et moments autour d’une origine déclarée.

Énergie élastique : `0.5*uᵀ*K_original*u`. La comparaison à `0.5*uᵀ*F` ne doit pas ignorer le travail des prescriptions non nulles.

## Résidus et métriques

Pour le système réduit, critère proposé : `||b−Au||₂ ≤ atol + rtol*||b||₂`. Le cas `b=0` ne doit jamais produire un ratio `0/0`. Recalculer le résidu final ; le statut dépend du critère et du nombre maximal d’itérations.

Erreurs de référence : `||q−q_ref|| / max(||q_ref||, q_scale)` avec échelle positive publiée pour les valeurs nulles. Rigidité : norme de Frobenius ; champs : norme L2 ou maximum explicitement nommé. Modes rigides : mesurer `||K*r|| / (||K||*||r||)` pour chaque vecteur non nul.

Bilans : normaliser les forces par une échelle de charges/réactions positive et les moments par cette échelle multipliée par une longueur de référence. Une simple somme vectorielle des charges, qui peut être nulle, n’est pas une échelle admissible.

## Références

Les conventions de Voigt et la politique d’orientation sont des choix FerraSim. La formulation élastostatique peut être vérifiée à partir du [chapitre 8.1 de A. F. Bower](https://solidmechanics.org/text/Chapter8_1/Chapter8_1.htm). Le format d’entrée maillage est défini par le [manuel Gmsh, MSH ASCII 2](https://gmsh.info/doc/texinfo/gmsh.html#MSH-ASCII-file-format).
