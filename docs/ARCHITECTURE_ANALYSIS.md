# NovaTune Architecture Analysis

## Objectif

Documenter l'architecture réellement en place après le refactor, et préciser ce qui a été simplifié tout en conservant un coeur partagé entre les apps.

## Résumé exécutif

Le dépôt a maintenant une architecture plus cohérente qu'au départ :

- `tuner-core` porte le domaine partagé
- `tuner-dsp-algo` porte l'algorithme DSP
- `tuner-engine` porte le moteur applicatif commun
- `tuner-dsp-web` et `tuner-dsp-native` sont revenus à un rôle d'adaptateurs
- `packages/tuner-web-core` isole le runtime web TypeScript
- `webapp-mobile` et `tuner-cli` sont allégés côté shell/UI

Le point principal du refactor a bien été atteint : le partage ne s'arrête plus au domaine et au DSP. Le comportement du tuner est maintenant centralisé dans une implémentation Rust unique.

## État actuel

### Monorepo Rust

Le workspace Rust dans [Cargo.toml](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\Cargo.toml) regroupe :

- `crates/tuner-core`
- `crates/tuner-dsp-algo`
- `crates/tuner-engine`
- `crates/tuner-dsp-native`
- `crates/tuner-dsp-web`
- `crates/tuner-dsp-embedded`
- `apps/tuner-cli`

Le frontend web actif est [apps/webapp-mobile](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile), appuyé par le package partagé [packages/tuner-web-core](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\packages\tuner-web-core).

### Domaine partagé

Le domaine reste centralisé dans [crates/tuner-core/src/lib.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-core\src\lib.rs), avec notamment :

- `TunerMode`
- `UiState`
- `TuningTarget`
- `TunerOutput`

`TunerOutput` porte maintenant aussi `clarity` et `rms`, ce qui permet au même coeur Rust d'alimenter correctement le CLI et le web.

Les règles de mapping restent concentrées dans :

- [mapping.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-core\src\mapping.rs)
- [tuning.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-core\src\tuning.rs)

### Algorithme DSP partagé

[crates/tuner-dsp-algo/src/lib.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-dsp-algo\src\lib.rs) reste le socle de détection de pitch partagé :

- preprocessing
- NSDF / peak detection
- smoothing
- pitch detector

Ce module n'est toujours pas couplé à une UI ni à une plateforme.

### Moteur applicatif partagé

La simplification majeure est l'introduction de [crates/tuner-engine](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-engine).

Ce module porte désormais :

- `TunerConfig`
- `PitchDetectorConfig`
- `SessionMode`
- `TunerEngine`

Le comportement produit du tuner y est centralisé :

- modes `preset` et `chromatic`
- calibration
- seuils RMS / clarté
- stabilité multi-frames
- mapping final vers `TunerOutput`

Le CLI ne possède plus son propre moteur local.

### Adaptateurs de plateforme

Les adaptateurs ont été aminci :

- [tuner-dsp-native](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-dsp-native\src\audio.rs) gère l'audio natif
- [tuner-dsp-web](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-dsp-web\src\lib.rs) est une façade WASM autour de `tuner-engine`
- [tuner-dsp-embedded](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-dsp-embedded\src\lib.rs) reste un support technique sans app dédiée active

## Ce qui a été simplifié

### 1. Un moteur produit unique

Le problème initial principal a été supprimé :

- le moteur tuner du CLI local a été retiré
- `tuner-dsp-web` ne reconstruit plus une logique métier concurrente
- `tuner-engine` est maintenant la source unique du comportement produit

### 2. Un runtime web partagé

Le runtime web n'est plus collé uniquement à l'app React.

Le package [packages/tuner-web-core](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\packages\tuner-web-core) concentre maintenant :

- types TS partagés
- wrapper `wasmTuner`
- helpers presets
- utilitaires audio web

`webapp-mobile` garde localement le chargement concret du bridge généré dans [bridgeLoader.ts](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\tuner\bridgeLoader.ts), ce qui est cohérent car ce point dépend des artefacts WASM de l'app.

### 3. Un frontend React moins couplé

Le frontend principal a été découpé.

[App.tsx](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\App.tsx) est maintenant un point de composition plus léger, et s'appuie sur :

- [useTunerController.ts](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\tuner\useTunerController.ts)
- [TunerPanel.tsx](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\tuner\TunerPanel.tsx)
- [SettingsView.tsx](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\settings\SettingsView.tsx)
- les composants du dossier [features/tuner/components](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\tuner\components)
- le layout [features/layout](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\layout)

Les contrats frontend ont aussi été regroupés par objets (`selection`, `settings`, `runtime`, `themeControls`) au lieu de longues listes plates de props.

### 4. Un CLI plus lisible

Le CLI reste piloté par `tuner-engine`, mais sa couche TUI a aussi été clarifiée.

[state.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\tuner-cli\src\tui\state.rs) expose maintenant des sous-structures de rendu :

- `header`
- `pitch`
- `telemetry`
- `overlays`

Cela aligne mieux la couche terminale avec la séparation adoptée côté web.

## Points forts actuels

### 1. Mutualisation correcte du comportement

Le coeur partagé couvre maintenant :

- domaine
- DSP
- comportement tuner

Le repo a donc franchi le cap entre "shared low-level core" et "shared application core".

### 2. Frontières plus nettes

Le découpage actuel est défendable :

- `tuner-core` : domaine
- `tuner-dsp-algo` : DSP
- `tuner-engine` : runtime applicatif
- `tuner-dsp-*` : adaptateurs techniques
- `packages/tuner-web-core` : runtime TS web
- `apps/*` : shells UI

### 3. Validation en place

Les validations exécutées sur l'état actuel couvrent :

- `cargo test`
- `npm run typecheck` dans `apps/webapp-mobile`
- `npm run test` dans `apps/webapp-mobile`
- `npm run build` dans `apps/webapp-mobile`

## Points encore ouverts

### 1. `packages/tuner-web-core` reste partiellement dépendant du contexte de l'app

L'extraction est utile, mais incomplète au sens strict :

- le chargement du bridge généré reste dans `webapp-mobile`
- les scripts WASM restent encore portés par l'app

Ce n'est pas incohérent, mais cela signifie que le runtime web n'est pas encore totalement portable hors de l'app actuelle.

### 2. Le support embarqué reste une décision ouverte

`crates/tuner-dsp-embedded` existe encore, mais sans shell actif.

Deux positions restent possibles :

- le conserver comme porte d'entrée technique future
- le retirer pour réduire la surface du repo

### 3. La documentation doit désormais être tenue comme documentation d'état, pas comme plan

Le refactor principal étant appliqué, les documents d'architecture ne doivent plus présenter `tuner-engine` ou `tuner-web-core` comme de simples intentions.

## Conclusion

L'architecture a effectivement été simplifiée sans perdre la modularité.

Le point de mutualisation a été déplacé au bon niveau :

- `tuner-core` pour le domaine
- `tuner-engine` pour le comportement partagé
- des adaptateurs minces par plateforme
- des apps recentrées sur l'interface

Le repo est maintenant plus cohérent entre ses deux clients actifs, [tuner-cli](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\tuner-cli) et [webapp-mobile](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile).
