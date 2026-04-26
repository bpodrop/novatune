# NovaTune Target Architecture

## Objectif

Décrire la cible d'architecture désormais retenue pour le repo, en distinguant :

- ce qui est déjà implémenté
- ce qui reste une extension optionnelle

## Architecture retenue

### Vue d'ensemble

```text
apps/
  tuner-cli/
  webapp-mobile/

crates/
  tuner-core/
  tuner-dsp-algo/
  tuner-engine/
  tuner-dsp-native/
  tuner-dsp-web/
  tuner-dsp-embedded/

packages/
  tuner-web-core/
```

## Niveaux de responsabilité

### `crates/tuner-core`

Rôle :

- domaine pur
- notes
- presets
- structures métier stables
- mapping métier de base

Ne doit pas contenir :

- audio
- binding WASM
- UI
- orchestration spécifique à une plateforme

### `crates/tuner-dsp-algo`

Rôle :

- algorithmes de détection de pitch
- smoothing
- preprocessing

Ce module reste indépendant du moteur produit et de l'UI.

### `crates/tuner-engine`

Rôle :

- moteur applicatif partagé du tuner

Ce module est maintenant la vraie source de vérité pour :

- modes `preset` / `chromatic`
- calibration
- règles de session
- stabilité multi-frames
- seuils runtime
- production des sorties utilisées par les clients

Il doit rester indépendant de :

- React
- TUI
- WASM
- CPAL

### `crates/tuner-dsp-native`

Rôle :

- capture audio native
- adaptation à l'entrée micro pour le CLI

Ce crate ne doit pas réintroduire de logique métier de tuning.

### `crates/tuner-dsp-web`

Rôle :

- binding WASM
- façade JS-friendly au-dessus de `tuner-engine`

Ce crate doit rester mince :

- handles
- session WASM
- conversion de types
- exports `wasm_bindgen`

### `crates/tuner-dsp-embedded`

Rôle :

- point d'entrée technique pour une cible embarquée éventuelle

Statut actuel :

- conservé
- sans app active dédiée

### `packages/tuner-web-core`

Rôle :

- runtime web TypeScript partagé

Contient aujourd'hui :

- types TS partagés
- wrapper `wasmTuner`
- logique presets web
- utilitaires audio web

Ce package doit rester indépendant de l'UI produit.

### `apps/webapp-mobile`

Rôle :

- UI React/Vite principale
- expérience mobile-first
- composition de l'interface

Cette app consomme :

- `tuner-web-core`
- le bridge WASM généré localement

Son `App.tsx` doit rester un assembleur léger, et la logique doit continuer à vivre dans `features/*`.

### `apps/tuner-cli`

Rôle :

- parsing CLI
- boucle terminale live
- rendu TUI

Cette app consomme :

- `tuner-engine`
- `tuner-dsp-native`

Sa couche d'état UI doit rester dérivée du moteur, sans recréer de logique produit parallèle.

## Flux retenus

### Flux CLI

```text
Audio input -> tuner-dsp-native -> tuner-engine -> tuner-cli TUI
```

### Flux Web

```text
Browser microphone -> tuner-web-core -> tuner-dsp-web (WASM) -> tuner-engine -> webapp-mobile UI
```

### Flux embarqué potentiel

```text
Embedded input -> tuner-dsp-embedded -> tuner-engine -> future embedded shell
```

## Principes à préserver

### 1. Une seule implémentation du comportement tuner

Le comportement produit ne doit exister qu'une seule fois : dans `tuner-engine`.

### 2. Des adaptateurs minces

Les crates `tuner-dsp-*` ne doivent pas redevenir des endroits où de la logique produit se reconstitue.

### 3. Des shells UI légers

Les apps ne doivent pas porter :

- règles de stabilité
- logique de session produit
- mapping final de tuning

### 4. Un runtime web séparé de l'UI

Le runtime TypeScript partagé doit continuer à vivre hors de l'app React.

## Extensions possibles

### 1. Extraction supplémentaire du bridge loader

Si le packaging des artefacts WASM est stabilisé, `bridgeLoader` pourra éventuellement rejoindre `packages/tuner-web-core`.

### 2. Rationalisation du support embarqué

Deux options restent légitimes :

- conserver `tuner-dsp-embedded` comme point d'extension
- le retirer si aucune cible réelle n'est prévue

## Définition de la cible

La cible d'architecture n'est plus théorique :

1. `tuner-core` reste petit et stable
2. `tuner-engine` centralise le comportement produit
3. `tuner-dsp-web` et `tuner-dsp-native` restent techniques
4. `packages/tuner-web-core` concentre le runtime TS partagé
5. `webapp-mobile` et `tuner-cli` restent des shells UI

Le travail restant éventuel porte surtout sur des raffinements de frontière, pas sur une réorganisation de fond.
