# Résolution CG/Jacobi — S05

`ferrasim-solvers::solve_cg_jacobi(K, b, settings)` résout le système creux réduit. `ReducedSystem::solve(settings)` relie cette API à l’assemblage FEM. Le moteur utilise exclusivement CSR et des vecteurs ; les résolutions denses restent des références de test.

## Domaine et contrôles

Le domaine attendu est une matrice symétrique définie positive, après prescriptions cinématiques suffisantes. CG préconditionné et le préconditionneur diagonal sont décrits dans [Templates for the Solution of Linear Systems, sections 2.3.1 et 3.2](https://www.netlib.org/templates/templates.pdf). FerraSim implémente sa propre version, sans copier de code tiers.

Les contrôles effectués avant les itérations sont :

- Dimension de `b` égale à celle de K, valeurs finies.
- `rtol` et `atol` finis et non négatifs, pas tous deux nuls ; budget strictement positif. Les choix du modèle sont explicitement CG et Jacobi.
- Symétrie de chaque paire stockée : `abs(Kij/s − Kji/s) ≤ 10⁻¹²`, avec `s=max(abs(Kij),abs(Kji))`. Un coefficient absent vaut zéro. Aucune symétrisation silencieuse.
- Chaque diagonale est strictement positive et son inverse est fini. La division Jacobi utilise ensuite `z_i=r_i/Kii`.

Le seuil de symétrie est une politique numérique FerraSim. Ces contrôles ne constituent pas une preuve de définie-positivité ou de rang complet. Les matrices singulières à diagonale positive peuvent passer les contrôles initiaux ; une charge compatible, notamment nulle, peut même donner un résidu nul. Un succès numérique ne prouve donc ni l’unicité de la solution ni la suffisance des appuis. Aucune détection exhaustive des singularités n’est annoncée.

## Itérations et convergence

Le point initial est zéro. Le moteur applique les récurrences CG préconditionnées, avec `rho=rᵀz`, `pᵀKp`, les mises à jour de x/r et la direction conjuguée. Une courbure ou un produit préconditionné non positif hors convergence provoque un échec explicite. Les valeurs non finies, divisions non représentables et scalaires requis qui deviennent nuls par sous-flux sont traités comme ruptures numériques. Les produits scalaires sont séquentiels en f64 ; les normes utilisent `hypot` pour éviter les carrés intermédiaires excessifs. Une arithmétique intermédiaire non représentable peut être rejetée même si une autre mise à l’échelle permettrait le calcul.

Le critère vérifié est :

```text
norm(b - K*x) <= atol + rtol*norm(b)
```

Le résidu est recalculé avec la matrice CSR originale à chaque mise à jour ; le résidu récursif ne décide jamais seul du succès. Si le récursif passe sous le seuil mais pas le résidu recalculé, la direction est redémarrée avec ce dernier, sans remettre le compteur à zéro. Le critère est vérifié avant de déclarer le plafond atteint, donc une convergence à la dernière itération autorisée est acceptée.

Pour `b=0`, les contrôles de matrice restent actifs ; une matrice admise retourne x=0, zéro itération et résidu nul, y compris avec `atol=0`. Le résidu relatif est `None`, sans division 0/0. Un système entièrement prescrit donne un système réduit 0×0, accepté de la même façon avec des paramètres valides.

Aucun amortissement, régularisation, correction des paramètres ou repli vers un solveur dense n’est appliqué. La vérification systématique du résidu ajoute un produit CSR par itération ; cette première version privilégie une décision de convergence inspectable. Elle n’est pas une qualification de performance HPC ou de robustesse pour tous les conditionnements.

## Rapports et échecs

`Ok(SolveReport)` est réservé à `Converged` et contient solution réduite, nombre de mises à jour, norme du résidu frais, résidu relatif et seuil demandé. Une solution initiale admise compte zéro mise à jour.

`Err(SolveError)` ne contient pas de solution susceptible d’être exportée comme réussie. Il expose statut, catégorie précise, nombre de mises à jour et, lorsque calculables, résidu frais, résidu relatif et seuil. Le résidu relatif d’un échec vaut `None` si sa division n’est pas finie ou si le second membre est nul.

| Statut d’échec | Exemples de catégories |
| --- | --- |
| `InvalidInput` | Paramètres ou second membre invalides |
| `InvalidMatrix` | Paire non symétrique, diagonale Jacobi inutilisable |
| `NumericalBreakdown` | Arithmétique non représentable, courbure non positive, produit préconditionné non positif |
| `IterationLimit` | Résidu frais supérieur au seuil à la dernière mise à jour |

Les réactions et le résidu global restent calculés par `ReducedSystem` à partir du système original, après une résolution réussie. Les diagnostics de CG concernent le système réduit. Une API qui accepte un vecteur quelconque pour reconstruire le champ n’affirme pas sa convergence.

## Corpus et critères

Le [test CG](../crates/solvers/tests/cg.rs) compare la solution à une LU dense séparée sur quatre matrices symétriques strictement à diagonale dominante (tailles 2, 5, 17 et 40). Une matrice tridiagonale SPD de taille 2 000 utilise un champ exact construit et 5 998 entrées CSR. Le critère de solution fixé est `norm(x-xref)/max(norm(xref),1) < 10⁻¹⁰`, avec `rtol=10⁻¹²`, `atol=10⁻¹⁴`, budget 1 000. Le test recalcule le résidu et contrôle le seuil demandé.

Les [tests FEM](../crates/fem/tests/assembly.rs) résolvent les deux références exactes S04 (patch affine et système hétérogène forcé), comparent les déplacements complets à 10⁻⁸ et le résidu global/réactions à 10⁻¹⁰. Voir les échelles dans le [protocole S04](ASSEMBLY.md). Les contraintes du patch ont déjà été vérifiées dans le chemin dense S04 ; S05 vérifie l’accord des déplacements et réactions obtenus avec CG.

Le cas singulier de référence est `K=[[1,-1],[-1,1]], b=[1,1]` : diagonale valide, direction initiale dans le noyau, rejet pour courbure nulle. Sont aussi testés courbure négative, corps FEM libre sous résultante non nulle, budget insuffisant, second membre nul, critères absolu/relatif, système vide, forte variation de diagonale, asymétrie, diagonales invalides et débordements/sous-flux numériques. Un critère extrêmement strict contrôle également l’absence de succès fondé sur le seul résidu récursif. Ces exemples d’échec ne prouvent pas la détection de toutes les singularités.

Le [contrôle d’exécutable](../scripts/check_solver_failure.py) lance les scénarios singulier et budget limité dans des répertoires temporaires vides : sortie non nulle, stdout vide, diagnostic attendu et aucun fichier créé. Les exports de résultats et la commande `solve` ne sont pas encore implémentés ; leurs contrats devront reprendre cette exigence en S07/S08.

## Reproduction

```sh
cargo test -p ferrasim-solvers --test cg --locked -- --nocapture
cargo test -p ferrasim-fem --test assembly --locked cg_ -- --nocapture
cargo run -p ferrasim-solvers --example cg_reference --locked -- spd
python3 scripts/check_solver_failure.py
cargo run -p ferrasim-cli --example inspect_solver --locked -- examples/minimal/model.json
```

Le script cherche l’exemple construit dans `target/debug/examples`, avec l’extension `.exe` sur Windows. L’exemple `inspect_solver` applique une force unitaire explicite au premier degré de liberté libre dans l’ordre interne. **Il n’intègre pas les charges JSON**, ce qui relève de S06. Il n’écrit aucun fichier de résultat et affiche les données uniquement après succès de tous les calculs.

Les références et tests sont écrits pour FerraSim sous la licence MIT du dépôt. La LU séparée et les références rationnelles ne remplacent pas une revue scientifique par une personne compétente distincte de l’auteur. La revue reste ouverte avant qualification ; les bilans généraux, la poutre, la convergence de maillage et la comparaison externe restent à leurs sprints prévus.
