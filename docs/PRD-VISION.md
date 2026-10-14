# FerraSim — PRD de vision produit

**Cible : plateforme de simulation industrielle compétitive face à SIMULIA**\
Version : 1.0 — vision proposée\
Date : 14 octobre 2026\
Statut : document de référence pour le produit cible ; capacités non encore implémentées\
Premier incrément : [PRD du MVP FerraSim 0.1](PRD.md)

Plan d’exécution : [roadmap générale](ROADMAP.md), [sprints du MVP](ROADMAP-MVP.md), [sprints vers le produit cible](ROADMAP-PRODUIT.md).

## 1. Résumé exécutif

FerraSim vise une plateforme open source de simulation industrielle permettant de préparer, calculer, vérifier, comparer et automatiser des études mécaniques et multiphysiques. Son moteur principal, son modèle de données et son orchestrateur sont développés en Rust. Python constitue l’interface scientifique principale ; une application graphique complète et une CLI utilisent les mêmes contrats de modèle et de calcul.

Le produit cible couvre la mécanique structurelle linéaire et non linéaire, la dynamique implicite et explicite, le contact, la thermique, la fatigue, les composites, l’acoustique, la CFD, l’électromagnétisme, le multicorps, l’optimisation et les couplages entre ces domaines. La couverture peut être native ou intégrée à travers des solveurs externes qualifiés. La provenance et le niveau de validation de chaque capacité restent visibles.

Être compétitif signifie permettre à un utilisateur industriel de remplacer un parcours de simulation identifié avec une précision, une robustesse, une productivité, une performance et un support suffisants. Une liste de fonctionnalités, un langage moderne ou un benchmark favorable isolé ne prouvent pas cette compétitivité.

La stratégie consiste à obtenir d’abord une profondeur crédible en mécanique structurelle, puis à élargir les parcours avec une plateforme commune. Le MVP 0.1 reste le premier jalon de vérification du moteur. Le présent document définit la destination, les critères de qualification et l’organisation des incréments nécessaires pour l’atteindre.

## 2. Référence concurrentielle et ambition

### 2.1 Périmètre de comparaison

