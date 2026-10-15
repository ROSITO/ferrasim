# Assemblage creux et prescriptions — S04

L’API Rust assemble la rigidité élastique TET4, élimine les déplacements imposés et reconstruit déplacements, résidu et réactions après fourniture d’une solution réduite. Depuis S05, `ReducedSystem::solve` utilise le [solveur CG/Jacobi](SOLVER-CG.md). Le solveur dense utilisé pour les vérifications existe uniquement dans l’exécutable de tests d’intégration, avec une limite de 96 degrés de liberté.

## Numérotation et affectations

`AssembledSystem::assemble(&ValidatedMesh, &Simulation)` vérifie le modèle et sa liaison au maillage avant tout assemblage. Les identifiants externes de nœuds sont triés ; chaque nœud possède trois degrés de liberté successifs x, y, z. `DofMap::index` vérifie identifiant et composante. Les identifiants ne servent jamais directement d’indices.

Chaque tétraèdre importé doit appartenir à exactement une affectation matériau. Les régions absentes ou vides utilisées, références invalides et affectations répétées sont rejetées. Le format MSH 2.2 retenu associe un seul groupe physique volumique à chaque tétraèdre ; la validation du modèle refuse plusieurs affectations d’une même région. L’assemblage contrôle aussi l’unicité au niveau élément pour éviter toute sélection implicite. Les nœuds sans tétraèdre sont déjà rejetés par la validation du maillage S02.

Chaque loi est construite avec les vérifications S03. Les avertissements matériau, dont la quasi-incompressibilité, restent accessibles via `material_warnings()`.

## Stockage et assemblage

Pour les indices globaux `I_e` de l’élément, `K[I_e[i], I_e[j]] += K_e[i,j]`. Tous les 144 coefficients locaux sont pris en compte, sans symétrisation artificielle. Les contributions sont accumulées dans un ordre déterministe, puis stockées en CSR via `nalgebra-sparse` : colonnes triées, doublons sommés et zéros exacts omis. Le système global conserve sa matrice originale immuable.

`ferrasim-solvers::SparseMatrix` fournit un constructeur par triplets, l’accès CSR en lecture et le produit matrice-vecteur. Dimensions, indices, valeurs finies et débordements lors des accumulations sont contrôlés. Une arithmétique intermédiaire non représentable est rejetée même si une autre réorganisation pourrait éviter le débordement. Aucun redimensionnement ou conditionnement automatique n’est appliqué.

Cette première implémentation garde les triplets avant conversion et utilise un `BTreeMap` pour leur agrégation. Elle n’alloue pas de matrice globale dense ; mémoire proportionnelle aux contributions, entrées non nulles et degrés de liberté. La réduction conserve une copie CSR du système original. Ni parallélisme ni objectif de performance industrielle ne sont qualifiés ici.

## Élimination et reconstruction

`Some(0)` est un déplacement imposé nul ; `None` est une composante libre. Les groupes sont développés vers les nœuds avant élimination. Les prescriptions identiques qui se recouvrent sont fusionnées ; des valeurs différentes sur le même degré de liberté sont rejetées par comparaison exacte, sans tolérance implicite.

Le vecteur de forces `F` est fourni explicitement dans l’ordre global. **Les charges JSON sont validées mais ne sont pas intégrées par S04** ; leur construction relève de S06. Passer un vecteur nul définit un scénario sans forces externes, pas une interprétation du chargement JSON.

Pour les indices libres `f` et prescrits `c` :

```text
Kff uf = Ff - Kfc uc
u[f] = uf ; u[c] = uc
r = K_original u - F_original
R[c] = r[c]
```

`AssembledSystem::reduce(F)` utilise les prescriptions du modèle. `ReducedSystem::new(K, F, prescriptions)` permet une réduction explicite, notamment pour les références de test. Le résultat expose la rigidité réduite, le second membre et la correspondance des indices libres. Les cas sans prescriptions et entièrement prescrits sont définis ; ce dernier retourne un système 0×0 et accepte une solution réduite vide.

