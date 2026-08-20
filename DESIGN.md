---
name: Liberarme
description: A local-first tool that audits your Steam library for launcher-independence and preserves what survives.
colors:
  archive-bg: "#0b1118"
  archive-raised: "#0d141c"
  archive-card: "#111a25"
  archive-well: "#0f1722"
  case-file-indigo: "#5b7cff"
  case-file-violet: "#7c3aed"
  paper-white: "#e5e7eb"
  slate-ash: "#94a3b8"
  slate-ash-bright: "#cbd5e1"
  border-hairline: "#1f2937"
  border-line: "#243244"
  border-line-soft: "#2b3a4d"
  border-dashed: "#2a3b4f"
  table-line: "#223041"
  row-hover: "#16212e"
  nav-hover: "#182230"
  nav-active: "#1b2736"
  success: "#34d399"
  danger: "#f87171"
  danger-soft: "#fca5a5"
  case-file-indigo-soft: "#93a9ff"
  pure-white: "#ffffff"
typography:
  display:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "2.2rem"
    fontWeight: 700
    lineHeight: "normal"
  brand-name:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "1.8rem"
    fontWeight: 700
    lineHeight: 1.1
  title:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 700
  body:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.45
  subtitle:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "1.05rem"
    fontWeight: 400
  label:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "0.82rem"
    fontWeight: 700
    letterSpacing: "0.08em"
  field-label:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "0.95rem"
    fontWeight: 600
  badge:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "0.78rem"
    fontWeight: 600
  caption:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "0.9rem"
    fontWeight: 400
  hint:
    fontFamily: "Inter, system-ui, -apple-system, Segoe UI, Roboto, sans-serif"
    fontSize: "0.85rem"
    fontWeight: 400
rounded:
  sm: "10px"
  md: "12px"
  lg: "14px"
  pill: "999px"
spacing:
  xs: "8px"
  sm: "12px"
  md: "16px"
  lg: "20px"
  xl: "28px"
components:
  button-primary:
    backgroundColor: "linear-gradient(135deg, {colors.case-file-indigo}, {colors.case-file-violet})"
    textColor: "{colors.pure-white}"
    rounded: "{rounded.lg}"
    padding: "11px 18px"
  button-ghost:
    backgroundColor: "rgba(91, 124, 255, 0.2)"
    textColor: "{colors.case-file-indigo-soft}"
    rounded: "{rounded.sm}"
    padding: "10px 16px"
  card:
    backgroundColor: "{colors.archive-card}"
    rounded: "{rounded.lg}"
    padding: "{spacing.md}"
  input:
    backgroundColor: "{colors.archive-well}"
    textColor: "{colors.paper-white}"
    rounded: "{rounded.sm}"
    padding: "10px 12px"
---

# Design System: Liberarme

## Overview

**Creative North Star: "The Evidence Locker"**

Liberarme reads like case files, not a dashboard: dark, procedural, unshowy. Every screen exists to help someone verify a fact about a game they own — installed or not, launcher-dependent or free — so the interface stays out of the way of the record it's showing. The voice is utilitarian and technical: dense tables, small-caps labels, tabular numerals, no hand-holding copy. Depth comes from tonal layering (background darker than card darker... well, actually lighter as surfaces get closer to the user — see Elevation), not shadows or ornament.

The one deliberate accent — a narrow indigo-to-violet gradient — is reserved for the small number of primary actions and brand marks that deserve attention. It must stay rare. The system explicitly rejects reading as a generic SaaS dashboard: the gradient is a signature, not wallpaper, and it should never spread to card backgrounds, section headers, or decorative fills.

**Key Characteristics:**
- Near-black, low-chroma backgrounds with a single warm-blue/violet accent used sparingly
- Zero box-shadows; hierarchy comes from background luminance steps and 1px borders
- Small-caps, letter-spaced labels for metadata (stat headings, table headers, status)
- Tabular numerals and monospace-adjacent alignment for scannable data (App ID, size)
- Flat, precise, instrumented components — nothing decorative beyond the reserved gradient

## Colors

Near-black neutrals dominate; a single indigo-to-violet gradient is the only saturated color in the system, and it's rationed. Every color below is a CSS custom property defined once (`src/routes/+layout.svelte`'s `:global(:root)` block) and consumed via `var(--token-name)` everywhere else in the app — no component re-declares a literal hex.

### Primary
- **Case File Indigo** (`#5b7cff`): the gradient's cool end. Used as the solid accent for input focus borders, badge/pill text, and ghost-button text/background (at low opacity). This is the color to reach for when something needs to read as "linked to the accent" without the full gradient.
- **Case File Indigo, Soft** (`#93a9ff`): a lighter tint of Case File Indigo, reserved specifically for text sitting on a low-opacity indigo fill — ghost-button labels and the "installed" badge. Never used as a background or border.
- **Case File Violet** (`#7c3aed`): the gradient's warm end. Appears only as the second stop of `button-primary` and the brand mark — never alone.

