# FerraSim — Sprints S12 à S99 : du MVP au produit cible

Références : [roadmap générale](ROADMAP.md), [PRD de vision](PRD-VISION.md), [roadmap MVP](ROADMAP-MVP.md).

## 1. Lecture et engagement

Prérequis global : G0, MVP 0.1 qualifié. Tous les sprints ci-dessous sont à faire.

Les sprints décrivent un **découpage cible**, à transformer en backlog détaillé au début de chaque vague. La cadence proposée est de deux semaines ; le nombre de cycles nécessaire à chaque objectif sera estimé avec les experts et les prototypes. Les thèmes à risque élevé — contact, explicite, distribution, turbulence, RF et couplages — doivent prévoir des cycles supplémentaires si leurs preuves ne sont pas acquises.

Chaque ligne associe objectif, incrément démontrable et preuve attendue. Les seuils et protocoles détaillés sont fixés avant réalisation dans le PRD de livraison correspondant. Les exigences de documentation, migrations, diagnostics, revue et tests de la [définition commune](ROADMAP.md) s’appliquent à toutes les lignes.

**Deux états restent distincts :** sprint terminé et porte industrielle franchie. La fin d’un numéro de sprint ne confère pas automatiquement un niveau C1, C2 ou C3.

## 2. V1 — Parcours structurel complet : S12 à S23

**Valeur :** préparer et résoudre une étude de composant depuis une CAO, avec GUI, Python et résultats vérifiables.\
**Prérequis :** G0 ; corpus pilote et choix des outils CAO/maillage à établir au début de la vague.\
**Exigences :** PRD vision §§7–9, 12, 19–22 ; GEO, MSH, STR-01/04, UX, PST.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S12 | Définir le parcours C1 et les contrats extensibles | Corpus de composants ; besoins des pilotes ; schéma versionné, migrations et unités contrôlées ; choix stockage et outils | Parcours et métriques approuvés ; archives 0.1 migrées sans perte ; capacité V1 réestimée |
| S13 | Ouvrir l’API Python | Création de modèle, validation, solve et lecture de résultats ; packaging ; exemples | Calcul Python équivalent à CLI/Rust ; installation propre sur plateformes annoncées |
| S14 | Créer l’application graphique minimale | Arbre de modèle, maillage 3D, propriétés, gestion locale des travaux ; journal de commandes | Parcours du MVP dans la GUI et rejeu du modèle normalisé |
| S15 | Qualifier l’import CAO | Adaptateur STEP/B-Rep, instances, unités et diagnostic de géométrie | Corpus importé avec volumes/transformations contrôlés ; licences et distribution vérifiées |
| S16 | Générer un maillage depuis la CAO | Intégration du mailleur, régions nommées, tailles et qualité | Cas STEP → maillage utilisable ; invalidités rejetées ; sélections conservées |
| S17 | Enrichir les solides | Première formulation quadratique TET10 ; base HEX8 selon choix de priorité | Patch tests et convergence ; limitations d’intégration et conditionnement publiées |
| S18 | Ajouter les formulations 2D | TRI/QUAD sélectionnés, contraintes planes, déformations planes et axisymétrie | Analytique, repères, épaisseurs et chargements vérifiés |
| S19 | Livrer l’analyse modale | Matrice de masse, solveur de valeurs propres, affichage de modes | Fréquences, orthogonalité et convergence sur références |
| S20 | Livrer la thermique stationnaire | Conduction, propriétés, flux/convection imposée, températures | Solutions analytiques et bilan thermique ; unités absolues/écarts contrôlées |
| S21 | Relier thermique et structure | Mapping séquentiel de température et dilatation ; provenance du transfert | Dilatation libre/contrainte et transfert sur maillages distincts vérifiés |
| S22 | Consolider le parcours ingénieur | Réparation/simplification initiale, sondes, coupes, courbes, rapport et annuler/rétablir | Tâches utilisateur observées ; script rejouable ; rapport attaché à la révision |
| S23 | Qualifier la première alternative ciblée | Campagne C1, corrections, dossier de méthodes et release | G1 : cas ciblés passent ; préparation complète et reproduction GUI/CLI/Python démontrées |

