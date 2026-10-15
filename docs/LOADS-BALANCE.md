# Charges et bilans mécaniques — S06

`AssembledSystem::assemble(mesh, model)` intègre désormais les charges du modèle en plus de K et des prescriptions. `forces()` expose le vecteur complet dans l’ordre interne xyz. `reduce_model_loads()` l’utilise pour l’élimination ; `solve_model_loads(settings, origin, scales)` résout par CG/Jacobi puis reconstruit déplacements, réactions et bilans.

L’API `reduce(explicit_forces)` reste disponible pour les références ou scénarios alternatifs. `balance(full_displacements, origin, scales)` évalue toujours **les charges du modèle stockées à l’assemblage** ; elle ne doit pas être utilisée pour diagnostiquer un scénario de forces alternatif. Ce dernier conserve son propre résidu/réactions via `ReducedSystem`.

## Forces nodales et tractions

Un enregistrement `nodal_force` ajoute ses trois composantes exactement une fois au nœud identifié, indépendamment du nombre de tétraèdres adjacents. Plusieurs enregistrements sont additifs, y compris ceux visant le même nœud ; aucune déduplication implicite d’un chargement explicitement répété. Les forces sont des forces par nœud, pas des résultantes à répartir sur un groupe.

Un enregistrement `surface_traction` ajoute pour chaque triangle sélectionné de surface A :

```text
F_a = integral(N_a * t dA) = (A/3) * t
```

Chacun des trois sommets reçoit cette contribution. Les nœuds partagés additionnent les contributions de leurs triangles incidents ; chaque triangle du groupe est parcouru une fois par enregistrement de charge. La traction est un **vecteur cartésien global**, en force/longueur² ; elle n’est pas une pression scalaire à multiplier par une normale. L’aire physique vient de la géométrie validée S02. Le résultat ne dépend pas du sens de numérotation du triangle source.

Les références de nœuds/groupes, sélections vides utilisées et surfaces internes sont rejetées par la liaison modèle/maillage avant assemblage. Seules les faces extérieures explicitement présentes dans le groupe peuvent recevoir une traction. Valeurs non finies, débordement d’accumulation, contribution de traction non nulle totalement sous-fluée ou poids A/3 non représentable provoquent une erreur ; aucun vecteur partiel n’est retourné. L’arithmétique intermédiaire reste conservatrice : pas de mise à l’échelle automatique pour récupérer tous les cas extrêmes.

