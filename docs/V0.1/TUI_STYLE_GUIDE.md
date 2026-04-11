# TUI_STYLE_GUIDE.md

## Objectif

Définir une direction claire pour implémenter un **TUI d'accordeur guitare en Rust avec ratatui** dans une esthétique **Cyberpunk 2077**, tout en conservant une excellente lisibilité en terminal.

Ce document sert de référence de développement pour un agent comme Codex.

---

## Vision produit

L'interface doit évoquer :

- une console futuriste tactique
- un système audio temps réel
- une interface néon à fort contraste
- une sensation de scanning / monitoring / calibration
- un rendu agressif, technique, lisible

Le résultat ne doit pas ressembler à un dashboard business classique.

---

## Principes visuels

### Identité

Le style recherché repose sur :

- **fond sombre profond**
- **couleurs néon très ciblées**
- **titres techniques en uppercase**
- **cadres anguleux et compartimentés**
- **signalétique système** (`LOCK`, `INPUT`, `TARGET`, `OFFSET`, `NOISE`)
- **densité visuelle maîtrisée**

### À éviter

- les bordures trop douces
- les blocs trop uniformes
- les écrans surchargés de texte
- les couleurs nombreuses sans signification
- les animations constantes qui nuisent à la lecture

---

## Palette recommandée

Utiliser peu de couleurs, mais avec une sémantique stricte.

### Couleurs principales

- **Jaune néon** : élément principal, note détectée, cible
- **Cyan électrique** : données temps réel, fréquence, télémétrie
- **Magenta / rouge néon** : alerte, surtension, erreur, désaccord fort
- **Vert discret** : accord juste, lock stable
- **Gris sombre** : panneaux secondaires, séparateurs
- **Noir profond** : arrière-plan

### Mapping terminal / ratatui

Exemples de mapping :

- `Color::Yellow` ou `Color::Rgb(255, 211, 0)`
- `Color::Cyan` ou `Color::Rgb(0, 229, 255)`
- `Color::Magenta` ou `Color::Rgb(255, 42, 109)`
- `Color::Green`
- `Color::DarkGray`
- `Color::Black`

### Sémantique

- **note détectée** → jaune
- **fréquence / cents / télémétrie live** → cyan
- **écart trop haut ou trop bas hors tolérance** → magenta/rouge
- **accord correct / centré** → vert
- **cadres secondaires** → gris foncé

---

## Layout recommandé

Le layout principal doit être découpé en **3 zones verticales**.

### 1. Header technique

Contient :

- nom de l'application
- état du flux audio
- profil d'accordage
- fréquence d'échantillonnage
- niveau de confiance éventuel

Exemple d'intention :

```text
╭─ ACCORD//R ───────── LIVE INPUT ───────── SIGNAL: LOCKED ─╮
│ DEVICE: JACK IN   SR: 48kHz   PROFILE: STANDARD E   MODE: TUNE │
╰────────────────────────────────────────────────────────────╯
```

### 2. Zone centrale héro

C'est la zone prioritaire. Elle doit contenir :

- la **note détectée** en très grand
- la **fréquence mesurée**
- l'**écart en cents**
- la **jauge horizontale principale**
- un **statut textuel** explicite

Exemple d'intention :

```text
┌──────────────────────── PITCH//CORE ────────────────────────┐
│                                                             │
│                           [ E2 ]                            │
│                        82.3 Hz   -12c                       │
│                                                             │
│ LOW ────────────────┬────────◎────────┬────────────── HIGH │
│                     ███                                     │
│                                                             │
│ STATUS: STRING UNDER-TENSION                               │
└─────────────────────────────────────────────────────────────┘
```

### 3. Footer / télémétrie

Contient :

- cible active
- stabilité de la mesure
- niveau de bruit
- confiance de détection
- mini historique compact

Exemple :

```text
╭─ TELEMETRY ────────────────────────────────────────────────╮
│ TARGET: E2   STABILITY: 74%   NOISE: LOW   CONF: HIGH     │
│ HISTORY: ▂▃▄▅▆▅▄▃▂                                         │
╰────────────────────────────────────────────────────────────╯
```

---

## Widgets à implémenter

### HeaderWidget

Responsabilité :

- afficher l'identité de l'app
- afficher l'état du signal
- afficher quelques métadonnées techniques

Style :

- bordure simple ou mixte
- titres en uppercase
- signal `LOCKED` en vert ou cyan
- `NO SIGNAL` en magenta/rouge

---

### PitchHeroWidget

Responsabilité :

- afficher la note détectée au centre
- afficher la fréquence et l'offset
- afficher l'état global d'accordage

Style :

- composant dominant visuellement
- note au centre avec fort contraste
- statut clairement lisible

Règle UX :

- la note est l'élément visuel n°1
- l'offset est l'élément visuel n°2
- la fréquence vient ensuite

---

### HorizontalTuneGauge

Responsabilité :

- représenter visuellement l'écart à la cible
- indiquer si la corde est trop basse ou trop haute
- rendre immédiatement visible la zone idéale au centre

### Comportement

