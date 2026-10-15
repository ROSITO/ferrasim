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

## Mesures v2 proposées — macOS ARM64

[Résultats archivés](../../validation/results/cantilever/v2-proposed.json), binaire release du correctif `01f25bc`, Rust 1.96.0. Quatre niveaux résolus à rtol=10⁻⁷ :

| m | TET4 | Erreur de flèche EB | Temps CLI (s) | Itérations |
| --- | ---: | ---: | ---: | ---: |
| 2 | 960 | 49.4832 % | 0.29 | 626 |
| 4 | 7680 | 20.7950 % | 1.11 | 1445 |
| 8 | 61440 | 6.4652 % | 15.95 | 3185 |
| 12 | 207360 | 3.0934 % | 76.50 | 4816 |

Nouvelle résolution de m=12 à rtol=10⁻⁸ : 86.31 s et 5217 mises à jour CG. Les critères de résidu frais demandés passent sur les cinq exécutions. Maximum des bilans normalisés contrôlés : 3.1303e-08, sous 10⁻⁶. Variation relative de flèche : 2.4348e-11 ; du champ complet : 1.9511e-11 ; de l'énergie : 4.8697e-11. Chacune reste sous 10⁻⁶. Ces nombres décrivent une sensibilité observée entre deux résolutions, pas une borne certifiée de l'erreur par rapport à la solution exacte du système.

Le seuil physique de 5 % est satisfait au niveau fin, avec une erreur de 3,0934 %. Temps exploratoires : une exécution par niveau, parcours CLI complet, compilation et génération exclues ; les autres activités de la machine peuvent influencer la durée. Aucun engagement de performance. La flèche et les critères restent les mêmes sur chaque maillage ; aucun résultat de contrainte à l'encastrement n'est utilisé pour l'acceptation.

## Décision restant ouverte

La proposition v2 est prête à être examinée. La v1 précisait « aucun assouplissement a posteriori » : elle ne peut être déclarée réussie sur la base de ces nouveaux paramètres. L'adoption explicite de la v2 permettrait une clôture technique selon les critères physiques inchangés et les nouveaux contrôles algébriques publiés. Elle ne constitue pas une revue scientifique indépendante ni une qualification industrielle.

## Vérification multiplateforme

La [CI du correctif `01f25bc6b78cd490bdf47c02141a2a7e300454de`](https://github.com/ROSITO/ferrasim/actions/runs/38714245667) passe sur Windows, macOS et Linux. Elle comprend les contrôles existants, les deux nouvelles références binaires exactes, la régression stricte m=4, les quatre niveaux v2 proposés, le second calcul fin et l'audit Decimal. Tests Rust : 122 sur Windows et macOS, 123 sur Linux (test de chemin non UTF-8 supplémentaire). Clippy, formatage, release, documentation, schéma JSON et références antérieures passent.

L'écart relatif de norme entre résidu compensé et référence Decimal vaut 5,12×10⁻⁹ sur Windows, 5,60×10⁻⁹ sur macOS et 4,35×10⁻⁹ sur Linux, sous le seuil d'audit de 10⁻⁶. Ce seuil contrôle le calcul de norme du système arrondi, pas l'erreur physique. Les vérifications de sensibilité et les bilans v2 passent sur les trois systèmes. Le statut reste « proposition passée, adoption et revue indépendantes en attente ».

Les empreintes des entrées des essais archivés se reproduisent avec le générateur livré, y compris celles du calcul fin resserré. L'empreinte du pilote de la série v2 correspond exactement au fichier commis dans le correctif ; la révision Git relevée au début de mesure et la révision d'implémentation sont distinguées dans les métadonnées.
