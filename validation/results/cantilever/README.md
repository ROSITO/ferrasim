# Mesures exploratoires S09

- [Protocole v1](protocol-v1.json) : premier maillage résolu, puis échec CG au niveau m=4 ; aucune sortie de résultats publiée pour le niveau échoué.
- [Diagnostic](diagnostic.json) : quatre niveaux, rtol=10⁻⁸, erreur de flèche finale 3,0934 %. Ce mode ne valide pas S09.
- [Sonde du budget CG](residual-budget.json) : échecs à 1000, 2000, 4000 et 8000 itérations, tous les autres paramètres conservés.
- [Courbe](convergence.png) : données du diagnostic, tracées avec matplotlib 3.11.2 ; [script](../../plot_cantilever.py).

Le binaire Rust n'a pas été modifié depuis la révision enregistrée. Le pilote Python évoluait dans l'arbre de travail : l'empreinte du pilote, lorsqu'elle est présente, identifie son état au début de la mesure ; la révision Git identifie le socle Rust, pas une version commise du pilote. Les empreintes MSH/JSON permettent de contrôler les entrées régénérées avec le pilote livré. Aucun temps n'inclut la compilation ou la génération des entrées. Les mesures sont locales, sans qualification de performance ; les ressources disponibles et les tâches système peuvent les influencer.

Les rapports volumineux, maillages et journaux sont conservés localement sous `target/s09-protocol-v1/` et `target/s09-diagnostic-final/` ; ils ne sont pas distribués dans Git. La génération et la CLI permettent de les recréer dans un répertoire neuf. Les synthèses contiennent les observables, bilans, paramètres CG et empreintes des entrées ; elles ne remplacent pas une référence de solveur externe ni une revue scientifique indépendante.

## Correctif et proposition v2

- [Essai v1 corrigé](corrected-v1.json) : m=4 passe, m=8 échoue encore au seuil v1.
- [Audit à 100 chiffres](residual-audit.json) : résidu du système f64 assemblé, calculé séparément avec Decimal.
- [Proposition v2 mesurée](v2-proposed.json) : quatre niveaux et nouvelle résolution fine avec tolérance resserrée, en attente d'adoption explicite.

Voir le [rapport de résolution](../../../docs/sprints/S09-RESOLUTION.md). Les résultats historiques ci-dessus restent inchangés. Les métriques v2 ne sont pas une réussite du protocole v1.
