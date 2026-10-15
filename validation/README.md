# Protocole de validation scientifique

Le [registre](cases.toml) fixe les cas, métriques et tolérances avant calcul. **S03 vérifie les opérateurs TET4 ; S04 vérifie un patch multiélément et une réduction avec prescriptions non nulles par références denses. S05 vérifie CG/Jacobi sur un corpus creux et les références FEM ; S06 vérifie charges, traction uniaxiale et bilans mécaniques.** Le champ affine est vérifié localement, sans remplacer le patch test multiélément de S04. Les entrées invalides et échecs sont partiellement couverts par S01–S06. La revue scientifique indépendante est ouverte ; le contrôle Python ne valide que les métadonnées, pas un solveur.

Chaque cas devra conserver entrées, référence indépendante, environnement, observable, échelles de normalisation, résultats et état passé/échoué. Les références sont choisies avant examiner les résultats. Les valeurs proches de zéro utilisent une échelle absolue ; les singularités ne sont pas des valeurs de comparaison ponctuelles valides.

## Protocole par famille

| Cas | Données et référence à préparer | Preuve attendue |
| --- | --- | --- |
| Symétrie et modes rigides | TET4 bien conditionné ; six champs rigides ; expression BᵀCB | Erreur normalisée ≤ 10⁻¹⁰ |
| Patch affine | Maillage multiélément et déplacement affine sur frontière | Déplacements et contraintes ≤ 10⁻⁸ |
| Traction uniaxiale | Barre, traction uniforme, contraintes supprimant seulement les modes rigides et permettant contraction latérale | Erreurs ≤ 10⁻⁶ |
| Prescriptions non nulles | Petit système et résolution dense indépendante | Erreur ≤ 10⁻⁸ |
| Traction surfacique | Triangle simple, aire et vecteur de traction connus | Résultante ≤ 10⁻¹⁰ |
| Équilibre | Charges, réactions et origine des moments | Bilan normalisé ≤ 10⁻⁶ |
| Poutre | Protocole ci-dessous et trois raffinements minimum | Convergence et erreur de déplacement ≤ 5 % selon PRD |
| Comparaison externe | Même maillage, matériau, forces et prescriptions ; solveur/version archivés | Déplacements/contraintes élémentaires ≤ 10⁻⁵ |
| Entrées invalides | Références absentes, groupes vides utilisés, dégénérescence, prescriptions contradictoires et singularité de référence | Rejet ou échec explicite, sans succès de calcul |

## Démonstrateur de poutre S09 — v2 adoptée

Poutre prismatique selon x : longueur 1 m, largeur 0,05 m selon y, hauteur 0,05 m selon z ; `E=210e9 Pa`, `ν=0.3`. Face x=0 encastrée ; résultante `Fz=−100 N` distribuée uniformément sur x=L. Aucune force concentrée sur un coin.

Observable : moyenne surfacique du déplacement z de la face chargée. Moment quadratique de référence pour cette flexion : `I = b*h^3/12`. Valeur Euler–Bernoulli : `−F_abs*L^3/(3*E*I) ≈ −0,000304761905 m`.

La formule est une référence de poutre approchée, pas une solution exacte du solide 3D. En S09, préparer au moins trois maillages documentés, publier erreur/déplacement/temps/ressources et distinguer discrétisation de l’écart de modèle. Le seuil de 5 % n’est pas un critère sur une contrainte singulière à l’encastrement. Si les hypothèses rendent cette comparaison inadaptée, utiliser une référence solide 3D indépendante convergée, comme autorisé par le PRD, et publier la justification.

## Sources et droits