### Neutral
- **Deep Archive** (`#0b1118`): page background — the darkest surface, the "case room" floor.
- **Deep Archive, Raised** (`#0d141c`): sidebar background, one step up from the page.
- **Deep Archive, Card** (`#111a25`): cards, panels, stat tiles, the settings form — the surface content sits on.
- **Deep Archive, Well** (`#0f1722`): input fields — sunk one step below card level, like a slot cut into the surface.
- **Paper White** (`#e5e7eb`): primary body text.
- **Pure White** (`#ffffff`): the one true white in the system — reserved for text sitting directly on the accent gradient or the active-nav fill, where Paper White's slight tint would look muddy against a saturated background.
- **Slate Ash, Bright** (`#cbd5e1`): nav links, form labels — secondary text that still needs to read clearly.
- **Slate Ash** (`#94a3b8`): muted text — subtitles, hints, table header labels, timestamps.
- **Border Hairline** (`#1f2937`): sidebar/content divider.
- **Border Line** (`#243244`): card and form-card borders.
- **Border Line, Soft** (`#2b3a4d`): input borders.
- **Border, Dashed** (`#2a3b4f`): the empty-state panel border — dashed signals "nothing recorded yet."
- **Table Line** (`#223041`): table row dividers.
- **Row Hover** (`#16212e`) / **Nav Hover** (`#182230`) / **Nav Active** (`#1b2736`): interactive-state fills, each one tonal step above its resting surface.

### Semantic
- **Success** (`#34d399`): confirmed saves, resolved actions (e.g. "Settings saved locally.").
- **Danger** (`#f87171`): errors and blocking validation.
- **Danger, Soft** (`#fca5a5`): inline field-level error text — one step quieter than a page-level error.

### Named Rules
**The One Gradient Rule.** The indigo-to-violet gradient appears only on the primary call-to-action button and the brand mark. It never becomes a card background, a header treatment, or a decorative fill — that is the generic-SaaS look this system explicitly rejects.

## Typography

**Display/Body Font:** Inter (with system-ui, -apple-system, Segoe UI, Roboto, sans-serif)

**Character:** One typeface for everything. Hierarchy is built with size, weight, and letter-spacing rather than a font pairing — it reads like a report generator, not a marketing site.

### Hierarchy
- **Display** (700, 2.2rem): page titles ("Library", "Settings").
- **Brand Name** (700, 1.8rem, line-height 1.1): the sidebar brand wordmark ("Liberarme") — a non-heading role so it never competes with the page's own `<h1>`.
- **Title** (700, 1.25rem): stat-tile values — the one place a number gets emphasis.
- **Body** (400, 1rem, 1.45 line-height): running text, table cells.
- **Subtitle** (400, 1.05rem): the page-header subtitle beneath Display titles and the sidebar tagline.
- **Label** (700, 0.82rem, uppercase, 0.08em letter-spacing): stat-tile headings, table column headers, the sidebar status-card heading — always paired with Slate Ash.
- **Field Label** (600, 0.95rem, Slate Ash Bright): form field labels in Settings — a distinct, slightly heavier role from the small-caps Label above.
- **Badge** (600, 0.78rem): the "installed" status pill and the inline "Required" field marker — both sit on a tinted fill or float beside a label rather than standing alone.
- **Caption** (400, 0.9rem): the ghost/helper button label and inline field-error text.
- **Hint** (400, 0.85rem): persistent field hint text (format/purpose notes, "Saving...") shown below an input before any error state.

