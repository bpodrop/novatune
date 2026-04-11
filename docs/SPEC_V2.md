# SPEC — Accordeur guitare simple en Rust — V0.2 définition

## 1. Contexte

L’objectif du projet est de développer une application simple d’accordeur de guitare en Rust.

La V0.1 a déjà livré une chaîne complète utilisable :

- capture micro temps réel
- détection de pitch monophonique pour guitare
- conversion fréquence -> note -> cents
- publication stabilisée via `TunerEngine`
- TUI terminal cyberpunk modulaire pour le mode live et un mode démo

La V0.2 étend cette base avec deux modes produit explicites :

- `Chromatic`
- `Preset`

Le projet cible en priorité une guitare 6 cordes, avec support de plusieurs accordages prédéfinis.

---

## 2. Objectif fonctionnel V0.2

Implémenter un pipeline temps réel capable de :

- capturer un signal audio mono depuis le micro
- détecter la fréquence fondamentale d’une corde jouée seule
- convertir cette fréquence en :
  - note la plus proche
  - fréquence cible
  - écart en cents
  - indicateur de confiance
- fournir un résultat suffisamment stable pour alimenter une UI d’accordeur
- exposer ce résultat via une CLI simple et une TUI terminal lisible
- supporter un mode `Chromatic` libre
- supporter un mode `Preset` guidé par un accordage sélectionné
- identifier en mode `Preset` la corde la plus probable et la note cible associée

---

## 3. Choix technique validé

La méthode de détection retenue est :

- **analyse de périodicité temporelle**
- basée sur **autocorrélation normalisée de type MPM / NSDF**
- avec :
  - **filtrage des faux pics**
  - **interpolation parabolique**
  - **stabilisation temporelle**

Ce choix remplace une FFT simple comme méthode principale.

---

## 4. Périmètre V0.2

### Inclus

- capture audio micro temps réel
- traitement mono
- bufferisation
- prétraitement léger
- détection de pitch par MPM/NSDF
- conversion pitch -> note musicale
- sortie exploitable par UI/CLI
- commande `strings`
- commande `analyze` live avec modes `Chromatic` et `Preset`
- sélection initiale du mode et du preset à l'entrée de `analyze` via la CLI
- calibration utilisateur A4 via argument CLI `--a4` (analyse + affichage strings)
- changement de mode et de preset à la volée dans la TUI live
- commande `tune <NOTE>` live mono-cible conservée comme mode expert/technique
- commande `demo`
- splashscreen terminal de démarrage, quittable avec `q` ou `Esc`
- TUI terminal modulaire avec états explicites `NoSignal`, `Searching`, `Unstable`, `TooLow`, `TooHigh`, `InTune`
- sélection d’accordages prédéfinis côté produit
- identification de corde en mode `Preset` par comparaison fréquentielle avec les cibles du preset sélectionné
- variante web mobile-first PWA (`apps/tuner-web-d-tunner`) utilisant le bridge wasm partagé
- tests unitaires sur la couche musicale et DSP de base

### Exclus

- reconnaissance polyphonique
- gestion de plusieurs instruments
- visualisation avancée
- suppression de bruit avancée / ML
- pitch shifting
- enregistrement audio
- édition utilisateur de presets dans cette itération

---

## 5. Contraintes produit

- application simple
- faible latence perçue
- comportement stable
- fonctionnement correct sur une guitare seule, une corde à la fois
- robustesse suffisante pour les fréquences entre **60 Hz et 400 Hz**
- code modulaire, lisible, testable
- en mode `Preset`, éviter les faux changements de corde d’une frame à l’autre

---

## 6. Exigences de précision

Le système doit viser :

- détection correcte de la bonne note sur les 6 cordes standard
- précision d’affichage suffisamment fine pour un accordeur
- erreur cible :
  - idéalement < ±3 cents sur signal propre stabilisé
  - tolérance acceptable V1 : < ±5 cents sur cas simples
- faible oscillation visuelle de la note affichée
- faible oscillation de l’identification de corde en mode `Preset`

Ce sont des objectifs de qualité, pas des garanties absolues sur tout environnement micro.

---

## 7. Hypothèses d’entrée

- source : microphone
- signal : monophonique
- une seule corde jouée à la fois
- environnement raisonnablement calme
- fréquence d’échantillonnage :
  - préférée : `44_100 Hz`
  - acceptable : `48_000 Hz`
- format interne : `f32`

---

## 7.b Contrat logique de sortie

Le moteur doit publier un résultat logique unifié, exploitable par la CLI et la TUI, contenant au minimum :

- `mode`
- `measured_frequency_hz`
- `confidence`
- `detected_note`
- `display_cents`
- `target_note` optionnelle
- `target_frequency_hz` optionnelle
- `preset_id` optionnel
- `string_index` optionnel
- `string_label` optionnel
- `ui_state`

