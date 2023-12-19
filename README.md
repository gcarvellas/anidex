# Anidex

A CLI to check an AniList user's manga entry progress with the latest chapter of MangaDex.

I am in no way affiliated with AniList or MangaDex. This is an unofficial non-profit tool using both of their API's. I give credit to AniList and MangaDex for their APIs

# How to use

## Requirements

- Rust
- AniList account

## How to use

`cargo run --release -- --username {ANILIST_USERNAME} --language {MANGADEX_CHAPTTER_LANGUAGE} --jobs {NUM_WORKERS}`

Note: I have only tested this with the language "en"

## Example

`cargo run --release -- --username josh --language en --jobs 12`

``
