# FerraSim — Sprints S00 à S11 : de zéro au MVP

Références : [roadmap générale](ROADMAP.md), [PRD MVP](PRD.md).

Cadence proposée : deux semaines. Fondations S00 livrées ; S01/S02 terminés ; S03 livré techniquement le 15 octobre 2026, qualification indépendante ouverte ; S04 livré techniquement le 15 octobre 2026 sur les trois OS, revue scientifique indépendante ouverte ; S05 livré techniquement le 15 octobre 2026 sur les trois OS, revue scientifique indépendante ouverte ; S06 lancé le 15 octobre 2026, vérifications locales terminées et CI à consigner ; S07 et suivants à faire. Voir les rapports [S00](sprints/S00.md), [S01](sprints/S01.md), [S02](sprints/S02.md), [S03](sprints/S03.md), [S04](sprints/S04.md), [S05](sprints/S05.md) et [S06](sprints/S06.md). Le détail constitue un backlog proposé, à réestimer avec la capacité réelle.

## S00 — Cadrage d’exécution et initialisation

**Objectif :** rendre le premier développement reproductible et les choix structurants explicites.

Livrables :

- Cargo workspace minimal correspondant au PRD ; règles locales de contribution.
- Choix de licence et examen des dépendances candidates, notamment algèbre creuse et sérialisation.
- Décisions documentées sur unités SI des exemples, Voigt/cisaillement, repères et orientation TET4.
- Définition des cas de validation, métriques, tolérances et sources avant calcul des résultats.
- Configuration formatage, analyse statique, tests et builds ; matrice de plateformes réellement vérifiées.
- Réestimation du MVP selon équipe, disponibilité de revue et inconnues numériques.

**Sortie :** workspace compilable depuis un environnement propre ; exemples d’entrées et protocole de vérification approuvés en revue. Les choix encore ouverts ont un responsable et une date de décision.

**Dépendances :** aucune. **Traçabilité :** jalon J0 ; PRD sections 8–12 et 15.

## S01 — Modèle et contrats d’entrée

**Objectif :** représenter une étude sans GUI ni solveur.

Livrables : identifiants, groupes, matériau et affectations ; prescriptions par composante ; forces nodales et traction surfacique ; paramètres de résolution ; modèle JSON versionné ; erreurs structurées ; règles de chemins relatifs au fichier de modèle.

**Sortie :** modèle minimal sérialisable et rechargé sans perte ; version inconnue, références invalides et données non finies rejetées ; un exemple valide sert de contrat pour les prochains sprints.

**Dépendances :** S00. **Traçabilité :** MOD-01, MOD-02, MAT-01, BC-01 et modèle de données.

## S02 — Maillage et import Gmsh

**Objectif :** charger un domaine TET4 et ses sélections de frontière.

Livrables : import MSH 2.2 ASCII ; nœuds, triangles et tétraèdres ; groupes physiques ; dérivation documentée des groupes nodaux ; détection connectivité invalide, volumes dégénérés et orientations ; vérification que les triangles chargés sont des faces frontières admissibles.

**Sortie :** petits maillages de référence importés avec groupes conservés ; éléments et formats non supportés rejetés ; orientation traitée selon une politique explicite ; cas invalides donnent des diagnostics localisables.

**Dépendances :** S01. **Traçabilité :** MSH-01, MSH-02, MOD-02.

## S03 — Élasticité isotrope et élément TET4

**Objectif :** établir une formulation élémentaire vérifiée.

Livrables : matrice constitutive isotrope, fonctions de forme, Jacobien, matrice B et rigidité ; validation du matériau ; contrat de tenseurs commun ; référence dense indépendante pour petits tests ; fiche de limites de l’élément.

**Sortie :** symétrie et six modes rigides vérifiés ; champ affine reproduit au niveau élémentaire ; tolérances élémentaires du PRD satisfaites. Revue numérique et réestimation avant assemblage.

**Dépendances :** S00, S02. **Traçabilité :** MAT-01, FEM-01 ; première partie de J1.

## S04 — Assemblage creux et prescriptions

**Objectif :** construire un système global correct avec contraintes cinématiques.

Livrables : numérotation des degrés de liberté ; assemblage creux ; affectations matériau exhaustives ; réduction par élimination ; prescriptions nulles et non nulles ; conservation de la formulation originale pour réactions ; petit solveur dense de référence réservé aux validations.

**Sortie :** patch test multiélément et déplacements non nuls passent avec la référence dense ; prescriptions contradictoires et affectations ambiguës rejetées.

**Dépendances :** S01–S03. **Traçabilité :** BC-01, FEM-01 ; fin J1 et début J2.

## S05 — Résolution CG/Jacobi et diagnostics

**Objectif :** résoudre le système réduit sans matrice dense en production.

Livrables : CG préconditionné Jacobi ; tolérances relative/absolue et plafond d’itérations ; résidu recalculé ; cas second membre nul ; validation diagonale et détection des ruptures numériques ; statut, itérations et erreurs explicites.

**Sortie :** accord avec référence dense sur corpus bien conditionné ; cas singulier de référence et non-convergence donnent un échec clair ; aucun fichier n’annonce un résultat réussi après échec.

**Dépendances :** S04. **Traçabilité :** SOL-01, SOL-02, FEM-01.

## S06 — Charges et équilibre mécanique