**Dépendances internes :** S13/S14 après S12 ; S16 après S15 ; S19 après masses/formulations adaptées ; S21 après S20 et unités/mapping ; S23 après qualification de tous les composants du parcours. S17/S18 n’autorisent que les formulations effectivement validées.

**Limite :** cette vague n’inclut pas encore un environnement d’assemblages non linéaires. L’import STEP doit être opérationnel, mais la réassociation robuste à toutes les modifications de CAO reste à approfondir.

## 3. V2 — Assemblages non linéaires : S24 à S35

**Valeur :** résoudre des assemblages et des lois non linéaires avec un état de calcul traçable.\
**Prérequis :** G1 ; accès à un expert contact et à une revue matériaux/non-linéaire.\
**Exigences :** STR-02/03, MAT-02/03/04/09, CNT, NUM, GEO-05/06 et liaisons.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S24 | Définir les états et procédures incrémentales | État d’essai/validé, rollback, étapes de charge et contrat constitutif | Échec d’incrément sans corruption ; tests de cycles de mise à jour ; replanification V2 |
| S25 | Mettre en place la résolution non linéaire | Newton, tangent, résidus, recherche linéaire et réduction de pas | Problèmes non linéaires contrôlés ; convergence et annulations tracées |
| S26 | Ajouter grandes transformations et hyperélasticité | Cinématique finie et premier modèle hyperélastique | Objectivité, rotations rigides et essais multiaxiaux ; dérivées vérifiées |
| S27 | Ajouter plasticité métallique | Retour plastique, écrouissage isotrope puis cinématique selon périmètre | Essais monotones/cycliques, tangent et dissipation vérifiés |
| S28 | Enrichir temps et matériaux utilisateur | Viscoélasticité/fluage initial ; banc constitutif et SDK de loi utilisateur | Historique temporel reproduit ; paramètres et état extensible contrôlés |
| S29 | Construire la détection de contact | Surfaces, recherche de candidats, projection et normale ; diagnostic | Géométries de référence, mouvements et configurations dégénérées traités |
| S30 | Résoudre le contact normal | Première méthode pénalité, activation et tangent | Hertz et contact simple ; pénétration/forces et sensibilité quantifiées |
| S31 | Ajouter frottement et grands glissements | Loi de Coulomb initiale, suivi tangent, état stick/slip | Glissement et dissipation vérifiés ; convergence évaluée |
| S32 | Résoudre des assemblages | Précharge, interférence, liaisons multipoints, corps rigides et connecteurs | Assemblage boulonné ou emmanché ; redondances et bilans contrôlés |
| S33 | Enrichir structures minces et instabilités | Poutres/coques initiales, flambement précontraint et premier suivi de chemin | Membrane/flexion/cisaillement ; charges critiques et chemin sélectionné |
| S34 | Durcir le contact et les modifications de modèle | Auto-contact initial, contacts multiples, diagnostic ; réassociation CAO avec confiance | Cas de contact complexe ; ambiguïtés CAO bloquées ; pas de remapping silencieux |
| S35 | Qualifier le parcours assemblage | Corpus non linéaire, revue indépendante, pilotes et corrections | G2 : matériaux/contact/assemblage passent leur protocole ; limites et paramètres publiés |

**Dépendances internes :** S25–S28 après S24 ; S30 après S25/S29 ; S31 après S30 ; S32 après contact et prescriptions compatibles ; S33 après cinématique et solveur non linéaire ; S35 après fermeture des risques du corpus. S29–S34 constituent un programme de contact et peuvent recevoir plusieurs cycles supplémentaires.

## 4. V3 — Dynamique, durabilité et C2 : S36 à S49

