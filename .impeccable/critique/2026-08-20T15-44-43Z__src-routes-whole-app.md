---
target: whole app (Library + Settings + shell)
total_score: 30
max_score: 40
na_heuristics: 
p0_count: 2
p1_count: 2
timestamp: 2026-08-20T15-44-43Z
slug: src-routes-whole-app
---
## Overall Impression

Liberarme's shell is disciplined and restrained — the "Evidence Locker" tokens (near-black surfaces, one rationed gradient, small-caps metadata labels, tabular numerals) are applied consistently across the Library and Settings pages, and the responsive collapse at 860px/640px is genuinely well built. But the product doesn't yet *feel* authored: the copy voice is generic SaaS ("No games scanned yet", "Ready to scan"), the one moment a first-timer needs the most help — pasting a Steam Web API key — offers none, and the signature gradient itself has a real WCAG contrast failure on both places it's allowed to appear (the brand mark and the primary CTA). The biggest opportunity: make the "case file" metaphor show up in copy and status, not just in color tokens, while fixing the accessibility bug on the one component the design system asks users to trust most.

## What's Working

- **Responsive shell collapse** ([+layout.svelte](src/routes/+layout.svelte#L187-L222)): the two-column shell degrades gracefully to a single column below 860px (sidebar becomes a horizontal bar, tagline/status hide) and the stat grid drops to one column below 640px — real, tested responsive behavior, not just a media query that happens to exist.
- **Autosaving API key with inline confirmation** ([settings/+page.svelte](src/routes/settings/+page.svelte#L182-L207)): blur-triggered autosave plus an animated checkmark (respecting `prefers-reduced-motion`) removes a manual save step for the highest-friction field on the page.
- **Fixed-layout data table** ([+page.svelte](src/routes/+page.svelte#L245-L260)): `table-layout: fixed` with per-column width percentages keeps install paths truncating with an ellipsis instead of breaking the grid — the one place "Evidence Locker" precision is fully realized.

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|-------|-----------|
| 1 | Visibility of System Status | 3/4 | Loading/saving states are clear, but the sidebar status card ("Ready to scan") never updates with a scan count or timestamp after a scan completes. |
| 2 | Match Between System and Real World | 3/4 | Steam-specific vocabulary (App ID, SteamID64) is accurate, but none of the "case file"/evidence-locker brand voice from DESIGN.md shows up in actual copy. |
| 3 | User Control and Freedom | 4/4 | No locked flows; scan is on-demand, credentials are editable, helper is optional. |
| 4 | Consistency and Standards | 4/4 | Tokens, spacing, and component patterns are applied consistently between the two pages. |
| 5 | Error Prevention | 3/4 | Field-level validation blocks bad input, but there's no unsaved-changes warning when leaving Settings mid-edit. |
| 6 | Recognition Rather Than Recall | 2/4 | Users must already know what a Steam Web API key or SteamID64 is — no link out, no inline explainer, no example. |
| 7 | Flexibility and Efficiency of Use | 3/4 | Autosave and the vanity-URL helper are good accelerators; no Enter-to-submit on the helper input, no shortcuts anywhere. |
| 8 | Aesthetic and Minimalist Design | 3/4 | Layout is clean and uncluttered, but see the confirmed contrast failure below — the one reserved accent is failing accessibility contrast where it's used most. |
| 9 | Error Recovery | 3/4 | Field errors are specific and actionable; system-level failures surface as a raw `TypeError` string instead of a translated message (confirmed live, see below). |
| 10 | Help and Documentation | 2/4 | Field hints exist but no contextual help, external link, or first-run guidance for the credential step that gates the entire app. |
| **Total** | | **30/40** | **Acceptable — real foundation, but onboarding and one accessibility bug hold it back from "Good"** |

## Design Specificity Verdict

**LLM assessment**: The system is better than a generic admin dashboard — the gradient is rationed, the table is domain-specific (App ID, install path, size), and tonal-only elevation is followed correctly. But it reads as a data-management utility, not an "evidence locker." The static "Ready to scan" status card, the instructional empty state ("No games scanned yet. Click Scan Library to begin."), and stat tiles that show app configuration (Source: Steam, Mode: Local scan) rather than library insight all miss the chance to make the case-file metaphor legible in words, not just tokens.

**Deterministic scan**: `detect.mjs --json src/routes` exited 2 with one finding: `overused-font` on [+layout.svelte#L69](src/routes/+layout.svelte#L69) (Inter). This is a **known false positive relative to the brief** — DESIGN.md commits explicitly to "one typeface for everything" as a deliberate anti-decoration choice, so this should be left alone rather than "fixed."

**Visual overlays**: Live injection of `detect.js` succeeded on both pages (overlays were rendered in the browser tab, not just logged) and surfaced two real findings the LLM review missed entirely:
- **`low-contrast`**: paper-white text (`#e5e7eb`) on the gradient measures **2.9:1** on the brand-mark "L" ([+layout.svelte](src/routes/+layout.svelte#L120-L127)), and white CTA text measures **3.6:1** on the Scan Library button ([+page.svelte](src/routes/+page.svelte#L149-L156)) — both fail the 4.5:1 WCAG AA minimum against the indigo end of the gradient (`#5b7cff`).
- **`ai-color-palette`**: flagged the indigo/violet gradient itself as a generic AI-SaaS pattern. Given DESIGN.md's explicit "One Gradient Rule" (the gradient is a rationed signature, used in exactly two places), **this is the brief overriding a saturated-pattern warning** — treat it as intentional, not a defect. The contrast failure above is the real, actionable issue; the color choice itself is not.

## Priority Issues

**[P0] No in-app guidance for the Steam API key / SteamID64 step**
- **Why it matters**: Settings is the mandatory first stop for every new user, and it currently assumes the visitor already knows what a Steam Web API key is and how to obtain one. A user who doesn't (the majority of "Jordan"-type first-timers) has no path forward except leaving the app to search externally.
- **Fix**: Add an inline explainer next to the API Key label (what it's for, that it's stored locally, a direct link to `steamcommunity.com/dev/apikey`) and a one-line explainer for SteamID64 before the helper input.
- **Suggested command**: `/impeccable onboard`

**[P0] Contrast failure on the brand mark and primary CTA**
- **Why it matters**: The detector confirmed live in-browser that both places the reserved gradient is allowed to appear — the sidebar "L" logomark and the Scan Library button — fail WCAG AA contrast (2.9:1 and 3.6:1 against a 4.5:1 minimum). This is a real accessibility bug on the two components DESIGN.md singles out as the system's signature, not a stylistic nitpick.
- **Fix**: Either darken the gradient's lighter stop, or move text/logo weight fully onto `--pure-white` with a heavier weight/larger size sufficient to qualify for the 3:1 large-text threshold, or add a subtle dark text-shadow/scrim. Re-test contrast against both gradient stops, not just the darker one.
- **Suggested command**: `/impeccable audit`

**[P1] System errors surface as raw exception text**
- **Why it matters**: Confirmed live on both pages — with the backend unreachable, users see `Failed to load settings: TypeError: Cannot read properties of undefined (reading 'invoke')` verbatim in the UI. In production this will read on any real Steam API failure (bad key, offline, rate-limited), making the app look broken rather than reporting a recoverable problem.
- **Fix**: Catch and translate invoke failures into plain-language messages ("Couldn't reach Steam. Check your connection and try again.") and reserve raw error text for a collapsed "details" disclosure, not the primary error line.
- **Suggested command**: `/impeccable clarify`

**[P1] Copy and status don't carry the "Evidence Locker" identity**
- **Why it matters**: The status card ("Ready to scan"), empty state ("No games scanned yet."), and stat tiles (Source: Steam, Mode: Local scan) are procedural rather than investigative, so the one differentiated idea in DESIGN.md's Overview never reaches the user-facing product. Right now the tokens carry 100% of the brand and the words carry none of it.
- **Fix**: Status card should report scan recency once data exists ("247 games audited · last scan 2h ago"); empty state and stat tiles should speak in case-file language and surface library insight (total install size, launcher-independent count once available) instead of app configuration.
- **Suggested command**: `/impeccable clarify`

**[P2] SteamID64 helper has weak affordance and no keyboard path**
- **Why it matters**: The helper input requires a separate button click with no Enter-to-submit, and its visual weight matches the two required fields around it, blurring which fields are mandatory vs. assistive.
- **Fix**: Add an Enter-key handler on the helper input, and reduce its visual weight (smaller label, lighter border) relative to the two Required fields.
- **Suggested command**: `/impeccable layout`

## Persona Red Flags

**Jordan (First-Timer)**: Reads "Configure Steam credentials (stored locally)" and immediately has to decide whether pasting an API key is safe — nothing on the page reassures her before she acts. She doesn't know what a Steam Web API key or SteamID64 is; the field hints ("Used to fetch your library from Steam's Web API") assume she already does. If she gets past that, saving produces "Settings saved locally." with no confirmation the credentials actually work, and after scanning she's shown a 127-row table of App IDs and install paths with no hint of what to do with that data next.

**Alex (Power User)**: Gets through Settings quickly (already knows what an API key is) but hits friction the app doesn't need to create: no Enter-to-submit on the vanity-URL helper, no keyboard shortcut to trigger a scan, and no sort/filter on the games table once it's populated with hundreds of rows.

## Minor Observations

- Activity column stacks "updated:" and "synced:" on two unlabeled lines in the same cell ([+page.svelte](src/routes/+page.svelte#L172-L175)) — easy to misread which date is which at a glance.
- Sidebar tagline "audit/backup your games" is plain-utility language; doesn't reinforce the case-file identity DESIGN.md describes.
- The `overused-font` detector finding on Inter is a false positive relative to the documented brief — no action needed.
- No unsaved-changes warning when navigating away from a partially-filled Settings form.

## Questions to Consider

- "If Liberarme is an Evidence Locker, why does the empty state read like a blank spreadsheet?" What would the case-file metaphor look like in the actual first-scan and empty-state copy?
- "Who gates entry to your product — a form, or a person?" Does the Settings credential step need a short first-run explainer flow, or just better inline hints?
- "What's the one thing a user should feel after their first successful scan?" Right now the answer is nothing; what would change that?
