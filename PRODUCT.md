# Product

<!-- impeccable:product-schema 1 -->

## Platform

Desktop (Tauri)

## Users

Primarily PC gamers auditing their own Steam library to find out which owned games would still run without Steam or another launcher (the dominant case). Secondary: digital preservationists/archivists concerned with long-term game survival independent of any platform, and users looking to avoid future re-purchases by identifying DRM-free alternatives to games they already own.

## Product Purpose

A local-first desktop app that inventories a user's Steam library, gathers preservation evidence, helps the user verify launcher-independence locally, archives verified copies, and surfaces known legitimate DRM-free alternatives for launcher-dependent games. Success means a user can understand what survives without Steam, why Liberarme believes that, and what can be preserved safely.

## Positioning

Liberarme is not a general game launcher or metadata manager. Its differentiator is the preservation loop: inventory → evidence → preservation assessment → local verification → archive, with legitimate DRM-free alternatives for games that remain launcher-dependent. Existing community/reference sources should be integrated rather than recreated where practical.

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
3. Steam first, depth over breadth: nail one storefront's evidence/verification/archive loop before expanding to other library sources.
4. Evidence over certainty: preservation assessments must be explainable, conservative, and revisable when stronger evidence appears.
