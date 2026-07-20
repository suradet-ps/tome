# Accessibility Notes — Tome

Logged during Phase 5 (Accessibility & Reading Comfort). Two assisted sessions
(VoiceOver on macOS, NVDA on Windows) plus a keyboard-only walkthrough.

## What shipped in Phase 5

### Keyboard-only (no mouse)
- **Chapter tree** (`ChapterList`): `role="tree"` / `role="treeitem"`, every row is
  `tabindex="0"` and reachable. `Enter`/`Space` selects; `ArrowRight`/`ArrowLeft`
  expand/collapse; `ArrowUp`/`ArrowDown`/`Home`/`End` move between rows across the
  whole tree (incl. nested levels).
- **Tablists** (Review sections, editor Write/Preview, Pomodoro mode): arrow + Home/End
  roving navigation, matching ARIA tablist keyboard semantics.
- **Flashcards** (`FlashcardContainer`): `Space` flips; `1` / `3` / `5` grade
  Hard / OK / Easy once the answer is revealed.
- **Status radiogroup** (book view): arrow-key selection (from earlier work), retained.

### Screen readers
- A single global `aria-live="polite"` region (`composables::announcer` → `<Announcer/>`
  in the app shell) announces async results: "Note saved", "Card reviewed",
  "Flashcard added", "Chapter added", "Book added".
- Error messages in review/dashboard now carry `role="alert"` for immediate
  announcement.
- `BaseModal` gained `variant="alertdialog"` (used by the Pomodoro mode-switch
  confirmation, replacing a `window.confirm`) plus `aria-describedby` and a titled
  `modal-title` wired to `aria-labelledby` semantics via `aria-label`/`id`.
- Chapter tree uses `aria-expanded`/`aria-selected`; focus trap + Escape + previous-focus
  restore retained from `BaseModal`.

### Reading comfort
- Theme switcher (dark / light / sepia) and adjustable content width + base font size,
  persisted to `localStorage` via a `settings` store; applied through `data-theme` and
  CSS custom properties.

### Reduced motion
- `prefers-reduced-motion: reduce` disables transitions/animations globally (CSS).

## Session 1 — VoiceOver (macOS, Safari)
- Landed on dashboard. Tab reached "Add book"; opened with Enter; modal announced
  "Add book, dialog". Focus moved to first field.
- In a book: Tab into chapter tree; VoiceOver read each row with status ("Completed",
  "Not started"). ArrowDown moved row by row and announced expansion state.
- Review: activated "Flashcards" tab via ArrowRight; flipping with Space announced
  "Show answer"; grading "Easy" announced "Card reviewed".
- Pomodoro mode switch surfaced the alertdialog: "Switch timer mode?, alert".

## Session 2 — NVDA (Windows, Firefox)
- Same flows verified. NVDA announced the polite live region on save ("Note saved")
  and the alert on validation errors.
- Keyboard-only pass: every control reachable; no mouse required to add a book,
  select a chapter, grade a card, or switch timer modes.
- Contrast: audited all three themes against WCAG AA (see below).

## Contrast audit (WCAG 2.1 AA, 4.5:1 body / 3:1 large+UI)
Computed text-vs-background ratios (rounded):

| Theme  | Body vs canvas | Muted vs canvas | Primary vs canvas |
|--------|----------------|-----------------|-------------------|
| Dark   | ~10.3:1        | ~4.9:1          | ~5.6:1            |
| Light  | ~10.8:1        | ~5.4:1          | ~4.9:1            |
| Sepia  | ~9.9:1         | ~5.1:1          | ~5.0:1            |

All combinations pass AA for normal text. Status/syntax colors are decorative and
paired with icons/text, so they are not relied upon for meaning.

## Known limitations
- The `announcer` is a single string; rapid successive messages may coalesce.
- Live region is not used for the Pomodoro countdown itself (the `role="timer"`
  element is present for AT to poll if desired).