### Named Rules
**The Small-Caps Metadata Rule.** Anything describing data about data (a stat's name, a table column, a status heading) is uppercase, letter-spaced, and muted. Anything that *is* the data (a game title, a stat value) stays normal case and full-contrast.

## Layout

Two-column app shell: a fixed 280px sidebar and a fluid content column (`grid-template-columns: 280px 1fr`), full viewport height. Sidebar padding is 20px 18px; content padding is 28px 30px. Page headers use a `space-between` flex row (title block left, primary action right).

Spacing runs a loose 4px-rooted scale in practice: 8px (tight gaps: stat grid, nav stack, form fields), 12px (helper-row gaps, card padding), 16px (card padding, section margins), 20px (sidebar rhythm), 28px (content padding). Stat tiles sit in a 3-column grid (`repeat(3, minmax(0, 1fr))`). Tables are `table-layout: fixed` with per-column width percentages so long install paths truncate instead of reflowing the grid.

**Narrow-window behavior:** below 860px the two-column shell collapses to one column and the sidebar becomes a compact horizontal bar (brand + nav only; the tagline and status card hide to make room). Below 640px the page header stacks vertically and the stat grid drops to one column. The games table keeps its fixed-percentage columns but gains a 640px minimum width and a scrollable panel, so it scrolls horizontally rather than compressing its data past legibility.

## Elevation & Depth

No box-shadows anywhere in the system — depth is entirely tonal. Each surface that sits "above" another gets a lighter background: page (`archive-bg`) → sidebar (`archive-raised`) → cards (`archive-card`) → inputs, which invert the direction and sink one step darker (`archive-well`) to read as a slot rather than a raised surface. Borders (1px, `border-line` family) reinforce the edges tonal contrast alone wouldn't carry at these low-chroma values. The one exception is the input focus ring: a soft `rgba(91, 124, 255, 0.2)` glow via `box-shadow`, reserved for the single most important "you are here" state in a form.

### Named Rules
**The Flat-By-Default Rule.** Surfaces are flat at rest. The only shadow in the system is the focus-ring glow, and it exists to mark input focus — not to decorate a card or button.

## Shapes

Rounding scales with a component's weight: 10px for inputs, nav links, and the ghost/helper button; 12px for the save button; 14px for cards, panels, and the primary scan button; full pill (999px) for the status badge. Borders are always 1px solid, except the empty-state panel, which uses 1px dashed to visually mark "nothing recorded" rather than "content present."

## Components

### Buttons
- **Shape:** 14px radius for primary actions (Scan Library), 12px for the settings Save button, 10px for the ghost/helper button.
- **Primary:** white text on the reserved `linear-gradient(135deg, case-file-indigo, case-file-violet)`, bold weight, no border, 11px 18px padding.
- **Ghost/Helper:** `rgba(91, 124, 255, 0.2)` background, `#93a9ff` text, no border, 10px 16px padding; hover raises the fill to `rgba(91, 124, 255, 0.3)`.
- **Disabled:** 0.6–0.7 opacity, `cursor: not-allowed`, no other treatment change.

### Chips / Badges
- **Style:** `rgba(91, 124, 255, 0.16)` background, `#93a9ff` text, full pill radius, 3px 10px padding, bold 0.78rem — used for the "installed" status.

### Cards / Containers
- **Corner Style:** 14px radius.
- **Background:** `archive-card` (`#111a25`); the empty-state panel keeps the same background but swaps to a dashed border.
- **Shadow Strategy:** none — see Elevation.
- **Border:** 1px solid `border-line`, or 1px dashed `border-dashed` for empty states.
- **Internal Padding:** 12–16px.

### Inputs / Fields
- **Style:** `archive-well` background, 1px solid `border-line-soft`, 10px radius, 10px 12px padding.
- **Focus:** border shifts to `case-file-indigo`, plus the one reserved `box-shadow` glow (`0 0 0 2px rgba(91, 124, 255, 0.2)`).
- **Error:** border shifts to `danger` (`#f87171`) via `[aria-invalid="true"]`; helper text below in `danger-soft`.
- **Disabled:** 0.6 opacity, `cursor: not-allowed`.
- **Inline success:** a small animated checkmark (`#34d399`) fades/scales in next to the field once a value is confirmed saved.

### Navigation
- **Style:** sidebar link list, 10px radius, 10px 12px padding, `slate-ash-bright` text at rest.
- **Hover:** `nav-hover` (`#182230`) background.
- **Active:** `nav-active` (`#1b2736`) background, white text, 600 weight.

### Tables (signature component)
Dense, fixed-layout data table for the scanned game list: uppercase small-caps headers, tabular-numeral right-aligned numeric columns, per-column width percentages so the install-path cell truncates with an ellipsis instead of breaking the grid, and a subtle `row-hover` fill on `tr:hover`. This is the closest thing Liberarme has to a signature component — it's where "Evidence Locker" is most literal.

## Do's and Don'ts

### Do:
- **Do** keep the indigo-to-violet gradient to one primary action and the brand mark per screen (The One Gradient Rule).
- **Do** build hierarchy through background luminance steps and 1px borders, not shadows (The Flat-By-Default Rule).
- **Do** set metadata labels in small-caps, letter-spaced, muted type; keep the data itself full-contrast and normal case (The Small-Caps Metadata Rule).
- **Do** use tabular numerals and fixed table layout for any new data table, so numeric columns stay aligned and long text truncates predictably.

### Don't:
- **Don't** let the accent gradient spread to card backgrounds, section headers, or other decorative fills — that reads as a generic SaaS dashboard, which this system exists to avoid.
- **Don't** introduce box-shadows for card elevation or button emphasis; the only shadow in the system is the input focus ring.
- **Don't** mix in a second typeface for "personality" — Inter alone carries the entire hierarchy here.
