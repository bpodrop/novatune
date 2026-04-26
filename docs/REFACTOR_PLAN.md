# NovaTune Refactor Status

## Objectif

Conserver une trace claire de ce qui a été appliqué, de ce qui reste ouvert, et de la logique de séquencement suivie pendant le refactor.

## Bilan global

Le coeur du plan de refactor a été appliqué.

## Appliqué

### 1. Extraction de `tuner-engine`

Statut :

- fait

Implémentation :

- [crates/tuner-engine](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-engine)

Effets :

- le moteur applicatif commun existe
- le CLI ne porte plus de moteur local concurrent
- le comportement tuner est mutualisé côté Rust

### 2. Réduction de `tuner-dsp-web` à un binding

Statut :

- fait

Implémentation :

- [crates/tuner-dsp-web/src/lib.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\crates\tuner-dsp-web\src\lib.rs)

Effets :

- le bridge WASM repose sur `tuner-engine`
- la logique produit n'est plus dupliquée côté web Rust

### 3. Introduction de `packages/tuner-web-core`

Statut :

- fait

Implémentation :

- [packages/tuner-web-core](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\packages\tuner-web-core)

Effets :

- les types TS, wrappers WASM, helpers presets et utilitaires audio sont extraits
- `webapp-mobile` n'est plus le seul endroit où cette logique vit

### 4. Décomposition de `webapp-mobile`

Statut :

- fait

Implémentation principale :

- [App.tsx](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\App.tsx)
- [features/tuner](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\tuner)
- [features/settings](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\settings)
- [features/layout](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\webapp-mobile\src\features\layout)

Effets :

- `App.tsx` est devenu un assembleur léger
- l'orchestration runtime est dans un hook dédié
- les composants UI sont séparés et testés

### 5. Harmonisation de la TUI CLI

Statut :

- fait

Implémentation principale :

- [apps/tuner-cli/src/tui/state.rs](C:\WORKSPACE\00-DropD.Tech\GIT\novatune\apps\tuner-cli\src\tui\state.rs)

Effets :

- l'état de rendu CLI est regroupé en blocs cohérents
- la TUI est plus lisible
- la séparation moteur / rendu est meilleure

## Validation exécutée

### Rust

- `cargo test`

### Web

Dans `apps/webapp-mobile` :

- `npm run typecheck`
- `npm run test`
- `npm run build`

### Notes d'exécution

Sous Windows, `vite` et `vitest` ont nécessité une exécution hors sandbox à cause d'un `spawn EPERM` au chargement de leurs helpers. Le code validé est néanmoins propre.

## Restant optionnel

### 1. Déplacement éventuel de `bridgeLoader`

Statut :

- optionnel

Raison :

- le chargement des artefacts générés reste spécifique au packaging de `webapp-mobile`

### 2. Décision finale sur `tuner-dsp-embedded`

Statut :

- ouvert

Options :

- conserver la crate comme porte d'entrée technique future
- la supprimer pour réduire la surface du repo

### 3. Nettoyage documentaire continu

Statut :

- à maintenir

Raison :

- les docs doivent désormais suivre l'implémentation, plus un plan théorique

## Ordre de réalisation effectivement suivi

1. mise à jour des docs et du workspace après suppression des apps obsolètes
2. extraction de `tuner-engine`
3. migration du CLI vers `tuner-engine`
4. simplification de `tuner-dsp-web`
5. création de `packages/tuner-web-core`
6. découpage de `webapp-mobile`
7. ajout de tests ciblés sur les nouveaux composants web
8. harmonisation de la TUI CLI

## Définition de succès atteinte

Les critères principaux sont maintenant remplis :

1. le comportement tuner est défini une seule fois côté Rust
2. le CLI et le web utilisent le même moteur applicatif
3. `webapp-mobile` utilise un socle TS partagé pour le runtime web
4. `App.tsx` ne porte plus toute la logique de session
5. l'architecture est plus lisible dossier par dossier