**Valeur :** étendre la mécanique industrielle aux histoires temporelles, événements rapides et durée de vie.\
**Prérequis :** G2 ; experts dynamique, composites et fatigue ; infrastructure de performance et pilotes.\
**Exigences :** STR-04…10, MAT-05/06, NUM-05, HPC structurel, fatigue et qualification C2.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S36 | Livrer la dynamique implicite | Intégrateur, inertie, amortissement, chargements temporels | Oscillateurs et cas structuraux ; ordre temporel et bilans |
| S37 | Livrer harmonique et spectral | Champs complexes, fonctions de réponse et amortissement | Références fréquentielles ; conventions et combinaison spectrale définies |
| S38 | Prototyper la dynamique explicite | Intégrateur, masse diagonale, pas stable et bilans | Ondes/oscillateurs ; stabilité et énergie avant ajout d’impact complexe |
| S39 | Qualifier contact explicite et impact | Contact compatible, contrôle modes parasites, mesure de masse ajoutée | Impact de référence ; limites sur énergie artificielle et scaling fixées avant campagne |
| S40 | Interrompre et reprendre un calcul | Checkpoints, états, formats et transfert implicite/explicite sélectionné | Reprise équivalente au continu ; contrôle énergie/équilibre lors du transfert |
| S41 | Ajouter composites et interfaces | Stratifiés, orientations, contraintes par pli et interface cohésive initiale | Cas d’empilement et délaminage sélectionnés ; énergie et sensibilité au maillage |
| S42 | Livrer une méthode de fatigue initiale | Histoires, comptage, courbes S–N/ε–N, correction de contrainte moyenne | Références et données tracées ; résultats avec domaine d’emploi |
| S43 | Réduire et sous-modéliser | Sous-structures, corps flexibles réduits et transfert global–local | Erreurs de réduction et de frontière mesurées sur domaine annoncé |
| S44 | Optimiser le calcul CPU | Profilage, assemblage parallèle, allocation et vectorisation utiles | Gain mesuré sans régression scientifique ; reproducibilité dans les tolérances |
| S45 | Prototyper le calcul distribué | Partitionnement, solveur distribué retenu et communication | Scaling initial, bilan communication/mémoire et décision de poursuite |
| S46 | Qualifier HPC et ordonnanceur | Reprise distribuée, Slurm initial, estimation de ressources et packaging | Soumission/récupération/annulation ; panne contrôlée ; benchmark archivé |
| S47 | Qualifier les pilotes structurels | Au moins trois organisations pilotes indépendantes selon PRD | Parcours complets, retours, défauts et comparaison documentés ; prolonger si preuves indisponibles |
| S48 | Préparer exploitation et support | Distribution, docs expertes, formation, politique de versions, incidents et migration | Installation propre et exercice de support ; archives relues ; limites publiées |
| S49 | Franchir la porte structurelle industrielle | Campagne performance/robustesse/UX, audit scientifique et release | G3/C2 : critères de vision §27 satisfaits sur parcours explicites |

**Dépendances internes :** S38 avant S39 ; S40 après états de V2 et premiers intégrateurs ; S41 après coques et matériaux ; S45 après profilage S44 ; S46 après S40/S45 ; S49 après pilotes et campagnes. Les données expérimentales et l’accès aux concurrents doivent être préparés avant S47.

## 5. V4 — Campagnes, optimisation et collaboration : S50 à S61