**Objectif :** appliquer des charges dont la résultante est vérifiable.

Livrables : forces nodales sans duplication ; traction uniforme intégrée sur triangles ; calcul de réactions à partir du système original ; bilans forces/moments et énergie de déformation ; références de traction et de chargement surfacique.

**Sortie :** résultante surfacique, traction affine, réactions et équilibre passent aux tolérances du PRD ; charges sur surfaces invalides rejetées.

**Dépendances :** S02, S04, S05. **Traçabilité :** LOAD-01, LOAD-02, RES-01 ; fin J2.

## S07 — Champs de résultats et exports

**Objectif :** produire des résultats mécaniques inspectables.

Livrables : déplacements nodaux ; déformations et contraintes constantes par élément ; von Mises ; réactions ; export VTU et CSV ; rapport JSON avec provenance, paramètres, résidu et bilans ; publication des conventions de composantes et de localisation.

**Sortie :** champs comparés à une référence ; fichiers VTU effectivement ouverts et contrôlés dans un visualiseur compatible ; CSV/JSON cohérents avec l’API ; aucune contrainte nodale lissée présentée comme donnée brute.

**Dépendances :** S03, S06. **Traçabilité :** RES-01, IO-01.

## S08 — CLI et API Rust de bout en bout

**Objectif :** rendre la chaîne utilisable depuis un modèle JSON.

Livrables : commandes `validate` et `solve` ; répertoire de sortie ; codes d’erreur distincts ; chemins d’entrée robustes ; exemple Rust reproduisant la CLI ; comportement documenté pour sorties existantes ; statut de résultats complets publié après succès.

**Sortie :** utilisateur reproduisant un calcul depuis un environnement propre ; entrées incorrectes et échecs solveur signalés correctement ; comparaison des résultats CLI et API dans les mêmes tolérances.

**Dépendances :** S01–S07. **Traçabilité :** CLI-01, API-01 ; J3.

## S09 — Démonstrateur de poutre et convergence

**Objectif :** montrer et documenter la précision et les limites de la chaîne.

Livrables : poutre élancée avec chargement distribué ; trois niveaux de raffinement au minimum ; observable définie ; comparaison Euler–Bernoulli selon le protocole du PRD ; script de reproduction et tableau précision/ressources ; limites des TET4 en flexion.

**Sortie :** convergence documentée et cible de déplacement satisfaite selon le protocole. Si l’écart de modèle rend la référence de poutre inadaptée, utiliser une référence solide 3D convergée comme prévu au PRD. Un échec de convergence suspend la livraison et déclenche investigation.

**Dépendances :** S08. **Traçabilité :** validation poutre, benchmark et exemples.

## S10 — Comparaison indépendante et durcissement

**Objectif :** vérifier la chaîne sur une implémentation indépendante et ses modes d’échec.

Livrables : comparaison externe sur maillage et modèle équivalents, CalculiX proposé ; fichiers, version et instructions ; tests de modèle sous-contraint, invalidités et valeurs limites ; mesure temps/mémoire sur machine documentée ; corrections issues de la revue scientifique.

**Sortie :** comparaison externe et suite obligatoire passent ; toutes les différences de conventions sont expliquées ; aucun défaut bloquant connu. Si le solveur externe n’est pas disponible, la porte reste ouverte jusqu’à obtention de cette preuve.

**Dépendances :** S08, S09. **Traçabilité :** PRD section 11 et exigences non fonctionnelles.

## S11 — Livraison et qualification MVP 0.1

**Objectif :** publier une première version reproductible.

Livrables : README d’installation/utilisation ; guide du modèle et unités ; dossier de validation ; exemples ; limites connues ; matrice des plateformes testées ; inventaire des dépendances/licences ; paquet ou procédure de build vérifiée ; notes de version.

**Sortie G0 :** toutes les exigences obligatoires du PRD MVP et sa définition de terminé sont satisfaites. La release candidate est validée depuis une installation propre ; les résultats et la comparaison externe sont archivés.

**Dépendances :** S00–S10. **Traçabilité :** J4 et PRD section 12.

## Correspondance exhaustive des exigences MVP

| Exigence | Sprints principaux | Preuve finale |
| --- | --- | --- |
| MOD-01 | S01, S08 | Modèle versionné et rejets |
| MOD-02 | S01, S02, S04 | Groupes, identifiants et références |
| MSH-01 / MSH-02 | S02 | Corpus d’import et invalidités |
| MAT-01 | S01, S03, S04 | Paramètres et affectations |
| BC-01 | S04 | Patch test et prescriptions non nulles |
| LOAD-01 / LOAD-02 | S06 | Résultantes et équilibre |
| FEM-01 | S03–S05, S09 | Élément, patch test et convergence |
| SOL-01 / SOL-02 | S05, S10 | Résidu, singularité et non-convergence |
| RES-01 | S06, S07 | Champs et réactions |
| IO-01 | S07 | VTU inspecté, CSV et JSON |
| CLI-01 / API-01 | S08 | Reproduction CLI/API |
| Validation et livraison | S09–S11 | Suite complète et comparaison externe |

## Règle de report

Une défaillance numérique se corrige avant le sprint dépendant. Les travaux IO/documentation indépendants peuvent continuer. Les extensions hors MVP restent dans la roadmap produit ; elles ne remplacent aucune preuve requise pour G0.
