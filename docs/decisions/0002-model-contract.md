# ADR 0002 — Contrat JSON et validation par étapes

Date : 14 octobre 2026. Statut : adopté en S01.

## Décisions

- Version exacte `0.1`, rejet du brouillon et des versions inconnues avant interprétation du schéma courant.
- Types de modèle dans `ferrasim-model`, lecture et sérialisation JSON dans `ferrasim-io`.
- Champs inconnus et clés connues répétées rejetés ; aucun remplacement silencieux des données physiques.
- Catalogue explicite des groupes nodaux, surfaciques et volumiques dans le modèle.
- Validation des références déclarées en S01 ; contrôle des membres réels, nœuds et groupes vides après import S02 ; compatibilité des affectations/contraintes avec la topologie en S04.
- Déplacements facultatifs par composante ; absence ou `null` signifie libre. Vecteurs de charge complets.
- Identifiants métier stables et typés ; chaînes comparées exactement, nœuds `u64` positifs distincts des indices internes.
- SI ou labels d’unités cohérentes explicites, sans conversions automatiques.
- Chemins de maillage relatifs au fichier modèle, à séparateurs `/`, autorisant `..` ; aucune sandbox implicite.
- Erreurs sémantiques collectées et déterministes avec code, champ et message.
- Validation avant sérialisation publique pour éviter la conversion silencieuse des valeurs non finies en JSON null.
- JSON Schema indépendant pour outils/éditeurs, complété par les règles sémantiques Rust.

## Conséquences

Un contrat valide peut encore référencer un fichier manquant, un groupe déclaré mais absent du maillage ou un système singulier. L’API et l’exemple l’indiquent explicitement. Cette séparation évite de prétendre avoir validé un maillage avant l’existence de l’importeur.

Le catalogue évite les références opaques dans S01 et fournit les attentes à vérifier en S02. Il sera conservé dans le format 0.1 ; toute modification incompatible nécessite une nouvelle version.

Les fonctions d’IO sont utilisables sans CLI. Le parcours utilisateur `ferrasim validate/solve` reste affecté à S08, après disponibilité de l’import et du moteur.
