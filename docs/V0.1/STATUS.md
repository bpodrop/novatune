# Status

## Règle d'usage

- `ROADMAP.md` définit la direction
- `TASKS.md` définit les tâches opérationnelles
- `STATUS.md` suit l'avancement courant
- mettre à jour ce fichier à la fin de chaque lot significatif
- rester factuel et court

## Légende

- `todo` : non démarré
- `in_progress` : en cours
- `blocked` : bloqué par une dépendance ou une décision
- `done` : terminé

## État global

- phase_courante : `V0.1 validée`
- statut_global : `done`
- dernier_point_connu : `clippy workspace propre et regroupement de paramètres tracé en documentation`

## Tâches

| ID | Titre | Statut | Owner | Notes |
| --- | --- | --- | --- | --- |
| T-001 | Introduire un TunerEngine | done | codex | Couche d'orchestration applicative introduite dans `cli::app` |
| T-002 | Brancher les fenêtres glissantes avec hop size | done | codex | `AudioCapture` produit maintenant des fenêtres glissantes avec recouvrement |
| T-003 | Stabiliser la lecture publiée | done | codex | Seuil de publication, hystérésis de note et stabilité multi-frames ajoutés |
| T-004 | Transformer analyze en boucle continue | done | codex | Boucle continue, rendu console vivant et arrêt explicite via `q` |
| T-005 | Durcir les heuristiques DSP | done | codex | Sélection de pics renforcée et tests guitare/bruit ajoutés |
| T-006 | Renforcer la gestion audio runtime | done | codex | Source audio explicite, raison du fallback et messages CLI clarifiés |
| T-007 | Compléter la couverture de test | done | codex | Tests de séquences moteur et documentation runtime/CLI complétées |
| T-010 | Introduire la TUI terminal | done | codex | TUI ratatui/crossterm, jauge 3 lignes, mode démo et intégration live |
| T-011 | Moderniser la TUI terminal | done | codex | Header badge, carte centrale arrondie, footer métriques et jauge produit |
| T-012 | Refactorer la TUI cyberpunk modulaire | done | codex | `theme/layout/widgets`, nouveaux états visuels, télémétrie enrichie et micro-animations sobres |
| T-008 | Maintenir la couche musicale | todo | unassigned | Tâche transversale |
| T-009 | Maintenir le pipeline NSDF minimal | todo | unassigned | Tâche transversale |

## Blocages

- aucun

## Journal

### 2026-03-30

- `done` création de `ROADMAP.md`
- `done` alignement de `TASKS.md` sur la roadmap
- `done` création de `STATUS.md`
- `done` implémentation de `T-001` avec `TunerEngine` et branchement initial dans la CLI
- `done` implémentation de `T-002` avec usage réel du `hop_size` dans `AudioCapture`
- `done` implémentation de `T-003` avec rejet d'outliers, seuil de publication et hystérésis
- `done` implémentation de `T-004` avec boucle continue et rendu console interactif
- `done` implémentation de `T-005` avec durcissement de la sélection DSP et nouveaux tests de robustesse
- `done` implémentation de `T-006` avec état audio live/mock explicite et messages runtime clarifiés
- `done` implémentation de `T-007` avec tests de séquences et documentation mise à jour
- `done` implémentation de `T-010` avec TUI ratatui/crossterm, rendu jauge 3 lignes et mode démo
- `done` implémentation de `T-011` avec TUI modernisée, badge de statut, footer métriques et documentation alignée

### 2026-03-31

- `done` implémentation de `T-012` avec refactor modulaire `theme/layout/widgets`
- `done` ajout des états UI `Searching` et `Unstable`
- `done` ajout d'un footer de télémétrie enrichi et d'un historique compact
- `done` ajout de micro-animations sobres sur les badges et labels décoratifs
- `done` merge de `feat/cyberpunk-tui` dans `dev`
- `done` déplacement de la documentation projet sous `docs/`
- `done` réalignement de `README`, `SPEC`, `ARCHITECTURE`, `CONTEXT` et `TESTING` sur l'état codé

### 2026-04-02

- `done` nettoyage `clippy` workspace avec `-D warnings`
- `done` regroupement des paramètres contextuels dans `dsp::audio` et `cli::tui::demo`
- `done` traçabilité documentaire du refactor via `ARCHITECTURE.md` et `DECISIONS.md`

## Prochaine action recommandée

- définir le périmètre de la V0.2 après validation terrain
