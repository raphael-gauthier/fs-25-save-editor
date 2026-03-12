# Phase 24 - Density Maps : lecture, écriture et cache

## Objectif

Remplacer les données imprécises de `fields.xml` (résumés IA) par les données réelles des cartes de densité binaires (GDM/GRLE). Permettre l'édition directe des fichiers binaires et mettre en cache les données pour un affichage instantané.

## Contexte

`fields.xml` stocke les résumés du système IA/missions, PAS l'état réel des champs en jeu. Les vraies données sont dans les fichiers binaires :
- **GDM** (Giants Density Map) : compression par chunks/palettes (fruits, sol, mauvaises herbes, pierres)
- **GRLE** (Giants RLE) : compression RLE (chaux, engrais, labour, rouleau, broyage, terrains)

## Étapes réalisées

### 24.1 Parsers binaires (Backend)

#### `parsers/grle.rs`
- Parser RLE avec header 20 octets (magic GRLE + version + dimensions + bpp + taille compressée)
- Writer RLE pour réécriture après modification
- `parse_grle()`, `parse_grle_with_header()`, `write_grle()`

#### `parsers/gdm.rs`
- Parser chunk-based avec palettes (magic `"MDF` ou `!MDF`)
- Support multi-ranges de compression combinées par bit-shifting
- Writer chunk-based avec encodage direct (bit_depth > 2) ou palette (bit_depth <= 2)
- `parse_gdm()`, `write_gdm()`

### 24.2 Résolution des types de fruits

#### `parsers/density_map_config.rs`
4 sources de résolution (dans l'ordre) :
1. `maps_fruitTypes.xml` — 25 types de base (indices 1-25)
2. XML de la carte (ex: `mapUS.xml`) — types spécifiques (MEADOW à l'index 26)
3. Log du jeu (`log.txt`) — types DLC (ONION à l'index 27+)
4. Liste de fallback `KNOWN_EXTRA_FRUIT_TYPES` — cultures de base absentes du log (GREENBEAN, PEA, SPINACH)

### 24.3 Service d'agrégation et d'édition

#### `services/density_map.rs`
- `aggregate_field_data()` : lecture + agrégation par terrain (distribution fruits, croissance, traitements)
- `save_density_edits()` : lecture → décodage → modification des pixels du terrain → réencodage → écriture
- Résolution des chemins : cartes intégrées depuis gamePath, cartes moddées depuis archives zip
- Facteur d'échelle entre farmlands GRLE (1024x1024) et info layers (4096x4096)

### 24.4 Commandes Tauri

#### `commands/density.rs`
- `load_field_density_data` : charge les données de densité pour tous les terrains
- `save_density_edits` : écrit les modifications dans les fichiers binaires

### 24.5 Modèles de données

#### `models/density.rs`
- `FieldDensityData` : données agrégées par terrain (distribution, croissance, traitements)
- `DensityEditPayload` : modifications à appliquer (fruit, croissance, chaux, engrais, etc.)

### 24.6 Frontend : affichage et cache

#### `stores/field.ts`
- État : `densityData`, `densityLoading`, `densityError`, `densityFromCache`
- Stratégie cache-first : données du cache affichées instantanément, rafraîchissement en arrière-plan
- Cache dans `density-cache.json` via `@tauri-apps/plugin-store`, clé = nom du dossier de sauvegarde
- Tracking des éditions de densité : `densityEdits` Map, sauvegardé via commande séparée

#### `views/FieldListView.vue`
- Colonnes fruit/croissance/sol : skeleton pendant le chargement (si gamePath configuré)
- Bannière d'alerte quand les données viennent du cache
- Fallback XML uniquement si gamePath non configuré
- Fonction `formatFruitName()` : affiche "Inconnu (29)" au lieu de "UNKNOWN_29"

#### `components/fields/FieldEditor.vue`
- Section densité avec actions rapides (max croissance, max chaux, etc.)
- Section XML masquée derrière le mode avancé quand la densité est disponible
- Description du sheet utilise le fruit dominant de la densité

### 24.7 i18n
- Clés ajoutées : `densityLoading`, `densityError`, `gamePathRequired`, `densityTitle`, `xmlEditTitle`, `fruitDistribution`, `densityActions`, `densityEditPending`, `maxPlow`, `unknownFruit`, `densityCacheRefreshing`, `densityCacheStale`

## Critères de validation

- [x] Les données de champs proviennent des density maps, pas de fields.xml
- [x] Les types de fruits sont résolus depuis 4 sources (XML, map XML, log, fallback)
- [x] L'édition des density maps fonctionne (fruit, croissance, traitements)
- [x] Le cache affiche les données instantanément au chargement suivant
- [x] La bannière "données en cache" s'affiche pendant le rafraîchissement
- [x] Les skeletons s'affichent quand il n'y a pas de cache et que le chargement est en cours
- [x] Les données XML sont masquées derrière le mode avancé quand la densité est disponible
- [x] Les cartes intégrées (MapUS, MapEU, MapAS) et moddées (zip) sont supportées
- [x] Les types inconnus s'affichent proprement : "Inconnu (29)" au lieu de "UNKNOWN_29"
- [x] Round-trip : modifier → sauvegarder → recharger → valeurs correctes