**Valeur :** exécuter une méthode industrialisée à grande échelle et comparer des variantes.\
**Prérequis :** G3 ; décisions sur stockage, serveur, isolation et responsabilités sécurité.\
**Exigences :** §17, API distante, gestion des travaux, provenance, UX-07 et §24.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S50 | Définir campagnes et méthodes approuvées | Gabarits, variables permises, graphe de dépendances, révisions immuables | Modification d’entrée invalidant uniquement les résultats/approbations dépendants |
| S51 | Gérer des travaux distants de façon sûre | API asynchrone, identités, secrets, isolation, droits minimaux et états | Soumission/annulation/retry idempotents ; essais d’accès non autorisé et de séparation des travaux |
| S52 | Explorer les paramètres | DOE, lots, erreurs et reprise de campagne | Campagne reproductible avec candidats réussis/échoués conservés |
| S53 | Calibrer les matériaux | Données d’essai, ajustement, identification et incertitudes | Paramètres non identifiables signalés ; qualité d’ajustement et provenance |
| S54 | Calculer sensibilités et optimiser | Variables, objectifs, contraintes, gradients sélectionnés et méthodes sans gradients | Dérivées contrôlées ; problèmes à optimum connu ; échecs gérés |
| S55 | Livrer un parcours topologique | Optimisation structurelle initiale, filtres et contraintes de fabrication | Sensibilité au maillage, contraintes satisfaites et recalcul de vérification |
| S56 | Optimiser la forme et reconstruire | Paramétrisation, évolution géométrique et maillage ; reconstruction topologique | Géométrie finale vérifiée ; comparaison au modèle optimisé |
| S57 | Évaluer robustesse et incertitudes | Distributions, corrélations, propagation et modèles de substitution | Jeux indépendants de validation ; limites statistiques explicites |
| S58 | Organiser revue et approbation | Espaces partagés, rôles complets, audit, révisions, méthodes approuvées | Droits testés ; révision invalidant approbation ; actions attribuées |
| S59 | Exploiter de grands résultats | Stockage partiel, chunks, compression, niveaux de détail et comparaison | Navigation sur jeu ≥ 100 Go selon protocole ; mémoire bornée et intégrité |
| S60 | Durcir la plateforme de campagne | Quotas, sauvegarde, rétention, déploiement hors ligne ; revue sécurité | Charge cible 1 000 soumis/100 actifs sur infrastructure fixée ; restauration et isolation |
| S61 | Qualifier le parcours campagne | Cas industriel optimisation/calibration ; dossier d’exploitation | G4 : reproductibilité, revue, ressources et méthode finale vérifiés |

**Dépendances internes :** S51 installe la sécurité de base avant toute ouverture distante ; S52 après S50/S51 ; S54 après paramètres et exécution stable ; S55/S56 après sensibilités ; S57 après méthodes de campagne ; S60 complète sans remplacer les protections déjà livrées.

## 6. V5 — Extension des physiques : S62 à S79

**Valeur :** fournir des parcours qualifiés dans les nouvelles physiques sans fragmenter l’environnement.\
**Prérequis :** G4, responsables scientifiques par domaine, droits sur les références et décision native/intégrée.\
**Exigences :** CFD-01…08, EM-01…06, acoustique, multicorps, SDK, formats spécialisés.