- [A. F. Bower — élastostatique FEM](https://solidmechanics.org/text/Chapter8_1/Chapter8_1.htm) : formulation et vérification théorique. Aucun code de cette source n’est copié.
- [Gmsh — manuel](https://gmsh.info/doc/texinfo/gmsh.html) : contrat du format MSH.
- [CalculiX — site du projet](https://www.calculix.de/) : candidat pour la comparaison externe ; installation et version à qualifier en S10.
- [Conventions FerraSim](../docs/NUMERICAL-CONVENTIONS.md) et [PRD MVP](../docs/PRD.md) : choix, tolérances et règles de remplacement de référence.

Les fixtures `tet4-positive.msh`, `two-tets-interface.msh` et le modèle JSON sont créés pour FerraSim et distribués sous la licence du dépôt. Les tests S02 vérifient les volumes, aires et incidences de ces géométries de référence. Les autres données à ajouter doivent porter provenance et droits de redistribution.

## Référence élémentaire S03

Le [générateur rationnel](generate_tet4_reference.py) produit trois [références denses](fixtures/tet4-reference.json) sans appeler le code Rust ni construire sa matrice B. Les tests comparent les gradients, volumes et 144 coefficients de rigidité, puis contrôlent des réponses et invariances analytiques. Voir la [formulation TET4](../docs/FEM-TET4.md) pour critères, limites et distinction entre référence calculée séparément et revue humaine indépendante.

## Références globales S04

Le [générateur global rationnel](generate_patch_reference.py) assemble quatre TET4, puis élimine les prescriptions en arithmétique exacte. Les [deux références](fixtures/patch-reference.json) couvrent le champ affine homogène et les matériaux hétérogènes avec forces explicites. Les tests Rust vérifient opérateurs, déplacements et réactions par un solveur dense limité aux tests. Le [protocole](../docs/ASSEMBLY.md) fixe métriques, échelles et limites ; le [rapport S04](../docs/sprints/S04.md) conserve les résultats. La revue humaine indépendante reste ouverte.

## Résolution CG/Jacobi S05

Le [corpus CG](../crates/solvers/tests/cg.rs) utilise une LU dense réservée aux petits tests et une solution construite de taille 2 000. Les références rationnelles FEM S04 sont également résolues par CG. Le [protocole](../docs/SOLVER-CG.md) définit le résidu recalculé, les tolérances et les scénarios d’échec ; le [rapport S05](../docs/sprints/S05.md) consigne les résultats. Le contrôle [des exécutables d’échec](../scripts/check_solver_failure.py) vérifie sortie non nulle et absence de succès/fichiers. La détection universelle des singularités et la qualification scientifique ne sont pas acquises.

## Charges et bilans S06

Le [générateur analytique](generate_load_reference.py) produit les forces consistantes, déplacements, contraintes et énergie attendus d’un [bloc en traction](../examples/uniaxial/README.md), sans résolution numérique. Les [tests S06](../crates/fem/tests/loads.rs) vérifient aussi les chargements mixtes, prescriptions non nulles, origine des moments et résultante nulle avec échelles positives. Voir le [protocole](../docs/LOADS-BALANCE.md) et le [rapport S06](../docs/sprints/S06.md). La revue humaine indépendante reste ouverte.

## Champs et exports S07

Les [tests FEM](../crates/fem/tests/results.rs) vérifient la récupération et von Mises ; les [tests IO](../crates/io/tests/results_export.rs) vérifient les données et la publication. Les lecteurs [XML/CSV/JSON](../scripts/check_result_exports.py), [analytique](verify_result_reference.py) et [VTK natif](../scripts/check_vtk_results.py) contrôlent les exports. Voir le [bundle observé](results/uniaxial/manifest.json), le [rendu inspecté](results/uniaxial-vtk.png), le [contrat](../docs/RESULTS-FORMAT.md) et le [rapport S07](../docs/sprints/S07.md). Le contrôle VTK natif a été observé sur macOS, distinct de la CI de formats sur trois OS.

## Parcours CLI/API S08

Les [tests CLI natifs](../apps/ferrasim-cli/tests/entrypoint.rs), [tests API](../crates/io/tests/workflow.rs) et la [reproduction en répertoire temporaire](../scripts/check_cli_workflow.py) vérifient les commandes, codes d’échec, chemins et sorties, sans déclarer la solvabilité après la seule validation. Voir le [contrat](../docs/CLI-API.md) et le [rapport S08](../docs/sprints/S08.md).

## Étude de poutre S09

Le [protocole v1](CANTILEVER-PROTOCOL.md), le [pilote reproductible](run_cantilever.py) et le [rapport](../docs/sprints/S09.md) conservent l’échec numérique et les séries diagnostiques. Ces archives décrivent l’état initial partiel ; elles ne remplacent pas l’acceptation ultérieure explicite de la v2. Les entrées sont générées sans dépendance Python externe, avec six TET4 par cellule et chargement réparti.

## Adoption S09

Le [protocole v2](CANTILEVER-PROTOCOL-V2.md) est adopté et actif par défaut. Le cas de convergence passe au statut numériquement vérifié, avec erreur finale 3,0934 %, bilans et sensibilité contrôlés. La [résolution](../docs/sprints/S09-RESOLUTION.md) distingue clôture technique, échecs historiques v1 et revue scientifique indépendante encore ouverte. La comparaison externe reste prévue en S10.
