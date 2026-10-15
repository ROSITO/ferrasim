# S09 — Proposition v2, non adoptée

La v1 demeure le protocole accepté. Cette proposition est enregistrée avant les nouvelles mesures correspondantes ; son succès éventuel ne vaut ni validation de la v1 ni qualification scientifique.

La v1 ajoutait un contrôle CG à rtol=10⁻¹², sans étude préalable de la précision atteignable pour cette matrice en f64. Ce choix d'implémentation, ajouté au critère physique du PRD, confondait un résidu relatif très faible et la précision de la flèche recherchée. La correction du calcul des sommes permet de résoudre le niveau m=4 au seuil v1, mais ne garantit pas un tel seuil pour tous les raffinements. Aucun seuil interne au solveur ne sera relevé automatiquement.

La géométrie, matériau, appuis, traction, référence EB, intégration et échelles restent ceux de la [v1](CANTILEVER-PROTOCOL.md). Niveaux : m=2,4,8,12. Critères physiques inchangés : erreur décroissante, erreur finale ≤5 %, bilans force/moment/résidu libre/énergie ≤10⁻⁶.

Proposition de séparation algébrique : résoudre la famille à rtol=10⁻⁷, atol=10⁻¹², budget 30000 ; résoudre à nouveau le niveau fin à rtol=10⁻⁸ avec le même budget. Exiger le résidu frais demandé pour chaque exécution, puis une variation relative ≤10⁻⁶ sur la flèche moyenne, la norme L2 du vecteur complet de déplacements et l'énergie de déformation. Aucun succès si une de ces vérifications manque ou échoue. Cette sensibilité observée n'est pas une borne mathématique certifiée de l'erreur algébrique.

Justification théorique : les critères d'arrêt doivent tenir compte de l'erreur de calcul et de la précision physique visée ; un résidu rapporté seulement à la charge peut imposer un seuil inaccessible. Voir [Templates, critères d'arrêt](https://netlib.org/linalg/old_html_templates/section2.8.2.html) et [Greenbaum, précision atteignable des résidus récursifs](https://ecommons.cornell.edu/entities/publication/3239efa7-2f56-4918-ade8-e220cbffc626). Ces sources ne prouvent pas l'inaccessibilité de la tolérance pour ce cas particulier ; les échecs observés restent publiés.

Le pilote expose cette étude par `--protocol v2-proposed`. Il conserve la v1 par défaut et marque la v2 comme proposée, en attente de décision explicite. L'adoption d'un protocole de remplacement doit être distinguée de la revue scientifique indépendante, toujours requise avant qualification. La v1 interdit un assouplissement a posteriori : cette proposition ne sera donc pas utilisée pour déclarer silencieusement S09 validé.
