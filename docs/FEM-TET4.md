# Élasticité isotrope et élément TET4 — S03

Statut : implémentation et vérification numérique élémentaire ; revue scientifique indépendante encore ouverte. Aucune résolution globale n’est livrée ici.

## Conventions et matériau

`IsotropicElastic::new(E, nu)` exige E fini strictement positif et −1 < ν < 0,5. Les coefficients dérivés doivent être représentables ; le tenseur normalisé doit admettre une factorisation de Cholesky à la précision de travail. Un jeu physiquement admissible mais numériquement inutilisable retourne une erreur.

La contrainte suit `sigma = lambda*tr(epsilon)*I + 2*mu*epsilon`, avec `mu = E/(2*(1+nu))` et `lambda = E*nu/((1+nu)*(1−2*nu))`. Le module volumique est `E/(3*(1−2*nu))`. Les matériaux auxétiques sont autorisés : lambda peut être négatif.

Ordre Voigt : xx, yy, zz, xy, yz, xz. Les déformations de cisaillement sont les valeurs d’ingénieur gamma = 2 epsilon, et les termes de cisaillement du tenseur valent mu. Toutes les unités restent celles du modèle, sans conversion automatique.

`warnings()` signale `NearIncompressible` pour ν ≥ 0,49. C’est un seuil d’avertissement initial, pas une borne physique, une correction de verrouillage ni une qualification de TET4 dans ce régime. Aucun paramètre n’est modifié automatiquement. Le code appelant doit rendre ces diagnostics visibles.

## Cinématique

`Tet4Element::from_mesh` conserve l’ordre des nœuds d’un `ValidatedMesh`. `from_coordinates` applique la même validation géométrique S02, y compris orientation positive et seuil relatif de dégénérescence 10⁻¹².

Les fonctions de forme sont `1−r−s−t, r, s, t`. Le Jacobien a pour colonnes les trois arêtes depuis le premier sommet. Les gradients globaux sont obtenus avec son inverse transposé. Le calcul de l’inverse utilise un Jacobien normalisé par la plus grande arête.

Pour un gradient de forme `(gx, gy, gz)` et les trois colonnes d’un nœud, le bloc de B est :

```text
xx : gx   0    0
yy : 0    gy   0
zz : 0    0    gz
xy : gy   gx   0
yz : 0    gz   gy
xz : gz   0    gx
```

Les déplacements sont ordonnés `[u1x,u1y,u1z,...,u4x,u4y,u4z]`. B est constant ; déformation et contrainte sont constantes par élément.

## Rigidité et réponse locale

La rigidité est l’intégrale `Bᵀ C B`, exacte ici avec les champs constants. L’implémentation utilise des opérateurs normalisés pour éviter de former prématurément B² dans les unités physiques. La matrice 12×12 est retournée sans symétrisation artificielle.

`strain(u)` calcule la déformation d’ingénieur. `response(material,u)` retourne déformation, contrainte et énergie intégrée `0,5 * epsilon_voigt.dot(sigma_voigt) * volume`. Ce sont des évaluations d’un champ fourni, pas la solution d’un chargement.

Les entrées non finies, débordements et sous-flux totaux détectables de contrainte, rigidité ou énergie retournent `FemError`. Cela ne constitue pas une analyse exhaustive du conditionnement ni une garantie d’exactitude de chaque très petite composante f64. Le TET4 non contraint possède six modes rigides ; il ne doit pas être inversé comme un système défini positif.

## Vérifications et indépendance des références

| Vérification | Référence et critère |
| --- | --- |
| Matériau | Traction uniaxiale libre latéralement, trois cisaillements, compression/dilatation uniforme ; cas auxétiques inclus |
| Fonctions de forme | Valeurs nodales, partition de l’unité et interpolation affine |
| Matrice dense complète | Trois cas exacts rationnels, erreur relative ≤ 10⁻¹² |
| Symétrie | Norme de Frobenius relative ≤ 10⁻¹⁰, sans corriger la matrice |
| Modes rigides | Trois translations et trois rotations infinitésimales ; `norm(Kr)/(norm(K)*norm(r)) ≤ 10⁻¹⁰` |
| Rang | Six valeurs propres numériquement nulles et six positives sur éléments bien conditionnés |
| Champ affine local | Comparaison au tenseur de déformation symétrique et à la loi tensorielle, erreur ≤ 10⁻⁸ |
| Énergie | Accord entre continuum, réponse et `0,5 uᵀKu`, erreur relative ≤ 10⁻¹⁰ |
| Covariance | Changement de repère rigide ; changement d’échelle géométrique et de module |
| Erreurs | Géométrie invalide, constantes interdites, non-finis et résultats non représentables |

La [référence dense](../validation/fixtures/tet4-reference.json) est produite par un [script séparé](../validation/generate_tet4_reference.py) : inversion rationnelle d’un polynôme affine 4×4, puis contraction du tenseur élastique d’ordre quatre. Elle n’utilise ni la matrice B, ni la bibliothèque Rust, ni une bibliothèque d’algèbre numérique. Les cas couvrent tétraèdre unité, tétraèdre oblique translaté et matériau auxétique. Les gradients et 144 coefficients de rigidité sont enregistrés comme fractions exactes.

Cette indépendance de calcul aide à repérer une erreur d’implémentation, mais n’est pas une revue par une personne indépendante. Le patch test multiélément avec assemblage reste à réaliser en S04 ; le test affine local ne le remplace pas. Les comparaisons à un solveur externe restent prévues en S10.

## Reproduction

```sh
python3 validation/generate_tet4_reference.py --check
cargo test -p ferrasim-fem --locked -- --nocapture
cargo run -p ferrasim-cli --example inspect_element --locked -- examples/minimal/model.json
```

`inspect_element` utilise uniquement la géométrie et le matériau d’un élément du modèle. Il impose artificiellement une déformation uniaxiale de 0,001, avec contraction latérale de Poisson. Les charges et conditions limites JSON ne sont pas résolues. Pour la fixture unité acier : sigma_xx attendu = 210 MPa et énergie = 17 500 J ; les composantes latérales résiduelles sont soumises aux erreurs d’arrondi.

## Sources et limites de qualification

Références théoriques : [A. F. Bower — élasticité linéaire](https://solidmechanics.org/text/Chapter3_2/Chapter3_2.htm) et [formulation FEM élastostatique](https://solidmechanics.org/text/Chapter8_1/Chapter8_1.htm). Les codes, fixtures et scripts sont créés dans ce dépôt ; aucun code de ces sources n’est copié.

La revue indépendante des conventions, du seuil géométrique et des preuves reste ouverte avant qualification scientifique S03. Ni ces tests ni leur réussite multi-plateformes ne valent qualification d’un solveur industriel.
