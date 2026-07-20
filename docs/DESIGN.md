# Tome — Design System

Tome is a *quiet, personal* place to read technical books. The visual system
exists to serve one thing: sustained, calm, focused reading and note-taking,
often for long stretches, often at night. Every choice below is measured against
that — not against looking impressive, not against competing for attention.

> **The feeling.** A calm reading room. The default dark is a cool-neutral
> charcoal that lets the eyes rest; the amber accent is low-voltage, a lamp
> pointing at the page rather than lighting the whole room. The interface should
> recede so the words — the book's chapters, your own notes — are what you see.
> Nothing pulses, nothing shouts, nothing sells.

This is the single source of truth for Tome's tokens. The implementation lives in
`public/styles/variables.css` (token definitions) and is consumed everywhere else
through `var(--token)`. **Raw color literals (`#rrggbb`) are only allowed inside
`variables.css`.** Everywhere else — every other stylesheet and every Leptos
`view!` — must reference a token. This rule is enforced in CI.

---

## Design Principles

1. **Calm over voltage.** A reading tool is not a dashboard or a trading terminal.
   The accent is used sparingly, for a single primary action or the current focus,
   never to energize the whole page. Large fields of saturated color are avoided.
2. **Restful, never clinical.** The dark canvas is a cool-neutral charcoal —
   close to lamplight-grey on paper, not a cold blue-black, never pure `#000`.
   The light theme is a warm off-white, and a dedicated Sepia theme leans into
   paper directly. The amber accent stays low-saturation in every theme so it
   points without yanking the eye off the text.
3. **Text is the interface.** Body copy is comfortable to read for a long time:
   generous line height, restrained contrast (easy on the eyes at night, still
   AA-compliant), a humanist sans for prose and a calm mono for code.
4. **One accent, used with restraint.** A single warm amber carries the brand and
   primary actions. There is no competing secondary brand color. Status colors
   (success / danger / info / warning) exist only to communicate state, never to
   decorate.
5. **Flat and quiet.** Flat surface steps, hairline borders that read as gentle
   elevation rather than hard lines, soft shadows only where real depth helps
   (modals). No gradients, no glow, no atmospheric backdrops.
6. **Three themes, one system.** `dark` (default), `light`, and `sepia` are all
   pure token remaps under `[data-theme]`. No component knows which theme is
   active; no theme introduces a new hex outside `variables.css`.

---

## Color

Tome's palette is warm, muted, and low-contrast-by-choice (within AA). It is the
opposite of a trading platform's high-voltage yellow-on-black.

### Accent (the one brand color)

- **Amber** (`--color-primary`): a warm, slightly muted amber — the color of
  lamplight, not of a warning sign. Carries the primary CTA, the wordmark, the
  active/focused element. Used *scarcely* on the dark canvas, for emphasis only.
- **Amber Active** (`--color-primary-active`): the pressed/hover variant, a touch
  deeper and warmer.
- **Amber Disabled** (`--color-primary-disabled`): a desaturated, dim amber for
  disabled primary actions on the dark canvas.
- **On Primary** (`--color-on-primary`): the near-black ink that sits on an amber
  button — warm dark, never pure black.

### Surface (dark, the default)

The dark theme is the home Tome was designed in. Surfaces step up in neutral
grey, not warmth — a cool charcoal that keeps dilated pupils from amplifying
bright elements.

- **Canvas** (`--color-canvas`): the page floor. A deep cool-neutral charcoal —
  a hint of grey, never a cold blue-black, never pure `#000`.
- **Surface Card** (`--color-surface-card`): elevated cards, panels, the editor
  chrome — one grey step up from the canvas.
- **Surface Elevated** (`--color-surface-elevated`): nested surfaces, hovered
  rows, popovers — one further step up.

### Text

Restrained contrast, comfortable for long night reading, verified AA.

- **Body** (`--color-body`): default running text — a soft off-white, not pure
  white, so a wall of prose doesn't glare under dilated pupils.
- **On Dark** (`--color-on-dark`): the highest-contrast text, for headings that
  need to lead.
- **Muted** (`--color-muted`): captions, metadata, chapter counts, placeholder
  text — legible but quiet.
- **Muted Strong** (`--color-muted-strong`): a second tier of muted for labels
  that need slightly more presence than a caption.

### Hairlines

- **Hairline** (`--color-hairline`): the 1px border tone. It reads as a gentle
  surface step, not an ink line — deliberately close to the elevated surface tone.

### Status (state only, never decoration)

- **Success** (`--color-success`): a calm, muted green — a completed chapter, a
  saved note. Not a neon "profit" green.
- **Danger** (`--color-danger`): a warm, muted red — destructive actions, errors.
  Not an alarm red.
- **Info** (`--color-info`): a soft blue for informational notices and the focus
  ring base.
- **Warning** (`--color-warning`): a warm ochre for cautions — harmonizes with
  the amber accent rather than fighting it.

### Light & Sepia themes

Both are token remaps only (`[data-theme="light"]`, `[data-theme="sepia"]`):

