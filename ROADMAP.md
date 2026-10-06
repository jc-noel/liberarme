# Liberarme Roadmap

## v1 Goal

Ship a preservation-first public release that helps a user answer:

> Which games I own can survive without their original launcher or an internet connection, which copies can I preserve safely, and which launcher-dependent games have known legitimate DRM-free alternatives?

Umbrella tracking issue: #65 — **Liberarme v1 — Preservation First Public Release**

## Foundation — Complete

- Steam Library Scanner (#1)
- Steam Ownership Discovery (#8)
- Ownership + installation data coexist safely (#12)
- Unified owned/installed library display (#13)

The older Library Reconciliation plan (#14–#19) is closed as superseded by the implementation above.

## v1 Build Order

### 1. Evidence System (#55)

Build the evidence layer before making stronger preservation claims.

- Evidence schema and query service (#56, #57)
- PCGamingWiki integration (#59)
- Evidence panel in the UI (#58)

### 2. Preservation Assessment (#25)

Turn evidence into conservative, explainable statuses.

V1 statuses:
- Unknown
- Needs Verification
- Likely Launcher Independent
- Launcher Required
- Locally Verified Independent

Launcher independence and full offline functionality are separate facts when the evidence allows that distinction.

### 3. Local Verification (#50)

Guide the user through testing an installed game with the relevant launcher closed.

- Record executable and observed process information
- Ask the user whether the game reached a usable state
- Store verification history
- Feed the result back into evidence and assessment

Liberarme observes behavior only. It does not patch, inject, crack, or bypass DRM.

### 4. Archive System (#45)

For locally verified launcher-independent games:

- Copy or compress the game files
- Generate `manifest.json`
- Generate human-readable restore notes
- Generate and validate SHA-256 checksums
- Preserve verification context and known limitations

### 5. DRM-free Alternatives (#40)

For launcher-dependent games, show known legitimate DRM-free editions when there is a trustworthy match.

V1 starts with a focused GOG path (#42). Ambiguous matches stay unconfirmed. Steam ownership never implies ownership on another storefront.

### 6. Release Hardening (#60)

Before public distribution:

- Reproducible desktop build and installer (#61)
- Safe forward database migrations (#62)
- Onboarding, privacy, safety, and limitations docs (#63)
- Representative-game release test matrix (#64)

## v1 Definition of Done

A user on a clean supported machine can:

1. Install Liberarme.
2. Scan installed Steam games and optionally sync owned games.
3. Inspect an evidence-backed preservation status.
4. Run a guided local verification on an installed game.
5. Archive a locally verified launcher-independent game and validate its checksums.
6. See a known legitimate DRM-free alternative for a launcher-dependent game when a trusted match exists.
7. Restart or upgrade the app without losing library or preservation data.
8. Understand the limits of every claim and action shown by the app.

## Explicitly Post-v1

- Community reports, moderation, and confidence voting (#30–#34)
- Price Intelligence (#35–#39)
- Additional storefront matchers beyond the first focused GOG path (#43)
- Multi-store library ingestion
- Cloud sync

## Product Boundary

Liberarme may detect, document, verify, copy, archive, and point to legitimate alternatives.

It does not provide cracking, DRM bypass, entitlement transfer, patched executables, key sharing, or instructions for circumventing access controls.