- centre = accord juste
- gauche = trop bas
- droite = trop haut
- curseur ou barre verticale mobile
- marque centrale plus forte que le reste
- utiliser une tolérance configurable (par exemple ±3 cents)

### Recommandations d'affichage

- graduations discrètes
- point central `◎`, `◆`, `│` ou équivalent
- indicateur coloré selon la distance

Exemple :

```text
LOW ────────────────┬────────◎────────┬────────────── HIGH
                    ███
```

Ou variante 3 lignes inspirée de ton idée initiale :

```text
{-----|-------------}
{         E2        }
{-----|-------------}
```

Version cyberpunk recommandée : conserver le concept, mais remplacer les délimitations trop simples par quelque chose de plus technique.

---

### TelemetryWidget

Responsabilité :

- afficher des mesures secondaires
- ne jamais détourner l'attention de la note principale

Contenu possible :

- `TARGET`
- `OFFSET`
- `CONF`
- `NOISE`
- `STABILITY`
- `FRAME TIME`

Style :

- faible hauteur
- lecture rapide
- cyan et gris majoritaires

---

### HistorySparklineWidget

Responsabilité :

- montrer la stabilité récente de la détection
- rendre visible le comportement du signal

Implémentation suggérée :

- sparkline ratatui
- historique glissant des offsets récents
- option de lissage visuel

---

## États visuels

L'interface doit gérer des états explicites.

### 1. No signal

Affichage :

- header en alerte
- zone centrale sans note ou avec placeholder
- message : `NO INPUT SIGNAL`
- jauge inactive ou masquée

### 2. Searching

Affichage :

- note absente ou instable
- message : `SCANNING INPUT`
- animation légère possible sur le header

### 3. Detected but out of tune

Affichage :

- note en jaune
- offset cyan ou magenta selon intensité
- jauge active
- message explicite du type :
  - `TENSION TOO LOW`
  - `TENSION TOO HIGH`

### 4. In tune

Affichage :

- note stable
- jauge centrée
- offset vert
- message : `LOCKED // IN TUNE`

### 5. Noisy / unstable

Affichage :

- warning discret
- confiance réduite
- télémétrie bruit mise en avant

---

## Animation et effets

Le terminal impose de rester sobre.

### Effets recommandés

#### Clignotement léger

Utilisable sur :

- `LOCKED`
- alertes
- statut critique

Fréquence faible. Pas de clignotement permanent sur toute l'UI.

#### Glitch subtil

Très occasionnel :

- sur 1 frame rare, léger changement d'un caractère décoratif
- micro décalage visuel d'un label non critique

Ne jamais appliquer le glitch à la donnée importante en continu.

#### Scan / pulse

Possible sur :

- séparateurs
- panneau central
- label `LIVE`

Le but est d'évoquer un système actif, sans gêner la lecture.

---

## Typographie terminale

Le terminal ne permet pas une vraie typographie, mais on peut travailler l'impression visuelle.

### Règles

- titres courts en uppercase
- labels techniques compacts
- espacement cohérent
- éviter les phrases longues dans l'UI

### Exemples de labels

- `PITCH//CORE`
- `SIGNAL TRACE`
- `INPUT LOCK`
- `TARGET PROFILE`
- `AUDIO FEED`
- `SYSTEM STATUS`

---

## Bordures et composition

### Recommandation

Mixer plusieurs niveaux d'intensité :

- cadre principal plus affirmé pour la zone centrale
- cadres secondaires plus sobres
- séparateurs fréquents mais fins

### Caractères utiles

- `─ │ ┌ ┐ └ ┘`
- `═ ║ ╔ ╗ ╚ ╝`
- `├ ┤ ┬ ┴ ┼`
- `╭ ╮ ╰ ╯`

### Règle de composition

Ne pas encadrer chaque petit élément indépendamment. Préférer :

- quelques panneaux forts
- des sous-zones internes
- une hiérarchie claire

---

## Hiérarchie d'information

Ordre de priorité à l'écran :

1. note détectée
2. écart à la cible
3. statut d'accordage
4. fréquence mesurée
5. télémétrie secondaire
6. historique

Si l'écran devient trop dense, retirer d'abord les éléments secondaires.

---

## Architecture Rust / ratatui recommandée

Structure suggérée :

```text
src/
  ui/
    mod.rs
    theme.rs
    layout.rs
    widgets/
      header.rs
      pitch_hero.rs
      tune_gauge.rs
      telemetry.rs
      history.rs
  app/
    state.rs
    model.rs
```

### theme.rs

Contient :

- palette
- styles réutilisables
- helpers `status_style()`, `note_style()`, `gauge_style()`

### layout.rs

Contient :

- découpage principal avec `Layout`
- responsive terminal si taille réduite

### widgets/*

Chaque widget doit :

- recevoir un state minimal
- être pur autant que possible
- ne pas contenir de logique métier audio

### state.rs

Expose les données nécessaires au rendu :

- note courante
- fréquence
- offset cents
- stabilité
- bruit
- lock signal
- target string

---

## Responsive minimal

