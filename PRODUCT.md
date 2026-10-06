# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primarily PC gamers auditing their own Steam library to find out which owned games would still run without Steam or another launcher (the dominant case). Secondary: digital preservationists/archivists concerned with long-term game survival independent of any platform, and users looking to avoid future re-purchases by identifying DRM-free alternatives to games they already own.

## Product Purpose

A local-first desktop app that scans a user's installed Steam library, classifies which games are launcher-independent, surfaces DRM-free alternatives, and preserves verified games through archiving. Success means a user can see, at a glance, which of their owned games survive without Steam.

## Positioning

Mostly (per user): automated local scan + verification — Liberarme scans the actual installed library and verifies launcher-independence directly, rather than requiring the user to manually cross-reference sites like PCGamingWiki or GOG DRM-free lists. Secondarily, it goes further than detection by archiving verified DRM-free copies, and — longer-term — matching to alternative stores and tracking pricing (see Roadmap: Classification Engine, Evidence System, Local Verification, Archive System, Store Matching, Price Intelligence).

## Operating Context

- Desktop app (Tauri + Rust backend, SvelteKit + TypeScript frontend, SQLite for local storage, Bun for package/scripts).
- Current v1 milestone: Evidence System → Preservation Assessment → Local Verification → Archive System → DRM-free Alternatives → Release Hardening.
- Steam integration requires the user's own Steam Web API key and SteamID64, entered and stored locally in Settings; a vanity-URL resolver helps users find their SteamID64.
- The v1 release sequence is tracked in `ROADMAP.md` and umbrella issue #65.

## Capabilities and Constraints

- Local-first, no cloud sync by default: scan results, credentials, and archives stay on-device unless the user later opts in to something else.
- Steam-only for now; other storefronts/libraries are explicitly deferred (Icebox: Multi-Store Libraries).
- No legal/DRM-circumvention claims or functionality: the product detects, classifies, and verifies launcher-independence and archives DRM-free copies — it must never claim or imply it cracks, bypasses, or circumvents DRM.
- Project is Pre-Alpha. Steam scanning, ownership sync, local persistence, and the unified owned/installed library are implemented. Evidence, preservation assessment, local verification, archiving, DRM-free alternatives, and release hardening remain for v1.

## Brand Commitments

- Name: "Liberarme". Existing mark: single-letter "L" logomark in the current UI. Tagline used in the current UI: "audit/backup your games".

## Evidence on Hand

None beyond the shipped Steam scanner UI itself (game list with title, app ID, install path/size, last-updated/synced timestamps). No testimonials, case studies, or third-party data exist yet — future work must not fabricate any.

## Product Principles

1. Local-first and user-owned: no data leaves the device unless the user explicitly chooses otherwise.
2. Detection and preservation, not circumvention: the product's legitimacy rests on verifying and archiving, never bypassing DRM.
3. Steam first, depth over breadth: nail one storefront's classification/evidence/archive loop before expanding to other stores.
4. Evidence-backed classification: launcher-independence claims should be verifiable, not guessed.