Les quatre programmes sont indépendants après stabilisation du SDK. Leur développement peut être simultané avec des équipes affectées. Les séquences ci-dessous restent l’ordre de référence si les moyens sont limités.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S62 | Stabiliser les participants de simulation | SDK adaptateur, champs, régions, discretisations non FEM, états et licences | Faux participant de test puis premier connecteur ; erreurs/timeouts/annulation et versionnement |
| S63 | Préparer un modèle CFD | Domaine fluide, frontières, propriétés, couches limites et format spécialisé | Géométrie/maillage admissibles ; corpus et décision de solveur documentés |
| S64 | Résoudre le CFD laminaire | Connecteur qualifié ou première méthode native ; résidus et bilans | Poiseuille/Couette et convergence ; aucun lancement opaque présenté comme validation |
| S65 | Ajouter turbulence et chaleur | Modèle sélectionné, parois, transfert conjugué et observables | Données de référence, traitement de paroi et flux interface validés |
| S66 | Étendre compressibilité et mouvement | Compressible initial, référentiels tournants et maillage mobile sélectionné | Cas dédiés, conservation géométrique et bilan ; sous-périmètres visibles |
| S67 | Qualifier le parcours CFD | GUI/Python/post-traitement, benchmarks et déploiement | Domaine CFD sélectionné qualifié ; performance et limites publiées |
| S68 | Livrer EM statique/conductif | Électrostatique, conduction, repères et matériaux | Capacités/potentiels/courants de référence et bilans |
| S69 | Livrer magnétisme et pertes | Magnétostatique, courants de Foucault et non-linéarité sélectionnée | Flux, inductances et pertes sur références ; conditions de jauge documentées |
| S70 | Préparer l’EM haute fréquence | Solveur/intégration, maillage adapté, ports et frontières absorbantes | Résonateur et propagation ; conventions champs complexes et ports |
| S71 | Qualifier un parcours RF | Paramètres S, antenne ou cas EMC sélectionné ; post-traitement | Puissance, résonances/rayonnement et référence expérimentale selon parcours |
| S72 | Livrer l’acoustique intérieure | Fréquentiel, frontières, excitation et cavités | Modes et pressions de référence ; niveaux logarithmiques corrects |
| S73 | Étendre rayonnement et transitoire | Acoustique extérieure et temporelle selon méthodes retenues | Sensibilité des frontières artificielles et propagation mesurée |
| S74 | Relier acoustique et structure | Premier couplage vibro-acoustique et affichage | Plaque/cavité ou référence sélectionnée ; transfert et bilans |
| S75 | Qualifier le parcours acoustique | Corpus bruit/vibration, méthodes et ergonomie | Qualification annoncée par procédure ; pas de généralisation à tous les régimes |
| S76 | Livrer un mécanisme multicorps | Corps rigides, joints, contraintes et forces | Fermeture cinématique, pendule/mécanisme et bilan énergétique |
| S77 | Ajouter flexibilité et commandes | Corps réduits, entraînements et premier connecteur de système/FMI si retenu | Causalité, échange temporel et erreur de réduction contrôlés |
| S78 | Qualifier le parcours multicorps | Mécanisme industriel sélectionné, diagnostic et comparaison | Dérive contrainte, trajectoires et intégration du modèle qualifiées |
| S79 | Auditer la couverture multi-domaines | Matrice native/intégrée, packaging, versions et dossiers | G5 : un parcours validé par domaine requis ; lacunes bloquantes identifiées et corrigées |

**Dépendances internes :** tous les connecteurs après S62 ; S65/S66 après S64 ; S70 nécessite une décision RF et une expertise propres ; S74 après acoustique et contrats structurels ; S77 après S43/S76 ; S79 attend chaque qualification. La qualification EM basse fréquence est auditée avec S69 et confirmée à S79, séparément de la RF.

**Condition d’investissement :** décider à S62 de la profondeur native et des intégrations par domaine. Le temps d’installation, les licences et la reproductibilité des solveurs externes comptent dans la qualification. Les extensions multiphases avancées sont affectées à S92.

## 7. V6 — Couplages et portefeuille compétitif : S80 à S89

**Valeur :** exécuter des parcours multiphysiques cohérents et franchir C3.\
**Prérequis :** G5, participants qualifiés et modèle de campagne fiable.\
**Exigences :** CPL-01…07, NUM, HPC multi-domaines et critères C3.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S80 | Qualifier le mapping multi-maillages | Champs intensifs/extensifs, repères, projection et transferts conservatifs | Erreur de mapping et bilans sur interfaces non conformes |
| S81 | Orchestrer les couplages temporels | Graphe, sous-cyclage, relaxation, timeout et checkpoint global | Synchronisation et reprise cohérente de tous les participants |
| S82 | Coupler thermique et électricité | Effet Joule et procédure bidirectionnelle sélectionnée | Conservation et convergence sur cas de référence |
| S83 | Coupler thermique, contact et structure | Conduction de contact, chaleur de friction et rétroaction mécanique | Énergie d’interface et référence thermomécanique |
| S84 | Livrer l’interaction fluide–structure | Mapping forces/déformations, maillage mobile et couplage partitionné | Conservation, stabilité/masse ajoutée et transitoire de référence |
| S85 | Relier EM et mécanique | Pertes/efforts vers thermique et structure ; parcours machine sélectionné | Couple, flux de chaleur et trajectoire/structure vérifiés |
| S86 | Enrichir monolithique et co-simulation | Une procédure monolithique choisie ; protocoles externes et sécurité | Jacobien couplé et bilans ; arrêt/reprise/échec global contrôlés |
| S87 | Optimiser le portefeuille de calcul | HPC multi-domaines et évaluation GPU des points chauds | Scaling et précision par backend ; décision GPU fondée sur mesures |
| S88 | Réaliser les campagnes comparatives C3 | Parcours GUI/API, performance, robustesse, sécurité et pilotes | Seuils PRD §27 évalués avec échecs inclus ; versions et corpus publiés |
| S89 | Livrer le portefeuille qualifié | Matrice de couverture, distributions, dossiers et support | G6/C3 : chaque domaine requis et couplage prioritaire qualifié ; limites clairement annoncées |

