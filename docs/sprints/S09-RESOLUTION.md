# S09 — Résolution numérique et protocole proposé

Statut : correction livrée ; v1 toujours non validée, proposition v2 distincte et non adoptée. Revue scientifique indépendante ouverte.

## Correction

Les produits CSR et les produits scalaires CG compensent les erreurs de produit et de somme. Les incréments des déplacements sont compensés ; leur compensation est remise à zéro lors du redémarrage sur le résidu frais. Le solveur conserve le critère demandé et vérifie le vecteur `f64` retourné ; aucun relèvement silencieux de tolérance. Voir [ADR 0010](../decisions/0010-compensated-cg.md).

Deux tests binaires exacts couvrent la perte d'un terme faible par annulation et l'arrondi d'un produit. Les références FEM et tests de résidu existants restent requis. La correction ne transforme pas le moteur en solveur de précision arbitraire.

## Résultat strict et audit

Le niveau m=4 passe à rtol=10⁻¹⁰ en 1730 mises à jour, au lieu d'échouer à 30000 avant correction. L'[audit Decimal à 100 chiffres](../../validation/results/cantilever/residual-audit.json) donne une norme de 2,061526159962095×10⁻⁹ pour la matrice et les déplacements exportés par l'exemple ; le solveur donne 2,0615261484243665×10⁻⁹. Écart relatif des normes : 5,60×10⁻⁹. Une accumulation naïve donne 4,194863635533417×10⁻⁹ sur les mêmes valeurs et surestime ici la norme d'environ un facteur deux. Le contrôle Decimal est distinct du calcul Rust mais produit par le même auteur ; il ne remplace pas la revue humaine indépendante.

Le [nouvel essai v1](../../validation/results/cantilever/corrected-v1.json) passe m=2 puis m=4, mais échoue à m=8 après 30000 mises à jour : résidu frais 3,1241162527726625×10⁻⁹, seuil 1,173164316138219×10⁻⁹. La v1 reste donc en échec. Ce constat ne prouve pas que tout algorithme f64 échouerait ni qu'une tolérance donnée est mathématiquement inaccessible.

Reproduction de l'audit, après construction release de l'exemple :

```sh
cargo build -p ferrasim-cli --example audit_residual --release --locked
python3 validation/run_cantilever.py --output target/s09-strict-regression --diagnostic-rtol 1e-10 --levels 1 2 4
python3 validation/audit_residual.py target/s09-system.json --run-model target/s09-strict-regression/m4/model.json --check
```

Le JSON intermédiaire du système réduit est volumineux et reste dans `target/`. Son empreinte est conservée dans l'audit. Les matrices, déplacements et chargements sont les valeurs `f64` assemblées ; l'audit ne valide ni l'assemblage ni la solution continue.

## Proposition de protocole v2

La [proposition enregistrée avant les nouvelles mesures](../../validation/CANTILEVER-PROTOCOL-V2-PROPOSED.md) conserve la référence, les charges, le seuil de 5 % et les bilans à 10⁻⁶. Elle remplace les tolérances CG v1 ajoutées sans étude préalable par une étude explicite de sensibilité : famille à rtol=10⁻⁷, niveau fin à rtol=10⁻⁸ ; variation de flèche, norme du champ complet et énergie chacune ≤10⁻⁶.

Une sensibilité mesurée n'est pas une borne mathématique certifiée. La v1 reste disponible par défaut ; `--protocol v2-proposed` produit un statut distinct en attente d'adoption, même lorsque tous ses contrôles passent. Aucun résultat v2 ne sera présenté comme une réussite de la v1.

```sh
python3 validation/run_cantilever.py --output target/s09-v2-review --protocol v2-proposed --levels 2 4 8 12
```
