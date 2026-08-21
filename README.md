# LIBERARME

## Overview

A local-first desktop application that helps users identify launcher-independent games in their Steam library, discover DRM-free alternatives, and preserve verified games through archiving.

## Current Status

Pre-Alpha

## Tech Stack

- Tauri
- Rust
- Svelte
- TypeScript
- Bun
- SQLite

## Initial Goal

Scan a local Steam library and display detected games.

## Setup

### Steam Web API Key

1. Go to **Settings** in the app.
2. Click **Get your key from Steam →** (opens `https://steamcommunity.com/dev/apikey`).
3. Log in and register for a Steam Web API key (any domain name works for local/personal use).
4. Paste the key into the **Steam API Key** field. It saves automatically when you click away from the field.

### Finding Your SteamID64

Your SteamID64 is a 17-digit number Steam uses internally. If you don't know it:

1. In **Settings**, paste your Steam profile URL (e.g. `steamcommunity.com/id/yourname`) or just your vanity username into the **SteamID64 Helper** field.
2. Click **Resolve**. The app calls Steam's `ResolveVanityURL` API and fills in the numeric SteamID64 for you.
3. If you already know your numeric SteamID64, you can paste it directly into the **SteamID64** field instead.

### Syncing Your Library

- **Scan Library** reads your local Steam installation directly (via `libraryfolders.vdf` and `appmanifest_*.acf` files) and requires no API key or internet connection.
- **Sync Owned Games** calls the Steam Web API using your saved API key and SteamID64, and requires your Steam profile/game details to be set to public visibility.

All credentials are stored locally in SQLite (`app.db`) and are never sent anywhere except directly to Steam's API.

## Roadmap

See ROADMAP.md