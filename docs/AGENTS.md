# AGENTS

## Objectif

Ce dépôt implémente une V0.1 d'accordeur guitare simple en Rust. `SPEC.md` est la source de vérité produit et architecture.

## Règles de contribution

- préserver `SPEC.md`, sauf correction minimale indispensable
- garder le code modulaire, lisible et testable
- privilégier Rust stable et peu de dépendances
- éviter la sur-ingénierie
- séparer clairement musique, DSP, orchestration et CLI

## Conventions de structure

- `core` contient les types métier et conversions musicales
- `dsp` contient le pipeline de détection de pitch
- `cli` contient l'interface console, l'orchestration applicative et la TUI

## Priorités techniques

1. logique musicale correcte et testée
2. pipeline DSP lisible avant optimisation
3. préserver la qualité de la TUI et préparer proprement la V0.2

