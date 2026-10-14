# Import de maillage et liaison modèle — S02

## Format pris en charge

Gmsh **MSH 2.2 ASCII**, en-tête `2.2 0 8`. Sections obligatoires : `MeshFormat`, `Nodes`, `Elements`. `PhysicalNames` est facultatif. Nodes, Elements et PhysicalNames peuvent se présenter dans un ordre différent après l’en-tête, mais chaque section n’apparaît qu’une fois. Les compteurs, marqueurs de fin et longueurs de records sont vérifiés.

Éléments acceptés : triangle linéaire type 2 et tétraèdre linéaire type 4. Les identifiants externes sont positifs et peuvent être non contigus ou désordonnés. Les IDs d’éléments sont uniques entre surfaces et volumes ; aucun doublon ne remplace silencieusement un record.

Les deux premiers tags sont conservés comme tag physique et tag d’entité élémentaire. Zéro signifie absence de tag. Les records sans tags ou avec un seul tag sont acceptés ; plus de deux tags sont rejetés, car partitions et métadonnées supplémentaires ne sont pas encore implémentées.

Les noms physiques de dimensions 2 et 3 sont conservés avec leurs tags ; ils sont uniques par dimension. Les tags physiques sans nom restent accessibles sur les éléments, sans nom inventé. Ils ne suffisent pas à une affectation matériau par nom dans le modèle.

**Refus explicites :** MSH 4.x, binaire, autre taille de donnée, autres topologies, dimensions physiques 0/1, partitions, sections périodiques ou autres sections non implémentées. Les noms Unicode et fins de ligne Windows sont supportés. Un en-tête binaire est diagnostiqué avant décodage UTF-8 du payload.

Référence du format : [manuel officiel Gmsh — MSH ASCII 2](https://gmsh.info/doc/texinfo/gmsh.html#MSH-ASCII-file-format). Les limitations ci-dessus définissent le sous-ensemble FerraSim ; elles ne prétendent pas décrire tout le format Gmsh.

## Géométrie et topologie

- Coordonnées finies, IDs cohérents, références nodales présentes et connectivités à nœuds distincts.
- Au moins un nœud et un TET4 ; aucun nœud sans appartenance à un volume.
- Aucun doublon de connectivité TRI3 ou TET4.
- Orientation TET4 positive ; les éléments inversés sont rejetés sans permutation.
- Dégénérescence relative : `|det(J)| / Lmax³ ≤ 10⁻¹²`, où `Lmax` est la plus grande des six arêtes.
- Volumes et aires calculés doivent être strictement positifs et représentables en f64.
- Chaque triangle doit être une face d’un ou deux TET4. Une face de plus de deux volumes, ou une adjacence de deux volumes du même côté de leur face partagée, est rejetée.

Les différences de coordonnées sont normalisées avant calcul du déterminant ; les tests vérifient le même comportement à plusieurs échelles. Le seuil est une politique initiale du MVP, pas une garantie générale de qualité de maillage ou de conditionnement FEM. Sa revue scientifique reste nécessaire avant qualification des formulations S03.

Le contrôle ne détecte pas toutes les intersections entre éléments sans face commune, ni tous les défauts géométriques globaux. Un maillage géométriquement admissible ne garantit pas que les contraintes suppriment les modes rigides.

## Surfaces, orientations et groupes nodaux

Les triangles d’une interface interne sont conservés, mais ne peuvent pas recevoir la traction de frontière du modèle 0.1. Ils peuvent fournir des nœuds pour une prescription.

Pour une face extérieure, la normale et l’ordre orienté extérieur sont déduits du TET4 adjacent. La connectivité du triangle source reste inchangée. `BoundaryFace::source_is_outward` signale son orientation d’origine ; une orientation entrante du triangle seul n’est pas une erreur, car elle peut dépendre de l’orientation de l’entité surfacique du fichier.

Un groupe nodal est l’ensemble trié, sans doublon, des nœuds du groupe physique surface **ou** volume portant le même nom. Si ce nom existe dans les deux dimensions, la dérivation est ambiguë et refusée, sans union automatique. Aucune sélection n’utilise une recherche approximative par coordonnées.

Les groupes physiques nommés vides restent visibles. Une sélection vide utilisée par matériau, prescription ou traction est rejetée. Une surface vide déclarée mais inutilisée peut être conservée.

## Liaison au modèle

`ValidatedMesh::validate_model` contrôle le contrat S01 puis :

1. Les groupes attendus existent dans le maillage, et les groupes nodaux ne sont pas ambigus.
2. Les sélections effectivement utilisées sont non vides.
3. Chaque TET4 importé est couvert par une affectation matériau du modèle ; aucun volume n’est exclu silencieusement.
4. Les charges nodales pointent vers des nœuds présents.
5. Les tractions portent uniquement sur des triangles extérieurs.
6. Les groupes nodaux qui se recouvrent n’imposent pas des déplacements différents au même nœud/composante.

Les matériaux sont déjà vérifiés par S01. Les matrices, réactions, résolution et singularités mécaniques restent aux sprints suivants. Le contrôle anticipé des prescriptions sur groupes chevauchants ne modifie pas le besoin de construire les contraintes en S04.

## API et démonstration

- `parse_msh22(&str)` et `parse_msh22_bytes(&[u8])` : parsing puis validation géométrique.
- `load_msh22(path)` : lecture du fichier, avec erreurs portant chemin et source.
- `Mesh::into_validated()` : contrôle d’un maillage construit depuis Rust.
- `load_meshed_model(path)` : modèle JSON, import source-relatif et validation des références.

`ValidatedMesh` conserve les records et métadonnées sans accès mutable permettant d’invalider leurs contrôles. Les erreurs de format portent catégorie et ligne ; géométrie et liaison au modèle produisent des listes déterministes avec IDs et champs affectés.

```sh
cargo run -p ferrasim-io --example inspect_mesh --locked -- examples/minimal/model.json
```

Résultat attendu du modèle minimal : quatre nœuds, un tétraèdre, un triangle extérieur, volume 1/6 et aire 1/2 dans les unités de la fixture. Ce cas est un test géométrique, pas le démonstrateur de poutre ni une simulation FEM.

Les fixtures [TET4](../validation/fixtures/tet4-positive.msh) et [interface à deux volumes](../validation/fixtures/two-tets-interface.msh) sont créées pour le dépôt sous sa licence MIT.