Le TUI doit rester utilisable sur plusieurs tailles de terminal.

### Terminal large

Afficher :

- header complet
- hero complet
- footer complet
- historique

### Terminal moyen

Réduire :

- densité du header
- largeur de la jauge
- nombre de métriques secondaires

### Terminal petit

Afficher seulement :

- note
- offset
- mini jauge
- statut

La lisibilité reste prioritaire sur le style.

---

## Accessibilité terminale

Même en style cyberpunk, il faut préserver :

- contraste fort
- lisibilité sans dépendre uniquement des couleurs
- signaux textuels explicites

Exemple : ne pas signaler uniquement l'état par couleur. Ajouter toujours un texte :

- `IN TUNE`
- `TOO LOW`
- `TOO HIGH`
- `NO SIGNAL`

---

## Critères d'acceptation

Le travail sera considéré réussi si :

1. la note détectée est immédiatement identifiable
2. l'utilisateur comprend en moins d'une seconde s'il doit tendre ou détendre la corde
3. l'interface évoque une console futuriste sans devenir illisible
4. les couleurs ont une sémantique stable
5. la jauge centrale est lisible et expressive
6. les états `no signal`, `searching`, `out of tune`, `in tune` sont distincts
7. le rendu reste propre sur une taille de terminal standard
8. le code UI reste modulaire et maintenable

---

## Exemple de mockup cible

```text
╔══════════════════════════ ACCORD//R v0.1 ══════════════════════════╗
║ CHANNEL_01 :: GUITAR INPUT :: PROFILE [STANDARD E] :: LINK ACTIVE ║
╠═════════════════════════════════════════════════════════════════════╣
║                                                                     ║
║                          DETECTED NOTE                              ║
║                              E2                                     ║
║                                                                     ║
║                        FREQ 82.31 Hz                                ║
║                        OFFSET -12 c                                 ║
║                                                                     ║
║   < LOW ────────────────┬──────── ◎ ┬──────────────── HIGH >        ║
║                         ███                                          ║
║                                                                     ║
║                 DIAGNOSTIC :: STRING UNDER-TENSION                  ║
║                                                                     ║
╠═════════════════════════════════════════════════════════════════════╣
║ TARGET E2   CONF 91%   NOISE 08%   STABILITY 76%   FRAME 2412      ║
╚═════════════════════════════════════════════════════════════════════╝
```

---

## Prompt Codex prêt à l'emploi

```md
Refactor the existing Rust ratatui TUI to adopt a strong Cyberpunk 2077-inspired visual identity while preserving excellent terminal readability.

Requirements:
- Keep the app focused on guitar tuning.
- Use a dark background with a strict neon color system:
  - yellow for the detected note / target emphasis
  - cyan for live metrics and telemetry
  - magenta/red for alerts and strong tuning error
  - green only for stable in-tune confirmation
  - dark gray for secondary borders and separators
- Build the screen in 3 vertical sections:
  1. technical header
  2. central hero pitch area
  3. compact telemetry footer
- The central hero area must contain:
  - the detected note as the most visually dominant element
  - measured frequency
  - cents offset
  - a horizontal tuning gauge with a very strong center mark
  - an explicit status message such as IN TUNE / TOO LOW / TOO HIGH / NO SIGNAL
- The UI should feel like a futuristic tactical audio console, not a generic dashboard.
- Use uppercase technical labels like PITCH//CORE, SIGNAL LOCK, TARGET, TELEMETRY.
- Add subtle animation hooks if architecture allows it:
  - soft blinking for lock / warning states
  - rare subtle glitch effect on decorative elements only
- Ensure the UI remains readable on smaller terminal sizes by degrading gracefully.
- Keep business logic separate from rendering logic.
- Introduce a reusable theme module for colors and text styles.
- Split UI into modular ratatui widgets/components.

Implementation guidance:
- Create or refactor modules such as:
  - ui/theme.rs
  - ui/layout.rs
  - ui/widgets/header.rs
  - ui/widgets/pitch_hero.rs
  - ui/widgets/tune_gauge.rs
  - ui/widgets/telemetry.rs
  - ui/widgets/history.rs
- Prefer clean rendering code and reusable style helpers.
- Avoid over-decorating every small area with heavy borders.
- Prioritize information hierarchy:
  1. note
  2. cents offset
  3. tuning direction/status
  4. frequency
  5. secondary telemetry

Acceptance criteria:
- The detected note is instantly visible.
- The user can understand whether to tighten or loosen the string in under one second.
- The interface clearly looks cyberpunk/futuristic.
- The UI remains readable and not noisy.
- The code is modular, maintainable, and easy to extend.

Also provide:
- a concise explanation of the widget hierarchy
- a summary of style decisions
- any new structs/enums introduced for UI state
```

---

## Recommandation finale

Commencer par implémenter d'abord :

1. `theme.rs`
2. `layout.rs`
3. `PitchHeroWidget`
4. `HorizontalTuneGauge`
5. `HeaderWidget`
6. `TelemetryWidget`

Puis ajouter seulement ensuite les micro-effets visuels.
