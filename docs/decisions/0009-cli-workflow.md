# ADR 0009 — Orchestration commune et erreurs par étape

Date : 15 octobre 2026. Statut : implémenté et vérifié par CI sur Windows, macOS et Linux, revue scientifique indépendante ouverte.

## Décision

La CLI ne contient pas de physique. IO expose validation et calcul de fichier complets, réutilisables sans CLI. Les erreurs distinguent options, entrée/préparation, numérique et sortie, avec les codes 2/3/4/5 ; zéro est réservé au succès.

Validation et résolution ont des contrats distincts : la première ne lance pas CG et ne garantit pas la solvabilité. Les paramètres solveur restent dans le modèle. Les échelles de bilan peuvent être explicites ou dérivées selon une règle publiée ; elles ne changent pas la physique.

Les chemins restent natifs. Le maillage est relatif au modèle, la sortie au répertoire de travail. La destination est nouvelle, sans création de parents ou écrasement ; le manifeste S07 est publié en dernier. Le succès utilisateur n’est annoncé qu’après publication complète.

## Conséquences

Pas de nouvelle dépendance externe. L’exemple Rust et la CLI utilisent le même chemin d’orchestration, avec une préparation d’assemblage supplémentaire documentée. Les contrôles de fichiers et les erreurs typées permettent de comparer les sorties et de vérifier l’absence de faux succès sur les trois OS.

Voir le [contrat CLI/API](../CLI-API.md) et le [rapport S08](../sprints/S08.md). Le parcours utilisateur n’acquiert pas à lui seul une qualification scientifique.