Cette intégration suit la forme faible élastostatique décrite par [A. F. Bower, chapitre 8.1](https://solidmechanics.org/text/Chapter8_1/Chapter8_1.htm). Les conventions d’API, de sommation, de diagnostic et de normalisation sont celles de FerraSim ; aucun code de cette source n’est copié.

## Réactions, forces et moments

Le système original est conservé avant élimination :

```text
r = K_original * u - F_original
R_c = r_c sur les composantes prescrites ; R_f = 0
force_imbalance = somme(F + R)
moment_imbalance = somme((x - origin) croix (F + R))
```

Les forces appliquées aux composantes prescrites sont bien soustraites pour obtenir les réactions. Les résidus libres ne sont pas réinterprétés comme des réactions : leur norme est publiée séparément.

`BalanceScales::new(force_scale, length_scale)` impose deux échelles finies strictement positives et un produit représentable. Les unités sont celles du modèle ; aucun changement d’unités implicite. Choisir des valeurs représentatives du chargement/réactions et de la géométrie, et les publier avec le résultat. La norme d’une résultante totale potentiellement nulle n’est pas un choix automatique admissible.

`MechanicalBalance` conserve l’origine et ces échelles, les résultantes appliquées et de réaction, leurs moments, les déséquilibres et :

- `relative_force_error = norm(force_imbalance)/force_scale`.
- `relative_moment_error = norm(moment_imbalance)/(force_scale*length_scale)`.
- `relative_free_residual = norm(r_f)/force_scale`.
- `relative_prescription_error = norm(u_c-u_c_imposé)/length_scale`.

L’origine doit être finie. Un changement d’origine modifie les moments de chaque résultante selon `M(O+d)=M(O)-d croix F`, sans modifier les forces ; le déséquilibre transformé reste inspectable. Le diagnostic des prescriptions évite qu’un champ arbitraire équilibré soit pris pour un champ respectant les appuis.

## Énergie et travail des prescriptions

```text
U = 0.5 * u^T K_original u
W_load = u^T F_original
W_prescribed = u_c^T R_c
energy_gap = abs(2*U - W_load - W_prescribed)
energy_scale = max(force_scale*length_scale, abs(2*U), abs(W_load)+abs(W_prescribed))
relative_energy_error = energy_gap / energy_scale
```

Les deux travaux sont des produits évalués à l’état final ; un chargement linéairement rampé donne la moitié de leur somme. Omettre le travail des réactions est incorrect lorsque les prescriptions sont non nulles. L’énergie globale est comparée à la somme des énergies élémentaires sur le cas analytique.

Ces scalaires sont calculés en f64 sans correction artificielle ; un mouvement rigide peut donner une très petite énergie signée due aux arrondis. Les valeurs brutes restent visibles. Les débordements des produits, sommes ou normalisations sont rejetés. Le diagnostic ne remplace ni une analyse de conditionnement ni une étude de convergence.

## Référence analytique et seuils

Le [générateur](../validation/generate_load_reference.py) construit un bloc SI de dimensions 2×1×1, partagé en six TET4, avec IDs non consécutifs. Les [entrées exécutables](../examples/uniaxial/model.json) et la [référence analytique](../validation/fixtures/uniaxial-reference.json) sont régénérables sans appeler Rust, B, K ou un solveur. Le calcul des forces et du champ analytique utilise des fractions exactes ; les aires des triangles chargés valent 1/2.

`E=1000 Pa`, `ν=0.25`, traction `(10,0,0) Pa` sur x=2. Les appuis imposent seulement ux=0 sur x=0, uy=0 sur y=0 et uz=0 sur z=0 ; ils permettent la contraction de Poisson. Ce n’est pas un encastrement complet de x=0.

```text
u = (0.01*x, -0.0025*y, -0.0025*z)
sigma = (10,0,0,0,0,0) Pa
U = 0.1 J
F = (10,0,0) N ; M à l’origine = (0,5,-5) N.m
```

Le [corpus de tests](../crates/fem/tests/loads.rs) fixe les critères suivants :

| Cas/quantité | Critère |
| --- | --- |
| Forces nodales de traction et résultantes | Erreur normalisée ≤ 10⁻¹⁰, échelle minimale 1 N ou 1 N.m |
| Déplacements et contraintes uniaxiaux | Erreur normalisée ≤ 10⁻⁶ ; échelles minimales 10⁻³ m et 1 Pa |
| Déplacement axial moyen sur x=2 | Erreur relative à 0.02 m ≤ 10⁻⁶ |
| Bilans forces, moments et résidu libre | Erreurs normalisées ≤ 10⁻⁶ |
| Énergie/travaux et prescriptions | Erreurs normalisées ≤ 10⁻¹⁰ |
| Énergie analytique et somme élémentaire | Écart absolu ≤ 10⁻¹⁰ J pour cette fixture |

Les bilans des cas de bloc utilisent une force de référence 10 N et une longueur de référence 2 m, même pour la résultante nulle. Le triangle oblique utilise 10 N et 1 m. Les comparaisons de champs utilisent `norm(q-qref)/max(norm(qref),q_scale)` ; les échelles minimales ci-dessus s’appliquent seulement à ces références.

Le corpus couvre aussi un triangle oblique à traction vectorielle, le retournement de triangles, les nœuds partagés, la superposition explicite, un mélange traction/force nodale sur un appui, les prescriptions non nulles, un couple de forces de résultante nulle, plusieurs origines, un modèle sans charge, le cas entièrement prescrit, un champ non résolu et les entrées/plages numériques invalides.

## Succès numérique et limites

`StaticSolution` est retourné seulement après succès de CG et calcul de tous les diagnostics. Un échec CG est propagé en `FemError::Solver` ; aucune solution statique réussie n’est retournée. Les seuils mécaniques ne sont pas cachés dans l’API : le statut CG concerne son résidu, les valeurs de bilan doivent être évaluées avec les critères déclarés du cas. `balance` accepte également des champs arbitraires pour les diagnostiquer, sans affirmer qu’ils convergent.

La couverture est limitée à la statique linéaire TET4 avec charges mortes nodales et tractions uniformes par groupe. Pas de pression suiveuse, charge volumique, inertie, contact ou interpolation variable de traction. L’exemple résout les charges réelles du JSON mais n’exporte pas encore de résultats ; S07 prépare les champs/exports et S08 la commande CLI complète.

```sh
python3 validation/generate_load_reference.py --check
cargo test -p ferrasim-fem --test loads --locked -- --nocapture
cargo run -p ferrasim-cli --example solve_model --locked -- examples/uniaxial/model.json 10 2
cargo run -p ferrasim-cli --example solve_model --locked -- examples/minimal/model.json 1 1
```

L’exemple prend les échelles force et longueur en arguments et utilise explicitement l’origine `(0,0,0)`. Il n’écrit aucun fichier et n’affiche les résultats qu’après tous les calculs réussis. Les fixtures sont créées pour FerraSim sous sa licence MIT. La référence analytique séparée et les tests ne remplacent pas une revue humaine indépendante ; cette revue reste ouverte avant qualification scientifique.