Règles d'interprétation :

- en mode `Chromatic`, `target_note` et `target_frequency_hz` correspondent à la note tempérée la plus proche
- en mode `Preset`, `target_note` et `target_frequency_hz` correspondent à la corde retenue dans le preset actif
- en mode `Preset`, `string_index` désigne la position de corde dans le preset, de `0` à `5` du grave vers l'aigu
- `string_label` est une représentation UI dérivée, par exemple `6E2`, `5A2`, `4D3`, `3G3`, `2B3`, `1E4`

Ce contrat évite que chaque couche UI réinterprète différemment la notion de cible.

---

## 8. Pipeline DSP V0.2

### 8.1 Capture audio

Le module audio doit :

- capturer le flux micro
- convertir vers mono si nécessaire
- pousser les échantillons dans un buffer circulaire
- ne pas faire de calcul DSP lourd dans le callback audio

### 8.2 Fenêtre d’analyse

Utiliser une fenêtre glissante de taille :

- valeur par défaut : `4096` samples
- option future : `2048` pour réduire la latence

Justification :

- 4096 améliore la robustesse sur la corde grave E2 et reste acceptable pour `Drop D`

À `44_100 Hz`, 4096 samples ≈ 92.9 ms.

### 8.3 Pas d’analyse

Le recalcul ne doit pas attendre un buffer entièrement nouveau.

Utiliser un hop size :

- recommandé : `512` ou `1024` samples

Objectif :

- mise à jour plus fluide sans coût excessif

### 8.4 Prétraitement

À appliquer avant la détection :

1. suppression DC offset
2. normalisation légère ou contrôle d’amplitude
3. rejet des buffers trop faibles
4. optionnel si simple : fenêtre Hann avant analyse

Ne pas sur-complexifier le filtrage en V0.2.

### 8.5 Plage de recherche

Limiter la recherche de période à la plage :

- `f_min = 60.0 Hz`
- `f_max = 400.0 Hz`

En nombre de samples :

```text
τ_min = floor(fs / f_max)
τ_max = ceil(fs / f_min)
```

Exemple à 44.1 kHz :

- tau_min ≈ 110
- tau_max ≈ 630

---

## 9. Algorithme de détection

### 9.1 Méthode retenue

Implémenter une version simple et lisible de :

- **MPM / McLeod Pitch Method**
- utilisant **NSDF** (Normalized Square Difference Function)

### 9.2 Étapes algorithmiques

Pour chaque fenêtre :

1. calculer la fonction NSDF pour `tau in [tau_min, tau_max]`
2. détecter les maxima locaux positifs
3. rejeter les pics faibles ou non plausibles
4. sélectionner le pic fondamental le plus crédible
5. raffiner sa position par interpolation parabolique
6. convertir `tau` raffiné en fréquence
7. calculer un score de confiance
8. lisser le résultat dans le temps

### 9.3 Filtrage des faux pics

Le filtrage doit au minimum inclure :

- ne considérer que les pics locaux positifs
- seuil minimum sur la hauteur du pic NSDF
- exclusion des pics hors plage fréquentielle
- rejet si le buffer a une énergie insuffisante
- pondération par cohérence temporelle

---

## 10. Modes produit

### 10.1 Mode `Chromatic`

Le mode `Chromatic` doit :

- afficher la note tempérée la plus proche de la fréquence détectée
- afficher la fréquence mesurée
- afficher l’écart en cents par rapport à la note tempérée la plus proche
- afficher un état visuel `TooLow`, `InTune` ou `TooHigh`

### 10.2 Mode `Preset`

Le mode `Preset` doit :

- permettre de sélectionner un accordage prédéfini
- comparer la fréquence détectée aux fréquences cibles des 6 cordes du preset actif
- identifier la corde la plus probable
- afficher la corde identifiée
- afficher la note cible associée
- afficher l’écart en cents par rapport à cette cible
- afficher un état visuel `TooLow`, `InTune` ou `TooHigh`

Le mode `Preset` ne doit pas déduire la corde uniquement à partir du nom de note arrondi. Il doit s’appuyer sur la distance fréquentielle à chaque cible du preset.

### 10.3 Sélection de mode et de preset

La V0.2 doit rester simple côté interaction :

- le mode actif peut être choisi au lancement de `analyze`
- si le mode est `Preset`, le preset actif peut être choisi au lancement
- pendant `analyze`, le mode courant peut être changé à la volée depuis la TUI
- pendant `analyze`, si le mode courant est `Preset`, le preset actif peut être changé à la volée depuis la TUI
- la TUI doit toujours afficher explicitement le mode courant et, si applicable, le preset actif