- **Light**: a warm off-white canvas (not stark `#fff`), warm dark ink for text,
  the same amber accent, the same status semantics.
- **Sepia**: a paper-cream canvas and a soft brown-black ink — a reading mode that
  leans fully into "words on warm paper" for the longest reading sessions.

---

## Typography

Reading comfort first. A humanist sans for prose, a calm mono for code, and a
tabular face for numbers so counts and timers stay steady.

### Families

- **Sans / Display** (`--font-sans`, `--font-display`): **Inter** with a native
  system fallback. Used for all prose, headings, labels, and UI. Inter is chosen
  for its excellent legibility at small sizes and long-reading comfort — it does
  not draw attention to itself, which is the point.
- **Number** (`--font-number`): a tabular-friendly face (**IBM Plex Sans** →
  JetBrains Mono fallback) for chapter counts, progress figures, timers, and
  cards-due — so numbers align and don't jitter as they update.
- **Mono** (`--font-mono`): **JetBrains Mono** for code blocks and inline code in
  notes. Calm, even color, comfortable for reading code inside prose.

All fonts are bundled/offline — **no CDN, no external font fetch** (matches the
CSP and the offline-first goal).

### Scale

A compact, reading-oriented scale. Body sits at a comfortable 14–15px; headings
step up gently rather than dramatically — this is a reading tool, not a landing
page.

| Token | Size | Use |
|---|---|---|
| `--text-xs` | 12px | captions, meta labels |
| `--text-sm` | 13px | secondary text, footnotes |
| `--text-base` | 14px | default UI text |
| `--text-md` | 15px | comfortable body / note prose |
| `--text-lg` | 18px | sub-section titles |
| `--text-xl` | 22px | section titles |
| `--text-2xl` | 28px | page titles |

### Weights & line height

- Weights: `--weight-normal` 400 (body), `--weight-medium` 500 (labels),
  `--weight-semibold` 600 (titles/buttons), `--weight-bold` 700 (strong emphasis).
  Display weight stays restrained — no 700 hero shouting.
- Line height: `--leading-tight` 1.1 (large titles), `--leading-normal` 1.4 (UI),
  `--leading-relaxed` 1.6 (**body prose and notes** — long reading wants air).

---

## Layout

### Spacing (4px base)

`--space-xxs` 4 · `--space-xs` 8 · `--space-sm` 12 · `--space-md` 16 ·
`--space-lg` 24 · `--space-xl` 32 · `--space-xxl` 48.

Reading surfaces breathe: card interiors use `--space-lg`, page bands use
`--space-xl`/`--space-xxl`. Density is for lists (chapters, cards); prose is never
crammed.

### Container & measure

- **Container** (`--container-default`): the app's max content width.
- **Reading measure**: note and preview content is width-capped for a comfortable
  line length (~60–75 characters) rather than stretching edge to edge — a core
  reading-comfort control (see the Roadmap's Accessibility phase).

### Radius

Small-to-medium, soft but not bubbly: `--radius-sm` 4 · `--radius-md` 6 (buttons)
· `--radius-lg` 8 (inputs, cards) · `--radius-xl` 12 (elevated containers) ·
`--radius-pill` (pills) · `--radius-full` (avatars/dots).

### Elevation & motion

- **Shadows**: `--shadow-focus` (the focus ring, info-blue based) and
  `--shadow-panel` (soft modal lift). Nothing else casts a shadow — flatness is
  the default.
- **Transitions**: `--transition-fast` 120ms · `--transition-base` 200ms ·
  `--transition-slow` 300ms, all `ease`. Every transition is gated by
  `prefers-reduced-motion` — a calm tool respects a user who wants no motion.

---

## Syntax Highlighting (code in notes)

Code appears inside notes, so its palette must sit *inside* Tome's warm system,
not import VS Code's cool defaults. The highlight tokens
(`--code-*` in `variables.css`, consumed by `highlight.css`) are a warm,
low-contrast set: muted rose/amber for keywords and strings, calm green for
numbers and additions, soft blue-grey for names and variables, and `--color-muted`
for comments. The goal is code that reads as part of the note, quietly, not a
neon block that pulls the eye away from the prose around it.

---

## Accessibility (non-negotiable for a reading tool)

- **Contrast**: every text/background pairing meets WCAG AA in all three themes.
  Restrained contrast is a *choice within AA*, never an excuse to drop below it.
- **Focus**: a visible focus ring (`--shadow-focus`) on every interactive element;
  never removed, only restyled.
- **Motion**: all animation honors `prefers-reduced-motion`.
- **Semantics**: semantic HTML + ARIA; icon-only controls carry `aria-label`.

---

## What this system is not

- Not high-voltage. There is no single dominant saturated color doing "brand
  voltage" across the page.
- Not a trading/finance aesthetic. No up/down green-red price semantics, no dense
  data-table-first layout, no tabular-number hero counters competing with charts.
- Not attention-seeking. No gradients, glows, animated accents, or marketing
  hero energy. Tome is meant to be lived in quietly, one chapter at a time.
