# S09 — Protocole v2 adopté

Adopté le 15 octobre 2026 après examen des résultats et accord de l'utilisateur pour avancer. Clôture technique S09 ; cet accord de pilotage ne constitue pas une revue scientifique indépendante. La référence de poutre reste approchée ; aucune qualification industrielle n'est revendiquée.

## Conditions et critères

Poutre de longueur 1 m et section carrée 0,05 × 0,05 m ; E=210 GPa, ν=0,3. Encastrement complet à x=0, traction uniforme globale t_z=−40000 Pa à x=1, résultante F_z=−100 N. Autres faces libres. Échelles de bilan : 100 N et 1 m ; origine nulle.

Maillages structurés emboîtés : nx=20m, ny=nz=m, six TET4 par cellule, m=2,4,8,12. Observable : moyenne surfacique de u_z à x=1, intégration du champ linéaire sur les triangles. Référence Euler–Bernoulli −FL³/(3EI)=−0,0003047619047619 m, I=bh³/12.

- Au moins trois raffinements, erreur de flèche décroissante et erreur finale ≤5 %.
- Bilans normalisés de force, moment, résidu libre et énergie ≤10⁻⁶, sur chaque résolution.
- Famille résolue par CG/Jacobi à rtol=10⁻⁷, atol=10⁻¹², budget 30000, avec résidu frais sous le seuil demandé.
- Niveau fin résolu de nouveau à rtol=10⁻⁸ ; variations relatives de flèche moyenne, norme du champ nodal complet et énergie chacune ≤10⁻⁶.

Cette sensibilité observée n'est pas une borne certifiée de l'erreur algébrique. Un échec de l'un de ces critères suspend la livraison. Les contraintes singulières à l'encastrement ne servent pas de critère de précision.

## Reproduction et traçabilité

Après activation du toolchain Rust :

```sh
cargo build --workspace --release --locked
python3 validation/run_cantilever.py --output target/s09-v2
```

La v2 et ses quatre niveaux sont désormais le choix par défaut ; `--protocol v2` est explicite, `--protocol v1` reproduit l'ancien protocole. `--diagnostic-rtol` ne qualifie aucun sprint ; `--generate-only` prépare seulement les entrées. Répertoires neufs requis.

Le pilote conserve empreintes du binaire et des entrées, environnement, paramètres CG, résidus, bilans et sensibilité. Temps mural du parcours CLI complet et pic RSS par processus (macOS/Linux ; indisponible sous Windows), sans engagement de performance.

La [proposition enregistrée](CANTILEVER-PROTOCOL-V2-PROPOSED.md), les [mesures antérieures à l'adoption](results/cantilever/v2-proposed.json) et les échecs de la [v1](CANTILEVER-PROTOCOL.md) restent inchangés dans l'historique. Le [rapport de résolution](../docs/sprints/S09-RESOLUTION.md) explique le changement de protocole et les vérifications natives Windows/macOS/Linux. L'adoption ne transforme pas les échecs v1 en succès et ne remplace pas la comparaison externe prévue en S10.