Interaction minimale requise dans la TUI live :

- `m` bascule entre `Chromatic` et `Preset`
- `p` ouvre le sélecteur de presets
- `↑` et `↓` naviguent dans la liste des presets
- `Enter` applique le preset sélectionné
- `Esc` ferme le sélecteur sans appliquer de changement
- `h` ou `?` affiche ou masque l'aide clavier

Lors d'un passage de `Chromatic` à `Preset`, le système doit réutiliser le dernier preset actif si disponible, sinon le preset par défaut.

### 10.4 Modèle d'état UI

Les états UI doivent être interprétés comme suit :

- `NoSignal` : niveau trop faible ou absence de détection exploitable
- `Searching` : signal présent mais cible ou note encore non stabilisée
- `Unstable` : détection plausible mais confiance ou stabilité insuffisante pour guider l'accordage
- `TooLow` : cible identifiée et écart négatif hors zone d'accordage
- `TooHigh` : cible identifiée et écart positif hors zone d'accordage
- `InTune` : cible identifiée et écart dans la tolérance d'accordage

Règle de priorité :

1. `NoSignal`
2. `Searching`
3. `Unstable`
4. `TooLow` / `TooHigh` / `InTune`

Les états d'accordage ne doivent être affichés que si la détection est suffisamment stable pour exposer une cible fiable.

---

## 11. Accordages prédéfinis V0.2

### E Standard

- corde 6 : E2 = 82.41 Hz
- corde 5 : A2 = 110.00 Hz
- corde 4 : D3 = 146.83 Hz
- corde 3 : G3 = 196.00 Hz
- corde 2 : B3 = 246.94 Hz
- corde 1 : E4 = 329.63 Hz

### Drop D

- corde 6 : D2 = 73.42 Hz
- corde 5 : A2 = 110.00 Hz
- corde 4 : D3 = 146.83 Hz
- corde 3 : G3 = 196.00 Hz
- corde 2 : B3 = 246.94 Hz
- corde 1 : E4 = 329.63 Hz

### Eb Standard

- corde 6 : Eb2 = 77.78 Hz
- corde 5 : Ab2 = 103.83 Hz
- corde 4 : Db3 = 138.59 Hz
- corde 3 : Gb3 = 185.00 Hz
- corde 2 : Bb3 = 233.08 Hz
- corde 1 : Eb4 = 311.13 Hz

### D Standard

- corde 6 : D2 = 73.42 Hz
- corde 5 : G2 = 98.00 Hz
- corde 4 : C3 = 130.81 Hz
- corde 3 : F3 = 174.61 Hz
- corde 2 : A3 = 220.00 Hz
- corde 1 : D4 = 293.66 Hz

### Drop C

- corde 6 : C2 = 65.41 Hz
- corde 5 : G2 = 98.00 Hz
- corde 4 : C3 = 130.81 Hz
- corde 3 : F3 = 174.61 Hz
- corde 2 : A3 = 220.00 Hz
- corde 1 : D4 = 293.66 Hz

### Open G

- corde 6 : D2 = 73.42 Hz
- corde 5 : G2 = 98.00 Hz
- corde 4 : D3 = 146.83 Hz
- corde 3 : G3 = 196.00 Hz
- corde 2 : B3 = 246.94 Hz
- corde 1 : D4 = 293.66 Hz

### Open D

- corde 6 : D2 = 73.42 Hz
- corde 5 : A2 = 110.00 Hz
- corde 4 : D3 = 146.83 Hz
- corde 3 : F#3 = 185.00 Hz
- corde 2 : A3 = 220.00 Hz
- corde 1 : D4 = 293.66 Hz

### DADGAD

- corde 6 : D2 = 73.42 Hz
- corde 5 : A2 = 110.00 Hz
- corde 4 : D3 = 146.83 Hz
- corde 3 : G3 = 196.00 Hz
- corde 2 : A3 = 220.00 Hz
- corde 1 : D4 = 293.66 Hz

Convention produit :

- l'ordre des cordes est toujours stocké et affiché du grave vers l'aigu
- l'index `0` correspond à la corde 6
- l'index `5` correspond à la corde 1

Le système doit proposer au minimum les presets suivants :

- `E Standard`
- `Drop D`
- `Eb Standard`
- `D Standard`
- `Drop C`
- `Open G`
- `Open D`
- `DADGAD`

---

## 12. Matching corde/preset

Pour chaque fréquence détectée en mode `Preset` :

1. calculer l’écart en cents entre la fréquence mesurée et chaque fréquence cible du preset
2. choisir la corde dont la distance absolue est minimale
3. rejeter la correspondance si la distance dépasse une fenêtre configurable, par exemple `100 cents`
4. lisser l’identification de corde sur plusieurs frames

