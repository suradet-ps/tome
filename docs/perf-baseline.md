# Performance Baseline — Tome

Measured during Phase 7 (Performance Budgets). **Budgets are set from real
numbers, never guessed** — everything below is measured, not claimed.

## Toolchain measured

- `rustc 1.97.1` stable, `--release` profile (`opt-level = "z"`, `lto`,
  `codegen-units = 1`, `panic = "abort"`, `strip`)
- `trunk 0.21.14`, `trunk build --release`
- **wasm-opt 131 active** — trunk invoked it with
  `-O --enable-bulk-memory --enable-nontrapping-float-to-int`
  (verified from trunk's verbose log; the bulk-memory flags are the reason
  wasm-opt works with the current Rust toolchain — see commit history for the
  incompatibility that forced it off earlier). **Verified directly:** without
  those flags, wasm-opt exits with status 1 on this toolchain's output — so
  the flags are load-bearing, not decoration. CI pins **binaryen version_131**
  from the official GitHub release (not the distro package: Ubuntu's binaryen
  108 is too old and rejects this toolchain's wasm), so the release build is
  wasm-opt'd with the same version everywhere.
- Host: Windows desktop (Ryzen-class CPU, wired network) — *not* a mid-tier
  device; browser-side figures below are the absolute ceiling, not a target.

## Bundle (deterministic, CI-verifiable)

| Asset | Raw bytes | gzip | brotli |
|---|---|---|---|
| `tome_*.wasm` | 2,204,786 | 851,212 (38.6%) | 651,390 (29.5%) |
| bootstrap `tome_*.js` | 67,947 | — | — |
| `main-*.css` | 36,948 | — | — |
| other css (`variables`, `reset`, `highlight`) | 6,384 | — | — |
| **total dist (12 files)** | **2,325,674** | **~871,757** | — |

> The .wasm is ~96% of total bytes. Every byte saved there is the whole game.

## Browser metrics — pending measurement

Not yet recorded: cold first-paint, dashboard/book load, and note-save
latency **on a mid-tier device with a throttled network** (the target
envelope Tome is meant for). The numbers below are where they will be
recorded once that run is scripted; nothing here is claimed until it is
measured.

| Metric | Value |
|---|---|
| First meaningful paint (cold, no service worker) | — (pending) |
| Dashboard load (empty library, warm Supabase) | — (pending) |
| Book view load (20 chapters, 10 notes) | — (pending) |
| Note save round trip (WASM→Supabase→WASM) | — (pending) |

Method once scripted: DevTools throttling (Slow 4G), mid-tier profile, three
cold runs each, medians recorded.

## To reproduce

```bash
trunk build --release
# sizes:
#   wasm  dist/tome-*_bg.wasm
#   gzip  gzip -9 -c dist/tome-*_bg.wasm | wc -c
#   brotli brotli -q 11 -c dist/tome-*_bg.wasm | wc -c
```

## Budgets (CI-enforced, see .github/workflows/ci.yml)

| Budget | Value | Basis |
|---|---|---|
| `.wasm` raw ceiling | 2,400,000 bytes (~9% headroom) | baseline 2,204,786 |
| `.wasm` gzip ceiling | 950,000 bytes (~12% headroom) | baseline 851,212 |
| First paint / interactive | targets set after the mid-tier run | pending |

Rationale for headroom: a regression under ~10% is noise; anything above it
is a real feature or a real mistake and should be a conscious, explained
change.