**Dépendances internes :** S82–S86 après S80/S81 et leurs participants ; S84 après CFD mobile ; S85 après EM et thermique ; S88 exige les données pilotes et le matériel avant campagne. S87 ne reporte pas les besoins HPC propres aux modules qui étaient nécessaires à G5.

**Point de décision :** les backends GPU sans gain pertinent ou sans précision suffisante ne deviennent pas des backends qualifiés. Le statut C3 s’applique aux parcours déclarés, même si des fonctions avancées de la vision restent encore à approfondir.

## 8. Approfondissement de la vision finale : S90 à S99

**Valeur :** couvrir les fonctionnalités avancées du PRD au-delà des premiers parcours compétitifs.\
**Prérequis :** G6 ; financement et responsables par module spécialisé.\
**Exigences :** fonctions avancées des §§7–17, contrats durables et couverture finale.

Cette phase est un portefeuille de programmes spécialisés. Chaque ligne commence par cadrage du domaine d’emploi, puis peut être déclinée en plusieurs cycles, par exemple S92-A…S92-N. Le numéro de référence n’est pas une estimation de deux semaines pour implémenter l’ensemble d’une famille physique.

| Sprint | Objectif | Livrables | Preuve de sortie |
| --- | --- | --- | --- |
| S90 | Approfondir géométrie et éléments | Connecteurs CAO sélectionnés ; surfaces moyennes ; prismes/pyramides et éléments mixtes/hybrides | Corpus par connecteur/formulation ; qualité, incompressibilité et compatibilités |
| S91 | Adapter le maillage et régulariser la rupture | Adaptation h/p retenue, transfert d’état, fracture/endommagement avancé | Estimateur, convergence, énergie et objectivité au maillage |
| S92 | Étendre les régimes CFD spécialisés | Multiphase, surfaces libres, particules et combustion dans des domaines choisis | Conservation et données expérimentales par régime ; programme replanifié selon expertise |
| S93 | Ajouter milieux poreux et matériaux spécialisés | Poromécanique, sols/béton/mousses, modèles dépendants et calibration | Qualification séparée de chaque loi ; limites identifiables |
| S94 | Ajouter fonctions thermiques et transducteurs | Rayonnement non linéaire, changement de phase, piézoélectricité et diffusion | Bilans et références indépendantes ; unités et couplages |
| S95 | Construire le module batteries sélectionné | Transport, réactions, lois électrochimiques et couplages utiles au parcours choisi | Données et protocoles sectoriels ; validation et sensibilités expérimentales |
| S96 | Approfondir fatigue et matériaux composites | Fatigue multiaxiale, critères/rupture par pli, propagation et assemblages sélectionnés | Données d’essai, traitement des incertitudes et méthodes versionnées |
| S97 | Étendre systèmes et optimisation | Contacts multicorps spécialisés ; optimisation multiphysique ; modèles de substitution qualifiés | Contrôles des dérivées, couplages, domaine d’apprentissage et recalcul final |
| S98 | Stabiliser l’expérience et l’écosystème | Accessibilité FR/EN complète, méthodes/gabarits, SDK, interopérabilité additionnelle, migrations et archivage | Corpus d’archives/imports ; UX observée ; pas de perte physique silencieuse |
| S99 | Auditer le produit cible et sa maintenance | Couverture PRD, requalification des évolutions, sécurité, support et dossier global | G7 : chaque exigence cible satisfaite dans son domaine déclaré, ou maintien explicite de la porte ouverte |

