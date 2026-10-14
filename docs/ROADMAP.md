# FerraSim — Roadmap de développement par sprints

Version : 1.0 — plan proposé\
Date : 14 octobre 2026\
Statut : fondations S00 livrées, revue scientifique indépendante ouverte ; S01 et S02 terminés le 14 octobre 2026, voir le [rapport S02](sprints/S02.md) ; S03 à lancer

Références : [PRD MVP](PRD.md), [PRD de vision](PRD-VISION.md).

## 1. Organisation du plan

Deux parcours structurent la roadmap :

- **S00 à S11 : de zéro au MVP 0.1**, avec un backlog suffisamment précis pour lancer le développement.
- **S12 à S99 : du MVP au produit cible**, avec des objectifs de sprint, dépendances et portes de qualification. Ces sprints sont des hypothèses de découpage à réestimer avant chaque vague.

Documents d’exécution : [sprints MVP](ROADMAP-MVP.md) et [sprints produit cible](ROADMAP-PRODUIT.md).

Les identifiants S00…S99 désignent l’ordre du parcours de référence. Ils ne constituent pas la preuve qu’une capacité scientifique complexe tient en deux semaines. Si une porte de qualification n’est pas franchie, des sprints supplémentaires sont ajoutés, par exemple S30-A et S30-B, avant les étapes dépendantes. Un sprint se termine à sa date de fin ; une fonctionnalité inachevée retourne au backlog.

## 2. Hypothèses de capacité

La cadence proposée est de **deux semaines par sprint**. Aucune date de démarrage n’est fixée.

Pour le MVP, hypothèse de travail : deux développeurs Rust/calcul scientifique, avec une revue numérique indépendante disponible à chaque jalon. Ce n’est pas une description d’une équipe déjà constituée. Une personne seule doit réestimer les durées et obtenir une revue scientifique extérieure.

Pour le produit industriel, le programme suppose une équipe permanente en structure/calcul, plateforme/IO/HPC, GUI/Python et qualification, puis des spécialistes dédiés aux physiques ajoutées. Le nombre de personnes et le budget sont à déterminer avant financement des vagues. La capacité nécessaire à la vision finale dépasse celle retenue pour le MVP.

Sur chaque sprint, réservation de capacité proposée : 60 % développement, 25 % vérification/validation et documentation, 15 % corrections et imprévus. La validation constitue une partie du travail de livraison dès le premier sprint.

S00–S11 représentent nominalement 24 semaines si chaque objectif tient dans son sprint et que les moyens supposés sont disponibles. **Ce total est une enveloppe de planification à recalibrer à S00 et S03.** Il ne s’agit pas d’un engagement. La succession S12–S99 ne doit pas être convertie en date de livraison finale : charge par domaine, répétitions, disponibilité des pilotes et possibilités de travail simultané ne sont pas encore estimées.

## 3. Vue d’ensemble

| Étape | Sprints de référence | Résultat | Porte de sortie |
| --- | --- | --- | --- |
| V0 — Fondations | S00–S11 | MVP 0.1 : TET4, CLI/API Rust, import MSH et résultats vérifiés | G0 : définition de terminé du PRD MVP |
| V1 — Parcours structurel | S12–S23 | Python, GUI, CAO/maillage, éléments enrichis, modal et thermique séquentielle | G1 : C1 sur un parcours déclaré |
| V2 — Assemblages non linéaires | S24–S35 | États matériau, grandes transformations, contact et assemblages | G2 : qualification non linéaire sélectionnée |
| V3 — Dynamique et durabilité | S36–S49 | Dynamique, composites, fatigue, reprise et HPC structurel | G3 : C2 sur les parcours déclarés |
| V4 — Campagnes et optimisation | S50–S61 | Campagnes, calibration, optimisation et collaboration sécurisée | G4 : parcours de campagne qualifié |
| V5 — Extension des physiques | S62–S79 | CFD, EM, acoustique et multicorps dans l’environnement commun | G5 : qualification de chaque domaine sélectionné |
| V6 — Couplages et portefeuille | S80–S89 | Couplages industriels, HPC multi-domaines et stabilisation | G6 : C3 sur portefeuille explicite |
| Approfondissement de la vision | S90–S99 | Fonctions avancées et modules spécialisés du PRD | G7 : couverture cible auditée et domaines d’emploi publiés |