SIMULIA regroupe plusieurs familles de produits. Le portefeuille présenté par Dassault Systèmes comprend notamment Abaqus, fe-safe, Tosca, Isight, CST Studio Suite, PowerFLOW et Simpack. Il couvre donc davantage qu’un solveur FEM structurel. Sources : [portefeuille et licensing SIMULIA](https://discover.3ds.com/simulia-unified-licensing), [solutions académiques SIMULIA](https://www.3ds.com/edu/education/academics/solutions/simulia).

Abaqus associe des produits de résolution, de préparation des modèles, de visualisation et de multiphysique. Cette organisation sert de référence pour la profondeur structurelle et le parcours utilisateur, sans imposer son architecture à FerraSim. Source : [famille Abaqus](https://www.3ds.com/products/simulia/abaqus).

| Domaine de référence | Référence SIMULIA | Cible FerraSim |
| --- | --- | --- |
| Préparation et post-traitement structurel | Abaqus/CAE | Environnement graphique, scripting et modèle ouvert commun |
| Mécanique implicite et explicite | Abaqus | Solveurs structurels natifs qualifiés |
| Fatigue et durée de vie | fe-safe | Évaluation de fatigue reliée aux histoires de contraintes et matériaux |
| Optimisation | Tosca, Isight | Optimisation de modèles et orchestration d’études paramétriques |
| CFD | PowerFLOW et offre CFD SIMULIA | Parcours CFD unifié, intégré puis natif selon décision de programme |
| Électromagnétisme | CST Studio Suite | Parcours EM basse et haute fréquence qualifiés par domaine |
| Dynamique multicorps | Simpack | Mécanismes rigides et flexibles, couplage avec modèles structurels |
| Multiphysique | Abaqus et intégrations du portefeuille | Mapping séquentiel, couplage itératif, monolithique et co-simulation |

Cette table constitue un cadrage des familles de besoins, pas une affirmation de parité actuelle. Les fonctionnalités détaillées des produits concurrents doivent être vérifiées à la version utilisée pour chaque campagne comparative.

### 2.2 Positionnement cible

FerraSim doit être choisi pour la combinaison suivante : résultats crédibles, automatisation complète, formats accessibles, possibilité de déploiement local ou souverain, extensibilité et coûts d’exploitation mesurables.

Les différenciateurs visés sont :

- un modèle inspectable et versionnable, avec conversions d’unités explicites ;
- la reproduction d’une étude depuis GUI, CLI ou Python ;
- une traçabilité complète du modèle jusqu’au rapport de décision ;
- un accès programmatique aux résultats et aux extensions ;
- des validations publiées et des limites affichées par capacité ;
- la maîtrise des données, des dépendances et de l’environnement d’exécution.

Ces caractéristiques sont des objectifs produit. Leur valeur sera évaluée avec des utilisateurs et des campagnes de mesure.

### 2.3 Trois niveaux de compétitivité

| Niveau | Signification | Preuve exigée |
| --- | --- | --- |
| C1 — Remplacement ciblé | Un parcours structurel défini peut être réalisé dans FerraSim | Benchmarks scientifiques et pilotes industriels sur ce parcours |
| C2 — Alternative structurelle industrielle | Une famille cohérente d’études linéaires, non linéaires, dynamiques et de contact est couverte | Qualification des solveurs, préparation complète, reprise, performance et support |
| C3 — Plateforme multiphysique compétitive | Plusieurs domaines et couplages partagent un environnement exploitable industriellement | Qualification par domaine et par couplage, interopérabilité, HPC et gouvernance des études |

La cible finale de ce PRD est C3. Chaque revendication commerciale devra nommer son niveau, les parcours couverts, les versions et les preuves associées.

## 3. Utilisateurs, acheteurs et secteurs

### 3.1 Personas

| Persona | Travail à accomplir | Critère de satisfaction |
| --- | --- | --- |
| Analyste de simulation | Construire et résoudre une étude fiable | Contrôle du modèle, diagnostics et résultats vérifiables |
| Ingénieur conception | Comparer des variantes sans être expert du solveur | Parcours guidé et méthodes approuvées |
| Expert méthodes | Définir formulations, matériaux et procédures réutilisables | API, extensions et qualification des modèles |
| Chercheur | Développer et comparer une méthode | Accès aux données, reproductibilité et bibliothèques extensibles |
| Responsable simulation | Organiser campagnes, validation et approbations | Traçabilité, gestion des versions et rapports |
| Administrateur HPC | Exécuter et surveiller les travaux | Déploiement, quotas, ordonnanceurs et diagnostics de ressources |
| Responsable informatique | Protéger les données et maintenir le logiciel | Installation maîtrisée, sécurité et politique de support |
| Enseignant et étudiant | Apprendre avec des modèles inspectables | Documentation pédagogique et installation simple |

### 3.2 Priorité des marchés

Le premier marché cible est constitué des bureaux d’études, PME industrielles et équipes de recherche réalisant des analyses mécaniques de composants et d’assemblages. Les premiers pilotes portent sur des pièces métalliques, assemblages boulonnés, structures minces et chargements thermomécaniques.

L’extension vise ensuite les machines, les transports, l’énergie, l’électronique et les composites. Les workflows de crash complet, turbomachines complexes, systèmes RF avancés et études réglementées nécessitent des programmes de qualification distincts. La couverture générale de la plateforme ne suffit pas à autoriser ces usages.

Les contraintes et priorités sectorielles seront validées par des entretiens et des pilotes ; ce PRD ne prétend pas disposer déjà de données de marché.

## 4. Objectifs produit et limites

### 4.1 Objectifs du produit cible

1. Préparer une étude depuis une CAO ou un maillage, jusqu’au rapport d’ingénierie.
2. Résoudre des analyses structurelles avancées avec diagnostics de convergence et bilans physiques.
3. Exécuter des analyses thermiques, fluides, EM, acoustiques et multicorps dans un environnement commun.
4. Relier ces analyses par des couplages qualifiés.
5. Automatiser campagnes, optimisation, calibration et post-traitement.
6. Exécuter sur poste de travail, cluster privé ou infrastructure cloud choisie.
7. Permettre inspection, migration, archivage et reproduction des études.
8. Assurer une maintenance industrielle des versions qualifiées.

### 4.2 Limites permanentes de la promesse produit

- Les résultats restent conditionnés par les hypothèses, matériaux, données et maillages du modèle.
- L’équivalence avec un logiciel externe est définie par parcours et par format ; elle n’est jamais présumée totale.
- La compatibilité binaire avec ses fichiers propriétaires ou ses sous-routines ne constitue pas un engagement global.
- La présence d’une fonction ne vaut pas validation pour tous ses régimes physiques.
- Un support à la certification signifie production de preuves et traçabilité ; il ne constitue pas une certification universelle du logiciel ou du produit étudié.
- Les méthodes expérimentales restent identifiées et séparées des capacités qualifiées.

## 5. Parcours utilisateur de bout en bout

### 5.1 Étude mécanique d’un assemblage

Importer un assemblage STEP, réparer les défauts identifiés, simplifier les détails inutiles, attribuer matériaux et sections, définir connexions et contacts, générer et vérifier le maillage, appliquer précharges et chargements, résoudre, inspecter contraintes et déplacements, vérifier convergence et équilibre, puis publier un rapport lié à la révision du modèle.

Acceptation : le parcours est réalisable dans la GUI et reproductible via un script exporté ; une modification de CAO indique les sélections et affectations à revalider ; le calcul peut être interrompu puis repris pour les procédures qui supportent la reprise.

### 5.2 Impact ou événement rapide

Définir vitesses initiales, contacts et matériaux dépendant du taux de déformation ; lancer une analyse explicite ; surveiller pas stable, contact et bilans énergétiques ; comparer déformation et courbes temporelles à une référence ; documenter toute mise à l’échelle de masse.

Acceptation : les énergies artificielles, masse ajoutée et paramètres de stabilisation sont quantifiés ; les critères propres au cas sont évalués automatiquement.

### 5.3 Thermomécanique

Résoudre un transfert thermique stationnaire ou transitoire, transférer les températures vers la structure, puis calculer déformations, contraintes et éventuellement contact. Pour le couplage bidirectionnel, échanger les états jusqu’à satisfaire les critères de convergence déclarés.

Acceptation : le mapping conserve les quantités pertinentes, les unités sont vérifiées et les bilans de chaque physique et de l’interface sont rapportés.

### 5.4 Écoulement et interaction fluide–structure

Préparer le domaine fluide, définir propriétés et conditions aux limites, mailler les couches limites, résoudre l’écoulement, transférer efforts et déformations avec la structure, gérer le mouvement du maillage et comparer forces, déplacement et stabilité du couplage.

Acceptation : le contrat précise régime, méthode de couplage, intervalle temporel, interpolation et critères de conservation. L’échec d’un participant ou du couplage produit un état d’échec global explicite.

### 5.5 Durée de vie et optimisation

Importer ou calculer des histoires de chargement, évaluer une durée de vie avec une méthode et un matériau tracés, puis explorer dimensions, forme ou topologie sous contraintes de fabrication et de performance.

Acceptation : chaque candidat conserve modèle, solveur et critères ; l’optimum est recalculé sur un modèle de vérification indépendant de la représentation optimisée.

### 5.6 Campagne industrielle

Un expert publie un gabarit de méthode approuvé. Un utilisateur renseigne les paramètres autorisés, soumet une campagne, suit ses travaux et soumet ses résultats à revue. Le responsable compare variantes et approuve une révision identifiée.

Acceptation : droits, historique, provenance et statut d’approbation sont conservés ; modifier une entrée crée une nouvelle révision et invalide les approbations dépendantes.

## 6. Convention des exigences et maturité

Chaque exigence ci-dessous appartient à la vision finale. Sa réalisation est affectée à une vague de livraison ; elle ne modifie pas le périmètre 0.1.

Chaque capacité publiée possède un niveau : **expérimentale**, **vérifiée numériquement**, **validée pour un domaine d’emploi**, ou **qualifiée pour un parcours industriel**. La fiche associée décrit hypothèses, formulations, éléments compatibles, plage de paramètres, restrictions et preuves.

Une intégration externe peut être qualifiée au même niveau qu’une capacité native, mais sa version, ses dépendances, son installation et ses conditions d’utilisation restent explicites. Le lancement automatique d’un solveur ne suffit pas à qualifier l’intégration.

## 7. Géométrie, assemblages et préparation

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| GEO-01 | Import STEP avec unités, instances et structure d’assemblage | Corpus de pièces et assemblages importé ; transformations et volumes contrôlés |
| GEO-02 | Import de géométries facettisées STL/OBJ et formats supplémentaires sélectionnés | Échelles, orientation, fermeture et anomalies affichées |
| GEO-03 | Réparation : coutures, petits défauts, faces et arêtes invalides | Modification prévisualisée ; journal des réparations ; géométrie d’origine conservée |
| GEO-04 | Simplification : suppression de détails, surfaces moyennes et idéalisations | Modifications paramétrées et rejouables ; impact sur sélections signalé |
| GEO-05 | Pièces, instances, repères, symétries et opérations de partition | Édition des instances sans corruption de la pièce source |
| GEO-06 | Réassociation après mise à jour CAO | Correspondances assorties d’un niveau de confiance ; ambiguïtés à revalider |
| GEO-07 | Connecteurs CAO additionnels | Qualification et licence par connecteur ; dégradation documentée |

Le noyau B-Rep est intégré après évaluation plutôt que réécrit comme préalable. IGES et formats natifs sont des connecteurs distincts. Une sélection attachée à une face disparue ne doit jamais être réaffectée silencieusement.

## 8. Maillage et discrétisation

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| MSH-01 | Maillages 1D, 2D et 3D, linéaires et quadratiques | Connectivité, orientation, ordre et correspondance géométrique validés |
| MSH-02 | Tétraèdres, hexaèdres, prismes, pyramides, triangles et quadrilatères | Formulations compatibles affichées ; transitions vérifiées |
| MSH-03 | Maillage libre, structuré et par extrusion | Contrôles de taille, courbure et raffinement rejouables |
| MSH-04 | Couches limites pour écoulements | Contrôle épaisseur, croissance et qualité ; évaluation liée au régime visé |
| MSH-05 | Qualité : Jacobien, distorsion, aspect et éléments inversés | Seuils associés à la formulation ; blocage des invalidités rédhibitoires |
| MSH-06 | Raffinement local et adaptation h/p selon procédures compatibles | Estimateur documenté ; étude de convergence et transfert d’état |
| MSH-07 | Régions, surfaces, groupes et partitions | Conservation ou remapping explicite des sélections |
| MSH-08 | Maillages non conformes et interfaces | Méthode de liaison et erreurs d’interface mesurées |
| MSH-09 | Sous-modélisation et transfert global–local | Repères, unités, interpolation et frontières tracés |
| MSH-10 | Import/export de maillages | Corpus d’aller-retour avec pertes connues et rapport d’import |

Gmsh, VTU et formats ouverts structurent le premier socle d’échange. Les formats spécialisés CFD, CGNS ou autres, sont retenus selon les solveurs qualifiés. La présence d’une topologie dans l’importeur ne signifie pas qu’une formulation physique existe pour cette topologie.

## 9. Mécanique structurelle

### 9.1 Analyses

| ID | Exigence cible | Acceptation scientifique |
| --- | --- | --- |
| STR-01 | Statique linéaire 2D/3D, axisymétrie, contraintes et déformations planes | Patch tests, solutions analytiques et convergence |
| STR-02 | Grandes transformations et statique non linéaire | Objectivité, cohérence tangentielle et chemins de charge de référence |
| STR-03 | Flambement linéaire et post-flambement avec imperfections | Modes, charges critiques et trajectoires comparés |
| STR-04 | Analyse modale, précontrainte et amortissement | Fréquences, modes, orthogonalité et sensibilité au maillage |
| STR-05 | Réponse harmonique et spectrale | Fonctions de réponse, amortissement et conventions complexes vérifiés |
| STR-06 | Dynamique implicite | Ordre temporel, conservation/dissipation et stabilité mesurés |
| STR-07 | Dynamique explicite | Pas critique, bilans, contacts et impact de référence |
| STR-08 | Transfert implicite–explicite et reprise d’état | Équilibre, variables internes et énergie contrôlés au transfert |
| STR-09 | Sous-structures et réduction de modèles | Erreur de réduction et plage fréquentielle documentées |
| STR-10 | Fracture, endommagement et propagation | Objectivité au maillage ou régularisation documentée ; énergie dissipée vérifiée |

### 9.2 Bibliothèque d’éléments

La bibliothèque cible inclut solides linéaires et quadratiques, poutres, barres, coques, membranes, éléments axisymétriques, ressorts, amortisseurs, connecteurs et interfaces cohésives. Les éléments mixtes ou hybrides traitent les régimes nécessitant un contrôle de l’incompressibilité.

Chaque formulation publie degrés de liberté, interpolation, quadrature, cinématique, limitations et compatibilités. Les formulations à intégration réduite publient leurs contrôles de modes parasites et leurs contributions énergétiques. Les coques sont qualifiées sur membrane, flexion, cisaillement, orientation et épaisseur.

### 9.3 Charges et interactions

Forces, moments, pressions, tractions, gravité, accélérations, forces centrifuges, champs thermiques et précharges de boulons sont exprimés dans des repères explicites. Les charges peuvent varier dans l’espace et le temps. Le caractère suiveur d’une charge est déclaré.

Les liaisons incluent déplacements prescrits, relations multipoints, corps rigides, couplages distribués, assemblages collés et connecteurs mécaniques. Les redondances et surcontraintes sont diagnostiquées. Pour chaque procédure, les réactions et bilans sont disponibles.

## 10. Matériaux, composites et identification

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| MAT-01 | Élasticité isotrope, orthotrope et anisotrope | Symétries, stabilité, repères et unités vérifiés |
| MAT-02 | Plasticité métallique avec écrouissages isotrope et cinématique | Essais monotones/cycliques, retour plastique et tangent cohérent |
| MAT-03 | Hyperélasticité et quasi-incompressibilité | Essais multiaxiaux, stabilité et formulation compatible |
| MAT-04 | Viscoélasticité, fluage et dépendances température/vitesse | Réponses temporelles et histoire d’état vérifiées |
| MAT-05 | Endommagement, rupture et cohésion | Paramètres, longueur caractéristique et dissipation tracés |
| MAT-06 | Composites stratifiés, orientations et critères de rupture | Empilement, contraintes par pli et cas de référence |
| MAT-07 | Matériaux spécialisés : mousses, sols, béton et milieux poreux | Qualification distincte par modèle et régime, sans label générique de validité |
| MAT-08 | Calibration sur données d’essai | Provenance, unités, incertitudes et qualité d’ajustement conservées |
| MAT-09 | Matériaux utilisateur | Tests constitutifs, contrat d’état, unités et dérivées définis |
| MAT-10 | Bibliothèque versionnée | Source, plage d’emploi, statut d’approbation et restrictions d’utilisation visibles |

Le contrat constitutif couvre initialisation, variables internes, mise à jour d’essai, validation ou annulation de l’incrément, contraintes, tangent et contributions énergétiques. Un calcul non convergé ne doit pas engager définitivement l’état du matériau.

Les bibliothèques ne confondent pas nom commercial d’un matériau et loi validée. Les données peuvent être privées et accessibles selon des droits propres. L’identification fournit plusieurs jeux plausibles si les paramètres ne sont pas identifiables de manière unique.

## 11. Contact et assemblages

Le contact constitue un programme produit spécifique, nécessaire au niveau C2.

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| CNT-01 | Contact normal, frottement et grands glissements | Pression, pénétration, forces et dissipation vérifiées |
| CNT-02 | Auto-contact et contact général | Cas multi-surfaces ; déterminisme des règles et diagnostics |
| CNT-03 | Méthodes pénalité et méthodes contraintes sélectionnées | Sensibilité aux paramètres documentée ; comportement des solveurs vérifié |
| CNT-04 | Contact thermique et couplage friction–chaleur | Bilan énergétique d’interface et conservation |
| CNT-05 | Interférences initiales et précharges | Initialisation contrôlée, état équilibré et séquence rejouable |
| CNT-06 | Visualisation et diagnostic | Zones actives, glissement, pénétration et évolution temporelle accessibles |

L’utilisateur contrôle les tolérances et peut examiner les choix automatiques. Les réglages de convergence ne doivent pas masquer une erreur géométrique ou une pénétration physiquement inacceptable.

## 12. Thermique, acoustique et autres champs

### 12.1 Thermique

Conduction stationnaire et transitoire, propriétés anisotropes et dépendantes de la température, convection imposée, rayonnement, sources volumiques et surfaciques, résistance thermique de contact et changement de phase dans un domaine défini.

Acceptation : solutions analytiques, conservation d’énergie, convergence spatiale/temporelle et contrôles de non-linéarité. Les températures absolues et écarts de température sont distingués dans les unités.

### 12.2 Acoustique et vibro-acoustique

Acoustique intérieure et extérieure, fréquentielle et transitoire selon les solveurs, conditions absorbantes, excitations structurelles, interaction acoustique–structure et indicateurs de bruit.

Acceptation : cavités, ondes et rayonnement de référence ; sensibilité aux frontières artificielles ; pression, niveaux et références logarithmiques explicitement définis.

### 12.3 Champs complémentaires

Électricité conductive, effet Joule, piézoélectricité, diffusion et poromécanique complètent les parcours multiphysiques. L’électrochimie et les batteries constituent un module sectoriel ultérieur avec transport, réactions et données spécifiques. Chaque champ reçoit ses propres bilans, conventions et domaines d’emploi.

## 13. CFD

### 13.1 Périmètre cible

Le produit cible couvre écoulements incompressibles puis compressibles, régimes laminaires et turbulents, stationnaire et transitoire, transfert de chaleur conjugué, domaines mobiles et machines tournantes. Multiphase, surfaces libres, particules et combustion forment des extensions spécialisées avec qualification propre.

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| CFD-01 | Préparation, régions fluides et conditions aux limites | Cohérence des frontières, propriétés et bilans d’entrée contrôlée |
| CFD-02 | Écoulement laminaire | Poiseuille, Couette et cas manufacturés avec convergence |
| CFD-03 | Turbulence | Modèle, traitement de paroi et domaine de validité affichés ; comparaison à données de référence |
| CFD-04 | Compressibilité | Bilans et résolution de discontinuités sur cas dédiés |
| CFD-05 | Transfert thermique conjugué | Températures et flux continus selon le contrat d’interface |
| CFD-06 | Maillage mobile et référentiels tournants | Conservation géométrique, intégrité du maillage et efforts vérifiés |
| CFD-07 | Post-traitement et convergence | Résidus, débits, forces, coefficients et statistiques temporelles accessibles |
| CFD-08 | Extensions multiphases | Conservation des phases et validation expérimentale par régime |

### 13.2 Stratégie de réalisation

La première couverture peut reposer sur un solveur externe, par exemple OpenFOAM, après qualification du connecteur. Le choix d’un solveur natif et de ses méthodes — volumes finis ou autre — fait l’objet d’une décision distincte fondée sur les usages prioritaires et les moyens disponibles.

Le support CFD ne doit pas dépendre d’hypothèses propres aux seuls éléments FEM structurels. Le modèle commun décrit champs, régions, frontières et provenance ; chaque méthode conserve les informations nécessaires à sa discrétisation.

## 14. Électromagnétisme

La couverture se décompose en basse fréquence et haute fréquence ; ces programmes ne partagent pas nécessairement le même solveur.

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| EM-01 | Électrostatique et conduction | Potentiels, charges, courants et bilans de référence |
| EM-02 | Magnétostatique et courants de Foucault | Champs, flux, pertes et non-linéarités matériaux vérifiés |
| EM-03 | Machines électriques et mouvement | Couple, pertes, rotation et couplage thermique contrôlés |
| EM-04 | Propagation haute fréquence et ports | Résonances, paramètres S, maillage et frontières absorbantes qualifiés |
| EM-05 | Antennes et compatibilité électromagnétique | Diagrammes, puissances, unités et protocole expérimental définis |
| EM-06 | Couplages EM–thermique–structure | Pertes, efforts et transfert de champs avec bilans |

Les méthodes natives, éléments adaptés aux champs, conditions de jauge et solveurs temporels/fréquentiels sont choisis dans les PRD de module. Une intégration tierce peut porter la première couverture. Une capacité électrostatique seule ne permet pas de présenter FerraSim comme une alternative complète à un environnement RF.

## 15. Multicorps, systèmes et commandes

La cible inclut corps rigides, articulations, contraintes cinématiques, ressorts et amortisseurs, entraînements, corps flexibles réduits, contacts spécialisés et échange avec modèles de commande.

Acceptation : mécanismes de référence, fermeture des contraintes, équilibre énergétique, dérive cinématique et validation des interfaces de corps flexibles. Les interfaces de co-simulation précisent causalité, pas de communication, interpolation et gestion des discontinuités.

Un connecteur FMI est envisagé pour les modèles de systèmes compatibles ; la version et les fonctionnalités réellement prises en charge seront déclarées. Il ne remplace pas la qualification mécanique des couplages.

## 16. Multiphysique et orchestration

Abaqus distingue notamment mapping séquentiel, résolution complètement couplée et co-simulation. Ces familles structurent aussi les besoins FerraSim. Source : [Abaqus Multiphysics](https://www.3ds.com/products/simulia/abaqus/multiphysics).

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| CPL-01 | Enchaînement séquentiel de calculs | Dépendances, états et versions tracés ; invalidation des résultats aval |
| CPL-02 | Mapping entre maillages | Repères, unités, champs intensifs/extensifs et erreur de transfert définis |
| CPL-03 | Couplage bidirectionnel partitionné | Résidus d’interface, relaxation et convergence accessibles |
| CPL-04 | Couplage monolithique pour procédures sélectionnées | Jacobien couplé et bilans validés |
| CPL-05 | Synchronisation temporelle et sous-cyclage | Instants d’échange et erreurs temporelles documentés |
| CPL-06 | Arrêt et reprise d’un système couplé | Checkpoint cohérent de tous les participants ; aucune reprise partielle silencieuse |
| CPL-07 | Co-simulation externe | Protocole versionné, timeout, annulation et erreurs structurées |

Le graphe d’exécution définit physiques, champs échangés, dépendances, unités, pas de temps et stratégie de convergence. Le transfert de forces, flux ou énergie applique une méthode conservatrice lorsqu’elle est nécessaire au contrat. Un simple interpolateur nodal ne suffit pas à toutes les interfaces.

Les premiers couplages qualifiés sont thermique–structure séquentiel, thermique–électrique et fluide–structure sur un domaine limité. La stabilité en présence d’effets de masse ajoutée, le mouvement du maillage et les transferts entre discrétisations différentes sont évalués avant extension.

## 17. Fatigue, optimisation et incertitudes

### 17.1 Fatigue et durée de vie

Histoires de contraintes et déformations, comptage de cycles, courbes S–N et ε–N, corrections de contrainte moyenne, méthodes multiaxiales et traitements propres aux assemblages ou composites selon qualification.

Acceptation : méthodes nommées et versionnées, données matériau tracées, cumul d’endommagement documenté, courbes expérimentales et sensibilité aux hypothèses. Les valeurs de durée de vie doivent être accompagnées des limites et facteurs influents.

### 17.2 Optimisation et exploration

Plans d’expériences, études paramétriques, optimisation sous contraintes, optimisation de forme et de topologie, calibration, sensibilités et gradients lorsque disponibles. Contraintes de fabrication et reconstruction géométrique sont intégrées au parcours de topologie.

Acceptation : gestion des calculs échoués, critères d’arrêt, historique des candidats, contrôle des dérivées et recalcul final. Toute approximation ou modèle de substitution conserve son domaine d’apprentissage et son erreur de validation.

### 17.3 Incertitudes

Variables aléatoires, distributions et corrélations, propagation d’incertitudes, analyses de sensibilité et études de robustesse. Les incertitudes de paramètres, de discrétisation et de modèle sont distinguées. Un intervalle statistique ne doit pas être présenté comme une borne garantie si le protocole ne l’établit pas.

## 18. Solveurs et qualité numérique

La bibliothèque cible comporte solveurs directs creux et itératifs adaptés aux classes de systèmes : symétriques définis positifs, indéfinis, non symétriques et complexes. Préconditionnement, mises à l’échelle et diagnostics de conditionnement sont exposés de façon utile.

Les procédures non linéaires incluent Newton, recherche linéaire, réduction automatique d’incrément et suivi de chemin lorsque nécessaire. Les solveurs temporels publient paramètres, stabilité, amortissement numérique et stratégies adaptatives.

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| NUM-01 | Diagnostic de convergence | Résidus normalisés avec échelles définies ; cas de charge nulle traités |
| NUM-02 | Analyse des singularités et contraintes | Messages localisant les causes lorsque possible ; limites du diagnostic déclarées |
| NUM-03 | Contrôle des erreurs et incréments | Historique complet, essais annulés et raisons des réductions |
| NUM-04 | Précision et reproductibilité | Référence f64 ; autres précisions qualifiées par procédure |
| NUM-05 | Checkpoint et reprise | Résultat repris équivalent au calcul continu dans la tolérance de référence |
| NUM-06 | Bilans physiques | Forces, moments, masse, énergie et charge électrique selon physique |
| NUM-07 | Repli de stratégie | Changement de méthode visible et tracé ; aucun changement de physique silencieux |

Le choix automatique d’un solveur s’appuie sur les propriétés du problème. Un statut « convergé » indique la satisfaction des critères numériques déclarés, pas la validation physique globale de l’étude.

## 19. Résultats, visualisation et rapports

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| PST-01 | Maillage, déformée, champs scalaires/vectoriels/tensoriels | Repères, unités, position d’évaluation et facteur de déformation visibles |
| PST-02 | Coupes, iso-surfaces, chemins, sondes et animations | Opérations rejouables et exportables |
| PST-03 | Courbes temporelles/fréquentielles et comparaison | Synchronisation et interpolation explicites |
| PST-04 | Valeurs aux points d’intégration, éléments et nœuds | Extrapolation et lissage identifiés ; accès aux champs non lissés |
| PST-05 | Critères personnalisés | Formules versionnées avec contrôle dimensionnel |
| PST-06 | Rapport d’ingénierie | Hypothèses, révisions, convergence, bilans, images et critères regroupés |
| PST-07 | Gros résultats | Lecture partielle, niveaux de détail et absence de chargement global obligatoire |
| PST-08 | Échanges ouverts | VTU/VTK, CSV et stockage scientifique documenté ; compatibilité vérifiée |

Les résultats comprennent les grandeurs pertinentes par physique : déplacements, vitesses, accélérations, contraintes, déformations, réactions, énergies, température, flux, pression, vitesse fluide, champs EM, pertes et grandeurs multicorps.

Les singularités et maxima dépendant du maillage sont signalés dans les méthodes d’évaluation. Le rapport distingue résultats bruts, post-traitements et conclusions de l’utilisateur. Un rendu graphique ne transforme pas automatiquement un résultat en critère de conformité.

## 20. Interface graphique et expérience produit

L’application cible comprend arbre de projet, vue 3D, inspecteur de propriétés, ateliers de géométrie/maillage/physique/résultats, gestionnaire de travaux, console et rapports. Les opérations longues restent annulables lorsque techniquement possible et n’immobilisent pas l’édition indépendante.

| ID | Exigence cible | Acceptation |
| --- | --- | --- |
| UX-01 | Parcours guidé et contrôle expert | Aides liées aux hypothèses ; réglages détaillés accessibles |
| UX-02 | Sélection et édition fiables | Sélections prévisualisées ; entités masquées et repères identifiés |
| UX-03 | Annuler/rétablir et autosauvegarde | Édition réversible ; récupération testée après arrêt anormal |
| UX-04 | Commandes rejouables | Opérations de modélisation persistantes dans un journal exportable |
| UX-05 | Diagnostics actionnables | Message, entités concernées, cause probable et action suggérée |
| UX-06 | Accessibilité et langues | Navigation clavier, contraste et interface français/anglais ; protocoles utilisateurs |
| UX-07 | Méthodes réutilisables | Gabarits avec paramètres autorisés, versions et domaine d’emploi |

Le même calcul depuis GUI, CLI et Python génère le même modèle normalisé et les mêmes paramètres effectifs. Les différences d’environnement sont enregistrées et évaluées, sans exigence d’identité bit à bit entre toutes les architectures.

## 21. API, scripting et extensions

### 21.1 Interfaces

API Rust pour intégration au moteur ; API Python pour préparation, calcul et analyse ; CLI pour automatisation et HPC ; API distante versionnée pour gestion des projets et travaux. L’API distante doit soumettre des travaux asynchrones et ne suppose pas qu’un calcul industriel tient dans une requête HTTP.

Chaque interface expose validation, lancement, suivi, annulation, statut final et lecture des résultats. Les versions stables suivent une politique de compatibilité publiée ; les dépréciations proposent un chemin de migration.

### 21.2 Extensions

Matériaux, éléments, conditions aux limites, post-traitements, importeurs et adaptateurs de solveurs peuvent être étendus. Le SDK fournit contrats, exemples, tests et métadonnées de compatibilité.

La stratégie d’extension privilégie des protocoles versionnés et des processus isolés pour les composants externes. Une ABI Rust native stable n’est pas présumée. Les extensions en processus doivent déclarer précisément leur compatibilité et leurs risques.

Les extensions disposent d’un contrat sur les ressources et permissions. Un projet partagé n’exécute pas automatiquement du code provenant d’un plugin ou script absent de la configuration autorisée.

### 21.3 Assistances automatisées

Une assistance à la préparation, au diagnostic ou au scripting est envisageable. Toute proposition affectant le modèle reste inspectable et rejouable. La production d’un modèle par une assistance n’altère pas les obligations de validation et ne constitue pas un argument de précision.

## 22. Modèle commun, formats et interopérabilité

Le projet cible conserve : géométrie, instances, repères, maillages, régions, matériaux, sections, interactions, charges, prescriptions, procédures, graphe de couplage, paramètres numériques, demandes de sortie et provenance.

Les unités sont des métadonnées contrôlées ; les grandeurs incompatibles sont rejetées et toute conversion est explicite. Les identifiants sont stables au sein d’une révision. Les états de calcul appartiennent à une révision immuable.

Le conteneur de projet associe un manifeste versionné, des métadonnées inspectables et des données binaires pour les volumes importants. Le format de stockage des champs est choisi après évaluation d’HDF5 et d’alternatives ; chunks, compression, lecture partielle et écritures concurrentes font partie des critères.

Les imports d’entrées Abaqus, CalculiX, Code_Aster ou Nastran sont qualifiés par sous-ensemble. Chaque import produit un rapport des entités reconnues, conversions, pertes et commandes inconnues. Une entité physique inconnue ne peut pas être ignorée sans décision explicite de l’utilisateur.

Le format propriétaire de résultats Abaqus n’est pas un format natif cible. Une interopérabilité peut passer par des exports ou outils autorisés dans l’environnement du client, sans promesse de compatibilité totale.

## 23. Calcul parallèle, HPC et déploiement

### 23.1 Calcul

CPU multithread, vectorisation, mémoire distribuée et accélération GPU sont évalués sur les procédures qui en bénéficient. Le produit cible propose partitionnement, assemblage et résolution distribués, checkpoints et estimation de ressources.

CUDA, Metal ou autres backends constituent des choix d’implémentation à qualifier. Un backend présent sur une plateforme ne garantit pas la disponibilité des mêmes accélérations sur toutes les autres.

### 23.2 Exécution

**Windows, macOS et Linux sont des plateformes obligatoires du produit**, dès le MVP et dans les incréments suivants. Le socle local — modèle, solveurs CPU natifs, CLI, puis GUI et API Python lorsqu’elles sont livrées — fournit les mêmes capacités fonctionnelles sur les trois systèmes. WSL, Docker ou un serveur distant ne sont pas requis pour ce socle.

Chaque release fournit une matrice de versions/architectures supportées, des instructions d’installation et des builds/tests réussis pour les trois systèmes. Les vérifications scientifiques utilisent les mêmes références et tolérances ; l’identité bit à bit entre architectures n’est pas exigée. Une régression bloquante du socle sur une plateforme bloque la release.

Les dépendances de géométrie, maillage, rendu et packaging Python sont évaluées pour les trois systèmes avant adoption. Un connecteur ou backend spécialisé peut avoir une disponibilité distincte, mais cette limite est affichée avant usage et ne réduit pas la portabilité du socle. Une capacité dépendant exclusivement d’un outil externe disponible sur un seul système ne peut pas être annoncée comme portable sans solution qualifiée sur les autres.

Linux reste la première cible du calcul HPC distribué ; conteneurs et connecteurs d’ordonnanceurs, avec Slurm comme première intégration proposée. Les clients Windows et macOS doivent pouvoir préparer, soumettre et récupérer ces travaux. Les backends GPU/MPI, connecteurs et extensions disposent de leur propre matrice de support ; leurs dépendances optionnelles ne doivent pas empêcher compilation et usage du socle CPU.

Travaux locaux, clusters privés et services cloud choisis par le client partagent un cycle : validation, mise en file, allocation, exécution, sauvegarde, récupération et archivage. Annulation, retry et reprise ne doivent pas dupliquer silencieusement les résultats ou la facturation.

### 23.3 Cibles proposées de dimensionnement

| Usage | Classe de problème de référence | Objectif de qualification |
| --- | --- | --- |
| Poste de travail | 10⁵ à 10⁶ degrés de liberté structurels | Mesure temps/mémoire sur machine documentée et algorithme adapté |
| Cluster structurel | 10⁷ degrés de liberté et au-delà | Scaling, robustesse et reprise sur plusieurs configurations |
| CFD distribué | 10⁷ cellules et au-delà | Bilans, débit de calcul et communication mesurés |
| Campagnes | 1 000 travaux soumis, 100 actifs sur infrastructure définie | États cohérents, quotas et absence de perte de travail |
| Post-traitement | Jeux de résultats ≥ 100 Go | Lecture partielle et navigation sans chargement intégral |

Ces classes sont des objectifs de test, pas une promesse qu’un problème quelconque de cette taille sera résolu sur une machine quelconque. Le coût dépend notamment de la formulation, des contacts, du conditionnement et du solveur.

## 24. Collaboration, sécurité et gouvernance

- Travail local complet sans compte ni transfert cloud obligatoire pour les capacités installées.
- Espaces partagés avec rôles lecteur, analyste, approbateur et administrateur.
- Authentification d’entreprise pour services partagés, gestion des secrets et journal d’audit.
- Isolation des travaux et protection contre archives malformées, chemins sortant du projet et imports hostiles.
- Chiffrement des échanges distants ; politique de sauvegarde et de rétention configurable.
- Télémétrie explicitement activée, documentée et désactivable ; données de simulation exclues par défaut.
- Versions, dépendances et artefacts identifiables ; nomenclature des composants et procédure de vulnérabilités.
- Déploiement hors ligne, politique de mises à jour maîtrisée et fonctionnement sur réseau restreint.

Acceptation : tests de droits et d’isolation, restauration de sauvegarde, audit des plugins, revue de sécurité avant service multi-utilisateur. Les engagements légaux, régionaux et de conformité feront l’objet d’une vérification spécifique au déploiement concerné.

## 25. Architecture de référence

```text
GUI / CLI / Python / API distante
                |
       Services du modèle et commandes
                |
  Projet versionné + graphe des analyses
                |
      Orchestrateur et gestion des travaux
                |
  Physiques et procédures qualifiées
       |                    |
 Méthodes natives     Adaptateurs externes
       |                    |
 Algèbre / CPU / HPC     Processus / services
                |
    Stockage de résultats et provenance
                |
  Visualisation / rapports / comparaison
```

Invariants : aucune physique dans la GUI ; séparation données/états d’exécution ; couches scientifiques testables sans réseau ; échanges versionnés ; frontières entre composants observables ; résultats liés à des entrées immuables.

Le workspace Rust grandit par besoins réels. Les composants proposés à terme sont modèle, géométrie, maillage, matériaux, méthodes FEM/CFD/EM, algèbre, solveurs, couplage, orchestration, IO, résultats, SDK, scripting et interfaces. Ce découpage cible n’impose pas de créer tous les crates avant le MVP.

Les solveurs externes sont considérés comme des participants identifiés et non comme des bibliothèques interchangeables sans coût d’intégration. Licences, installation, unités, erreurs et archivage de leur environnement sont des contrats de produit.

## 26. Validation, qualification et preuves industrielles

### 26.1 Pyramide de preuves

1. Vérification des opérations mathématiques, lois constitutives et éléments.
2. Solutions analytiques ou manufacturées et études de convergence.
3. Comparaisons à des solveurs indépendants sur des modèles équivalents.
4. Comparaisons expérimentales avec incertitudes et conditions d’essai.
5. Pilotes industriels représentatifs et revues de méthode.
6. Régressions sur les versions, plateformes et backends qualifiés.

La comparaison entre deux solveurs constitue une preuve de cohérence, pas à elle seule une validation physique. Les tolérances sont définies avant examen des résultats, par observable et domaine, avec gestion des valeurs proches de zéro et des singularités.

### 26.2 Bibliothèque cible de qualification

| Domaine | Familles de cas obligatoires |
| --- | --- |
| Structure | Traction, flexion, torsion, plaques/coques, modes, flambement |
| Non-linéaire | Plasticité cyclique, hyperélasticité, grandes rotations, instabilités |
| Contact | Hertz, glissement, assemblage préchargé, auto-contact |
| Dynamique | Oscillateurs, propagation d’ondes, impact et bilans énergétiques |
| Thermique | Conduction, transitoire, rayonnement, interfaces et effet Joule |
| CFD | Écoulements analytiques, turbulence, compressibilité, convection |
| EM | Capacités, inductances, pertes, résonateurs et rayonnement |
| Multicorps | Mécanismes, contraintes et corps flexibles |
| Couplage | Thermomécanique, fluide–structure, acoustique–structure |
| Fatigue/optimisation | Histoires connues, références expérimentales, optima vérifiables |

Des suites reconnues, telles que des benchmarks NAFEMS, peuvent compléter cette bibliothèque après sélection et obtention des droits nécessaires. Aucun accès à ces données n’est présumé.

### 26.3 Dossier d’une capacité qualifiée

Formulation, domaine d’emploi, entrées, unités, solveurs compatibles, références, métriques, tolérances, résultats, limites, version, plateformes et responsables de revue. Les anomalies connues sont publiées avec leur effet sur les usages.

## 27. Mesure de la compétitivité et critères de sortie

Les seuils suivants sont des cibles proposées pour le programme. Ils deviennent des engagements seulement lorsqu’une campagne, une baseline et un environnement sont fixés. Ils ne décrivent pas des performances déjà obtenues.

| Axe | Cible proposée | Protocole |
| --- | --- | --- |
| Exactitude | 100 % des cas de qualification passent leur tolérance préétablie | Références indépendantes, convergence et contrôles physiques |
| Robustesse | ≥ 95 % de réussite sur un corpus éligible de modèles industriels valides | Configuration et budget de ressources fixés ; échecs conservés dans le bilan |
| Productivité | Temps médian de préparation au plus égal à la référence ; objectif −20 % sur parcours automatisés | Utilisateurs comparables, formation et tâches identiques |
| Performance | Temps médian total ≤ 1,25× la référence sur corpus prioritaire | Même précision cible, matériel équivalent et réglages documentés |
| Mémoire | Mémoire médiane ≤ 1,25× la référence sur ce corpus | Pics mesurés, méthode de résolution déclarée |
| Scaling | Efficacité forte cible ≥ 60 % entre 1 et 8 nœuds sur cas sélectionnés | Taille fixe, baseline et matériel identiques ; calcul E = T₁/(8T₈) |
| Fiabilité de reprise | Tous les scénarios de reprise qualifiés respectent la tolérance du calcul continu | Arrêts aux points de sauvegarde et scénarios de panne contrôlés |
| Traçabilité | 100 % des travaux qualifiés disposent d’un manifeste complet | Entrées, versions, paramètres effectifs et hashes contrôlés |
| Interopérabilité | Aucun objet non pris en charge perdu silencieusement | Corpus d’import et rapport de pertes |
| Expérience | ≥ 90 % des participants accomplissent les tâches prioritaires sans assistance bloquante | Protocole observé et erreurs consignées |

Les distributions et cas extrêmes sont publiés avec les médianes. Les cas non convergés ne sont pas exclus pour améliorer artificiellement la comparaison. Coût total, formation, maintenance et infrastructure sont évalués séparément du temps solveur.

### Qualification C2

Toutes les capacités du parcours structurel sélectionné sont qualifiées ; GUI/CLI/Python reproduisent les études ; la campagne comparative passe ; au moins trois organisations pilotes indépendantes accomplissent leurs parcours avec une revue documentée ; installation, documentation, reprise et support sont opérationnels.

### Qualification C3

C2 est acquis. Au moins un parcours qualifié est disponible dans chacun des domaines de la matrice concurrentielle : structure, fatigue, optimisation, CFD, EM basse et haute fréquence, multicorps et acoustique. Les couplages prioritaires sont qualifiés ; gestion des campagnes, HPC et gouvernance satisfont leurs protocoles. La largeur exacte de couverture est publiée, même lorsque certaines capacités reposent sur des intégrations.

Le statut C3 signifie compétitivité sur ce portefeuille de parcours. La parité exhaustive avec toutes les fonctions et options de SIMULIA ne peut pas être déduite de ce statut.

## 28. Roadmap vers le produit cible

| Vague | Valeur livrée | Capacités principales | Porte de sortie |
| --- | --- | --- | --- |
| V0 — Fondations | Calcul structurel minimal vérifié | Périmètre 0.1 du PRD MVP | Définition de terminé 0.1 |
| V1 — Parcours structurel complet | CAO/maillage/calcul/résultats sur composant | GUI initiale, Python, éléments enrichis, modal et thermique séquentielle | C1 sur cas identifiés |
| V2 — Assemblages non linéaires | Études structurelles industrielles | Contact, plasticité, grandes transformations, coques, précharges | Campagne non linéaire et pilotes |
| V3 — Dynamique et durabilité | Événements transitoires et durée de vie | Implicite/explicite, composites, fatigue, reprise, HPC structurel | C2 et support des parcours déclarés |
| V4 — Campagnes et optimisation | Exploration industrielle répétable | DOE, calibration, forme/topologie, gestion de travaux et collaboration | Qualification des méthodes et déploiement partagé |
| V5 — Extension des physiques | Environnement commun multi-domaines | CFD, EM, acoustique et multicorps via modules/intégrations | Qualification par domaine |
| V6 — Couplages et portefeuille | Parcours multiphysiques industriels | Couplages prioritaires, HPC multi-domaines, API et données stabilisées | C3 |

Les vagues portent des portes de qualification, pas des dates promises. Certains modules peuvent avancer en parallèle après stabilisation des contrats communs et allocation d’équipes ; leurs dépendances scientifiques demeurent obligatoires. La collaboration et la sécurité sont introduites dès les premiers services partagés, pas reportées à la fin.

Chaque vague reçoit un PRD de livraison avec exigences retenues, exclusions, ressources, tests, critères de sortie et plan de migration. Les analyses de risque et preuves peuvent conduire à revoir l’ordre sans affaiblir les critères.

## 29. Organisation, maintenance et modèle économique

### 29.1 Compétences nécessaires

Le programme final nécessite des responsabilités explicites en calcul scientifique structurel, contact/matériaux, CFD, EM, couplage, géométrie/maillage, HPC, GUI, qualification, documentation, sécurité et support. Le rôle scientifique de validation doit disposer d’une revue indépendante des auteurs des formulations.

La taille de l’équipe, le budget et le calendrier restent à établir après analyse des travaux par domaine. Une estimation globale sans choix natif/intégré ni niveau de qualification serait prématurée.

### 29.2 Gouvernance open source

Licence du cœur et des outils choisie avant développement substantiel ; politique de contributions, revues et propriété des données ; documentation des dépendances ; comité technique ; processus de décision publique pour les contrats stables ; publication des preuves accessibles et des limites de diffusion.

Une extension propriétaire peut coexister uniquement selon la licence retenue. La compatibilité des connecteurs et dépendances fait l’objet d’un examen avant distribution.

### 29.3 Exploitation industrielle

Versions stables avec durée de support publiée, correctifs prioritaires, migration des projets, canaux de signalement, bases de connaissances, formations et prestations de validation. Les niveaux de service contractuels sont définis selon les moyens réellement disponibles.

Options économiques à évaluer : support et formation, hébergement géré, prestations d’intégration et méthodes sectorielles. Le coût d’usage doit distinguer logiciel, calcul, stockage, maintenance et licences des outils externes.

## 30. Risques majeurs et réponses

| Risque | Effet | Réponse et déclencheur de revue |
| --- | --- | --- |
| Couverture trop large | Multiplication de modules peu fiables | Priorité C1/C2 ; aucune extension sans responsable et critères de qualification |
| Sous-estimation du contact/non-linéaire | Échec sur assemblages réels | Programme dédié, corpus difficiles et experts identifiés |
| Confusion convergence/validité | Décisions erronées | Bilans, domaines d’emploi et revue de méthode |
| Performances insuffisantes | Impossibilité d’usage industriel | Profilage et budgets mémoire avant promesses HPC |
| Réassociation CAO fragile | Conditions appliquées aux mauvaises faces | Confiance explicite et revalidation des ambiguïtés |
| Interopérabilité incomplète | Migration coûteuse ou incorrecte | Sous-ensembles déclarés et corpus d’import |
| Couplage instable | Résultats faux ou calculs interrompus | Tests conservation/stabilité et diagnostics globaux |
| Dépendance externe critique | Rupture de déploiement ou maintenance | Versions épinglées, stratégie de remplacement et distribution documentée |
| Dette de formats et API | Projets devenus illisibles | Versionnement, migrations et tests d’archives |
| Absence de financement durable | Produit sans maintenance | Modèle de support et budget par vague |
| Module spécialisé sans expertise | Couverture affichée mais non crédible | Qualification sectorielle et pilotes indépendants |

## 31. Décisions ouvertes et prochains livrables

| Décision | Moment requis | Critères |
| --- | --- | --- |
| Licence et gouvernance | Avant implémentation substantielle | Distribution, contributions, dépendances et extensions |
| Corpus C1 et premiers pilotes | Avant cadrage V1 | Usages répétables, accès aux données et références |
| Noyau géométrique et maillage | Avant parcours CAO | Qualité, licences, maintenance et associativité |
| Bibliothèque d’algèbre | Fondations puis revue HPC | Robustesse, classes de matrices et portabilité |
| Stockage des projets/résultats | Avant stabilisation API | Gros volumes, migrations et lecture partielle |
| Stratégie native/intégrée par domaine | Avant financement de V5 | Différenciation, coûts, validation et contrôle du déploiement |
| Cibles de performance définitives | Avant campagne C2 | Matériel, corpus, baseline et précision |
| Ressources et calendrier | Après décomposition des vagues | Compétences, risques et charge de qualification |

Livrables suivants : matrice de couverture versionnée, PRD de la vague V1, plan de validation par domaine, étude des dépendances structurantes, puis estimation de charge et budget. Le [PRD MVP](PRD.md) demeure la spécification applicable à la première implémentation.

## 32. Sources et nature du document

Les descriptions du portefeuille concurrent proviennent des pages officielles suivantes, consultées le 14 octobre 2026 :

- [SIMULIA — portefeuille et licensing](https://discover.3ds.com/simulia-unified-licensing).
- [SIMULIA — solutions académiques et domaines](https://www.3ds.com/edu/education/academics/solutions/simulia).
- [Abaqus — famille de produits](https://www.3ds.com/products/simulia/abaqus).
- [Abaqus/Standard — analyses et éléments](https://www.3ds.com/products/simulia/abaqus/standard).
- [SIMULIA — simulation structurelle, fatigue et optimisation](https://www.3ds.com/products/simulia/structural-simulation).
- [Abaqus — multiphysique et familles de couplage](https://www.3ds.com/products/simulia/abaqus/multiphysics).

La stratégie FerraSim, les exigences, l’ordre des vagues et les seuils proposés sont des décisions de cadrage de ce document. Ils ne proviennent pas de ces sources et ne constituent pas des capacités actuellement disponibles ou des résultats de benchmark.