**Règle de complétude :** une lacune de la vision n’est pas masquée par le statut C3. Si un module est retiré ou différé, le PRD de vision et la matrice doivent être révisés explicitement ; le produit ne peut pas être déclaré conforme à toute la vision initiale par simple décision de fin de planning.

**Choix natif/intégré :** les programmes S90–S97 peuvent approfondir des capacités natives ou des intégrations qualifiées. Une ambition de tout réécrire en Rust constituerait un périmètre supplémentaire à estimer et financer séparément.

## 9. Couverture du PRD de vision

| Famille de besoins | Sprints de référence | Porte principale |
| --- | --- | --- |
| Marchés, parcours, critères et pilotes | S12, S23, S35, S47–S49, S88–S89 | G1, G3, G6 |
| Géométrie et assemblages | S15–S16, S22, S32–S34, S90 | G1, G2, G7 |
| Maillage et éléments | S16–S18, S33, S63, S90–S91 | G1, G2, G5, G7 |
| Structure linéaire, modes et flambement | S17–S19, S33, S37 | G1–G3 |
| Non-linéaire et matériaux | S24–S28, S41, S91, S93, S96 | G2, G3, G7 |
| Contact et assemblages | S29–S35, S39, S83 | G2, G3, G6 |
| Dynamique, reprise et réduction | S36–S40, S43, S46 | G3 |
| Thermique et champs complémentaires | S20–S21, S82–S83, S94–S95 | G1, G6, G7 |
| CFD | S63–S67, S84, S92 | G5–G7 |
| EM basse/haute fréquence | S68–S71, S85, S94 | G5–G7 |
| Acoustique et vibro-acoustique | S72–S75 | G5 |
| Multicorps et systèmes | S76–S78, S97 | G5, G7 |
| Couplages | S21, S74, S80–S86 | G1, G5, G6 |
| Fatigue | S42, S96 | G3, G7 |
| Optimisation, calibration et incertitudes | S52–S57, S97 | G4, G7 |
| Résultats, visualisation et rapports | S14, S22, S59, S67/S71/S75/S78, S98 | Toutes vagues |
| GUI et ergonomie | S14, S22, S34, S58, S88, S98 | G1, G4, G6, G7 |
| API, formats, interopérabilité et SDK | S12–S13, S50–S51, S59, S62, S98 | G1, G4–G7 |
| Solveurs et qualité numérique | S17–S21, S24–S40, chaque physique | Toutes portes scientifiques |
| HPC et ressources | S44–S46, S60, S67, S87 | G3, G4, G5, G6 |
| Sécurité et collaboration | S51, S58, S60, S62, S86, S88, S99 | G4–G7 |
| Open source, maintenance et support | S00, S48, S61, S79, S89, S99 | G0, G3–G7 |

Les responsabilités non fonctionnelles restent continues : sécurité à chaque nouvelle surface distante, tests d’archives à chaque changement de format, revue de licences à chaque dépendance, qualification à chaque méthode ou backend.

## 10. Décisions nécessaires avant engagement de chaque vague

| Vague | Décisions d’entrée |
| --- | --- |
| V1 | Utilisateurs C1, parcours, CAO/mailleur, GUI, stockage et distribution Python |
| V2 | Formulations non linéaires, modèles matériau, méthode contact et corpus assemblage |
| V3 | Intégrateurs, modèles fatigue/composites, stratégie distribué, matériel et pilotes |
| V4 | Architecture serveur, authentification, stockage partagé et méthodes d’optimisation |
| V5 | Responsables scientifiques, native/intégrée par domaine, licences et références |
| V6 | Couplages prioritaires, protocoles, critères d’interface et campagne comparative |
| Vision approfondie | Domaine d’emploi par module spécialisé, données, financement et stratégie de maintenance |

## 11. Après G7

Le produit reste soumis à maintenance, requalification, support et évolution des besoins. La roadmap suivante est alimentée par incidents, usages pilotes, changements d’environnement et nouvelles méthodes. « Produit final » désigne ici une cible de couverture et de qualification, pas l’arrêt du développement.