Formule :

```text
cents = 1200 * log2(measured / target)
```

L’identification de corde doit être considérée comme instable si la clarté, la stabilité ou la distance à la cible ne sont pas suffisantes.

---

## 13. Exigences fonctionnelles V0.2

### FR-MODE-01
Le système doit supporter deux modes d’accordage :
- `Chromatic`
- `Preset`

### FR-MODE-02
En mode `Chromatic`, le système doit afficher la note tempérée la plus proche de la fréquence détectée.

### FR-MODE-03
En mode `Preset`, le système doit permettre de sélectionner un accordage prédéfini.

### FR-MODE-04
En mode `Preset`, le système doit comparer la fréquence détectée aux fréquences cibles des cordes du preset sélectionné.

### FR-MODE-05
En mode `Preset`, le système doit identifier la corde la plus probable à partir de la distance fréquentielle minimale.

### FR-MODE-06
En mode `Preset`, le système doit afficher :
- la corde identifiée
- la note cible
- l’écart en cents par rapport à cette cible

### FR-MODE-07
Le système doit rejeter ou marquer comme instables les détections dont la confiance est insuffisante.

### FR-MODE-08
Le système doit lisser l’identification de corde sur plusieurs frames pour éviter le flicker visuel.

### FR-MODE-09
Le système doit conserver `tune <NOTE>` comme mode mono-cible expert tant que l’API CLI n’est pas simplifiée davantage.

### FR-MODE-10
Le système doit permettre de choisir le mode initial de `analyze` au lancement via la CLI.

### FR-MODE-11
Le système doit publier un résultat logique unifié contenant au minimum la fréquence mesurée, la confiance, la cible affichée, l'écart en cents, l'état UI et, en mode `Preset`, l'identifiant du preset et la corde retenue.

### FR-MODE-12
Le système doit représenter les cordes de preset avec un index stable du grave vers l'aigu afin d'éviter les ambiguïtés d'affichage entre moteur, CLI et TUI.

### FR-MODE-13
Le système ne doit afficher `TooLow`, `TooHigh` ou `InTune` que lorsque la cible affichée est jugée suffisamment fiable.

### FR-MODE-14
Pendant `analyze`, le système doit permettre de basculer à la volée entre les modes `Chromatic` et `Preset` sans relancer l'application.

### FR-MODE-15
Pendant `analyze`, lorsque le mode courant est `Preset`, le système doit permettre de changer le preset actif à la volée depuis la TUI.

### FR-MODE-16
Le système doit exposer au minimum les interactions clavier suivantes dans la TUI live :
- `m` pour basculer entre `Chromatic` et `Preset`
- `p` pour ouvrir le sélecteur de presets
- `↑` et `↓` pour naviguer dans la liste des presets
- `Enter` pour appliquer le preset sélectionné
- `Esc` pour fermer le sélecteur ou quitter selon le contexte

### FR-MODE-17
La TUI live doit exposer une aide clavier affichable à la demande via `h` ou `?`.

### FR-MODE-18
L'application doit afficher un splashscreen de démarrage avant l'entrée dans le mode principal, avec terminaison automatique et interruption possible via `q` ou `Esc`.

### FR-MODE-19
La CLI doit permettre de configurer la calibration A4 via `--a4 <hz>` sur `analyze` et `strings`.

### FR-MODE-20
La couche DSP plateforme (`native`, `web`, `embedded`) doit exposer des APIs de configuration de preset et de calibration pour aligner le mapping note/cents entre clients.

---

## 14. Critères d’acceptation V0.2

- l’utilisateur peut lancer `analyze` avec un mode initial `Chromatic` ou `Preset`
- l’utilisateur peut passer visuellement de `Chromatic` à `Preset` pendant l'exécution
- l’utilisateur peut ouvrir un sélecteur de presets et appliquer un preset pendant l'exécution
- les presets `E Standard`, `Drop D`, `Eb Standard`, `D Standard`, `Drop C`, `Open G`, `Open D` et `DADGAD` sont disponibles
- en mode `Preset`, une corde jouée seule est associée à la cible la plus proche du preset actif
- l’utilisateur peut régler A4 via `--a4` sur les commandes `strings` et `analyze`
- la TUI affiche clairement le mode courant, le preset actif, la cible et la corde probable
- une aide clavier est consultable en cours d'exécution
- le démarrage affiche un splashscreen terminal quittable
- l’affichage reste stable et lisible sans flicker excessif sur des entrées simples

---

## 15. Limites connues

- la détection reste monophonique
- les résultats dépendent de la qualité du micro et du bruit ambiant
- la robustesse doit encore être validée sur matériel réel et plusieurs guitares
- les presets utilisateur ne font pas partie de cette itération