Une vague produit des versions candidates et incrémentales. Les numéros de version après 0.1 seront attribués selon les politiques de compatibilité ; aucun numéro commercial n’est déduit automatiquement d’un sprint.

## 4. Dépendances et chemin prioritaire

```text
Contrats / unités / validation
             |
TET4 -> assemblage -> résolution -> réactions et bilans
             |
Import + CLI + exports -> MVP qualifié
             |
Modèle versionné / Python / GUI / CAO et maillage -> C1
             |
État matériau -> cinématique non linéaire -> contact
             |
Dynamique / reprise / HPC / pilotes -> C2
             |
Campagnes et plateforme -> modules par physique
             |
Mapping et couplages -> qualification du portefeuille -> C3
             |
Extensions spécialisées -> audit de la vision cible
```

Le plan est séquentiel par défaut. Les pistes GUI, matériaux, HPC et physiques peuvent être menées simultanément seulement lorsque leurs contrats sont disponibles, qu’une équipe est affectée et que les dépendances de qualification sont respectées. Les numéros indiquent alors un ordre de dépendance, pas nécessairement des dates consécutives.

Les plateformes partagées doivent intégrer droits, isolation et gestion des secrets dès leur création. Les campagnes de sécurité, performance et documentation ne peuvent pas être repoussées au dernier sprint.

## 5. Définition commune d’un sprint terminé

1. L’objectif utilisateur est démontré avec un cas reproductible.
2. Les exigences retenues sont reliées à des critères d’acceptation et à des preuves.
3. Les modifications sont revues et les vérifications pertinentes passent.
4. Le domaine d’emploi, les limites et les changements d’API/format sont documentés.
5. Aucun défaut connu ne compromet le résultat démontré sans être explicitement bloquant.
6. Les erreurs, annulations et échecs du parcours sont pris en compte.
7. Les mesures nécessaires sont conservées : précision, bilans, ressources ou ergonomie selon le sujet.
8. Les changements du socle passent la CI Windows, macOS et Linux. Les dépendances et fonctions nouvelles sont vérifiées sur les trois systèmes ; les backends spécialisés publient leurs limites propres sans rendre le socle dépendant d’un seul OS.

Une fonctionnalité peut être livrée comme expérimentale sans ouvrir la porte industrielle correspondante. Un critère échoué ne devient pas acceptable par modification rétroactive de sa tolérance.

## 6. Pilotage et replanification

Avant le sprint : sélectionner un objectif compatible avec la capacité, préciser les tâches et références, identifier dépendances et risques. Les deux sprints suivants peuvent être préparés finement ; les sprints lointains restent au niveau objectif/livrable/preuve.

Pendant le sprint : rendre visibles travail en cours, obstacles et résultats. Une exploration scientifique est bornée et se termine par une décision, un prototype et une estimation, même si la méthode n’est pas retenue.

À la revue : montrer le cas utilisateur, examiner preuves et défauts, puis accepter ou reporter les éléments. À chaque porte G0…G7 : auditer le périmètre, réestimer la prochaine vague et vérifier ressources et dépendances.

Le suivi conserve : sprint, objectif, exigences liées, responsable, dépendances, statut, résultats de validation et décision de passage. Les rapports ne mélangent pas pourcentage de tâches terminées et maturité scientifique.

## 7. Règles de protection du périmètre

- Aucun ajout de GUI, STEP, Python, contact ou GPU avant G0, sauf travail de cadrage sans incidence sur le MVP.
- Une méthode non validée ne peut pas devenir un défaut implicite de production.
- Un solveur externe exige un connecteur qualifié, des versions et conditions de distribution documentées.
- Le financement et la disponibilité des experts sont vérifiés avant ouverture d’un nouveau domaine.
- Les intégrations qualifiées peuvent satisfaire les parcours C3 ; la réécriture native de toutes les physiques n’est pas une condition artificielle de livraison.
- La profondeur native ou intégrée souhaitée dans chaque domaine est décidée avant V5 et apparaît dans la matrice de couverture.

## 8. Première action exécutable

S02 est terminé pour l’import et les contrôles du maillage. Voir les [preuves et limites](sprints/S02.md). Le prochain travail est **S03 : élasticité isotrope et élément TET4** ; la revue scientifique indépendante des conventions de S00 et du seuil géométrique reste à organiser avant qualification.
