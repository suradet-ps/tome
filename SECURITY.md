# Security Policy

## Supported Versions

Tome is under active development on the `main` branch. Only the latest published release
and the `main` branch receive security fixes.

| Version | Supported |
|---------|-----------|
| `main`  | ✅ Yes    |

## Reporting a Vulnerability

If you discover a security vulnerability, **please report it privately** — do not open a
public GitHub issue.

- Use GitHub's private vulnerability reporting: open **Security → Report a
  vulnerability** on the repository.
- Or email the maintainer directly (see git history / `Cargo.toml` authors) with the
  subject `Tome security report`.

Please include:

- A clear description of the vulnerability and its impact.
- Steps to reproduce (or a proof-of-concept).
- Affected version(s) / commit(s).

We aim to acknowledge reports within **72 hours** and will keep you updated as we triage
and fix. Credit will be given in the release notes unless you prefer to remain anonymous.

## Scope & Known Protections

Tome is a client-side WASM app backed by Supabase. Security relies on several layers:

- **Row-Level Security (RLS):** every `reading_*` table enforces
  `auth.uid() = user_id`, so users can only read/write their own rows. The schema in
  `supabase-schema.sql` is the source of truth — verify RLS is enabled before trusting
  any new table.
- **XSS protection:** user markdown (notes/flashcards) is rendered via `pulldown-cmark`
  and sanitized with `ammonia` before injection. Never bypass the sanitizer or inject raw
  HTML.
- **Content-Security-Policy:** enforced via `vercel.json` headers. Keep it strict; avoid
  adding `unsafe-inline`/`unsafe-eval` or widening `connect-src` beyond Supabase.
- **No `unsafe` code:** the crate denies `unsafe_code` at the crate level.
- **Length caps & validation:** note content and titles are length-capped server-side
  (see schema constraints) and client-side.

## Out of Scope

- Vulnerabilities requiring physical access to a user's device.
- Social engineering or phishing attacks against users/maintainers.
- Issues in third-party dependencies — please report those upstream, but feel free to
  open an issue so we can track the upgrade.

## Dependency Updates

Security-relevant dependency updates are handled via `Cargo.toml` / `Cargo.lock`. To
check for known advisories locally:

```bash
cargo install cargo-audit
cargo audit
```