`reconstruct` vérifie la dimension et la finitude de la solution fournie. `residual` conserve toutes les composantes : les composantes libres servent à vérifier l’équilibre et les composantes contraintes sont les réactions. `reactions` retourne uniquement ces dernières, avec leurs indices globaux. Ces fonctions ne déclarent pas la convergence : une solution quelconque produit un résidu quelconque. Les diagnostics et le critère de convergence du solveur sont définis en [S05](SOLVER-CG.md), sans preuve exhaustive de singularité.

## Références et seuils

La référence [Python rationnelle](../validation/generate_patch_reference.py) utilise le calcul tensoriel exact S03, assemble dans des listes denses et réduit par élimination rationnelle, sans appeler Rust ou sa matrice B. Le [fichier régénérable](../validation/fixtures/patch-reference.json) contient K, Kff, le second membre réduit, les déplacements et le résidu complet pour deux cas :

- Patch homogène de quatre tétraèdres subdivisant le tétraèdre `(0,0,0), (4,0,0), (0,4,0), (0,0,4)` autour du nœud intérieur `(1,1,1)`. Les quatre sommets extérieurs ont un déplacement affine général non nul ; les trois composantes intérieures sont résolues. `E=150`, `ν=0.25`, unités SI cohérentes.
- Même maillage avec quatre modules `150, 300, 75, 225` et un vecteur de forces explicite non nul, y compris aux composantes prescrites. Ce cas vérifie le choix du matériau et la soustraction des forces originales dans les réactions ; il n’a pas de contrainte uniforme attendue.

Les IDs de nœuds sont volontairement non consécutifs et non ordonnés dans la référence. Les tests convertissent les références vers l’ordre interne.

| Vérification | Critère fixé |
| --- | --- |
| K, Kff et second membre | Erreur L2/Frobenius normalisée ≤ 10⁻¹² |
| Déplacements et contraintes du patch | Erreur normalisée ≤ 10⁻⁸ |
| Résidu complet de référence | Erreur normalisée ≤ 10⁻¹⁰ |
| Symétrie et six modes rigides globaux | Erreur relative ≤ 10⁻¹⁰ |
| Énergie, forces et moments du patch sans charges | Écart absolu ≤ 10⁻¹⁰ dans les unités SI de cette fixture |

Les comparaisons normalisées utilisent `norm(q-qref)/max(norm(qref),1)` : l’échelle 1 est exprimée dans l’unité cohérente de la quantité testée. Les derniers contrôles absolus concernent seulement cette fixture ; ils ne constituent pas le protocole de bilans généraux sous chargement prévu en S06. Le contrôle d’énergie compare la somme élémentaire à `0.5*uᵀ*K*u`, en tenant compte des prescriptions non nulles.

Les autres tests couvrent composantes libres, prescriptions nulles et non nulles, recouvrements cohérents ou contradictoires, forces contraintes, systèmes entièrement prescrits, entrées non finies et débordements. Un test CSR diagonal de 100 000 degrés de liberté vérifie le stockage creux sans constituer un benchmark FEM.

## Reproduction et limites

```sh
python3 validation/generate_patch_reference.py --check
cargo test -p ferrasim-fem --test assembly --locked -- --nocapture
cargo test -p ferrasim-solvers --test sparse --locked
cargo run -p ferrasim-cli --example inspect_assembly --locked -- examples/minimal/model.json
```

L’exemple affiche les dimensions et les nombres d’entrées stockées, avec un vecteur de forces explicitement nul, sans résoudre ni intégrer les charges JSON.

La formulation globale suit le principe des travaux virtuels et l’assemblage décrits par [A. F. Bower, chapitre 8.1](https://solidmechanics.org/text/Chapter8_1/Chapter8_1.htm). Les conventions de numérotation, seuils et choix CSR sont ceux de FerraSim. Aucun code tiers n’a été copié ; les fixtures sont créées pour le projet sous sa licence MIT.

La référence séparée a le même auteur : elle ne remplace pas la revue scientifique par une personne compétente distincte. Cette revue reste ouverte avant qualification. Les vérifications portent sur de petits systèmes linéaires TET4 ; la convergence CG est vérifiée sur le [corpus S05](SOLVER-CG.md), sans qualification indépendante ; pas de qualification de traction surfacique, poutre, performance industrielle ou comparaison à un solveur externe.
