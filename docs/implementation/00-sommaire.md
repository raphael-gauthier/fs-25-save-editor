# Documentation d'implémentation - FS25 Save Editor

## Principe

Ce guide découpe l'implémentation en **20 micro-phases** ordonnées. Chaque phase produit un résultat **testable et vérifiable** avant de passer à la suivante. Les phases sont regroupées en blocs logiques.

## Vue d'ensemble des phases

### Bloc A - Fondations (socle technique)

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [01](./01-setup-projet.md) | Setup projet Tauri + Vue | `pnpm tauri dev` ouvre une fenêtre vide |
| [02](./02-structure-backend.md) | Structure backend Rust | Commande `greet` fonctionne via IPC |
| [03](./03-shell-ui.md) | Shell UI (layout, sidebar, routing) | Navigation entre pages vides avec sidebar |

### Bloc B - Détection & chargement des sauvegardes

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [04](./04-detection-saves.md) | Détection des sauvegardes (backend) | `list_savegames` retourne les saves du disque |
| [05](./05-ui-selection-save.md) | UI sélection de sauvegarde | Écran d'accueil affiche les saves avec infos |
| [06](./06-chargement-savegame.md) | Chargement complet d'une sauvegarde | `load_savegame` parse career + farms + vehicles + sales |

### Bloc C - Système de backup

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [07](./07-backup-system.md) | Système de backup | Création, listage, restauration, suppression de backups |

### Bloc D - Éditeur de finances

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [08](./08-editeur-finances.md) | Éditeur de finances (UI + écriture) | Modifier l'argent/prêt, sauvegarder, vérifier dans le XML |

### Bloc E - Éditeur de véhicules

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [09](./09-ui-liste-vehicules.md) | UI liste des véhicules | Liste avec recherche/filtres, navigation vers détail |
| [10](./10-detail-vehicule.md) | Détail véhicule + écriture | Éditer prix/âge/fill levels, sauvegarder dans le XML |
| [11](./11-actions-groupees-vehicules.md) | Actions groupées véhicules | Sélection multiple, remplir tout, remettre à neuf |

### Bloc F - Marché d'occasion

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [12](./12-marche-occasion.md) | Marché d'occasion complet | Lister, éditer, actions rapides, écriture XML |

### Bloc G - Fonctionnalités transverses

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [13](./13-i18n-parametres.md) | i18n, thème et paramètres | Changement langue/thème instantané, persistance |
| [14](./14-dirty-tracking.md) | Dirty tracking et garde navigation | Badge modifications, confirmation avant quitter |

### Bloc H - Phase 2 fonctionnelle (enrichissement)

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [15](./15-champs-cultures.md) | Champs et cultures | Lister/éditer champs, actions groupées, propriété terrains |
| [16](./16-monde-meteo.md) | Monde et météo | Éditer jour/heure, prévisions météo, actions rapides |
| [17](./17-batiments.md) | Bâtiments et structures | Lister/éditer bâtiments, terminer constructions, stocks |
| [18](./18-missions-collectibles.md) | Missions et collectibles | Éditer missions/récompenses, marquer collectibles |

### Bloc I - Enrichissement visuel

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [20](./20-images-vehicules.md) | Images de preview des véhicules | Thumbnails DDS→PNG dans listes véhicules et marché |

### Bloc J - Finalisation

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [19](./19-finalisation.md) | Tests, polish et build | Tests passent, build produit un exécutable fonctionnel |

### Bloc K - Distribution & mises à jour

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [22](./22-mise-a-jour-automatique.md) | Vérification de mise à jour | Check GitHub Releases au démarrage, popup changelog, lien téléchargement |

### Bloc L - Données de densité (champs)

| Phase | Titre | Vérification |
|-------|-------|--------------|
| [24](./24-density-maps.md) | Density maps : lecture, écriture et cache | Données de champs précises depuis GDM/GRLE, édition, cache local |

## Prérequis

Avant de commencer la Phase 01, s'assurer que les outils suivants sont installés :

- **Node.js** 20+
- **pnpm** 9+
- **Rust** toolchain stable (via `rustup`)
- **Tauri CLI** (`cargo install tauri-cli`)
- **WebView2** (Windows, généralement pré-installé sur Windows 10/11)

## Convention de vérification

Chaque phase se termine par une section **"Critères de validation"** listant les vérifications à effectuer avant de passer à la suite. Ne passez à la phase suivante que si **tous les critères sont verts**.
