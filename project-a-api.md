# Project A — Verse of the Day API

Build a small HTTP API service that gives its callers the verse of the day as **one clean response**, hiding the two-step YouVersion API dance behind a single endpoint.

Read [START-HERE.md](START-HERE.md) first — it has the ground rules, the app key, and the YouVersion API reference.

## The feature

YouVersion's API requires two calls to show a verse of the day:

1. `GET /v1/verse_of_the_days/{day}` → returns a `passage_id` (e.g. `"REV.3.20"`)
2. `GET /v1/bibles/{version_id}/passages/{passage_id}?format=text` → returns the verse text

Your service makes both calls internally and merges them, so *your* callers make one request:

```
GET /votd?day=195&version=206
```
```json
{
  "day": 195,
  "reference": "Revelation 3:20",
  "text": "Behold, I stand at the door and knock: if any man hear my voice and open the door, I will come in to him, and will sup with him, and he with me.",
  "version_id": 206
}
```

### Parameters

| Param | Type | Notes |
|---|---|---|
| `day` | integer, 1–366 | Optional. When omitted, use today's day of the year. |
| `version` | integer | A YouVersion Bible version ID (e.g. `206`, `12`, `3034`). |

The spec intentionally leaves some details open (for example: when `day` is omitted, *whose* "today" is it? what happens when `version` is omitted?). Make sensible calls and record them in your README's **Decisions & Assumptions** section — or email us and ask.

## Requirements

These are specific and we will check each one:

1. **Success response shape** — exactly the four fields shown above: `day` (integer), `reference` (string), `text` (string), `version_id` (integer).
2. **Error response shape** — every error returns JSON of exactly this shape:
   ```json
   { "error": { "code": "INVALID_DAY", "message": "day must be an integer between 1 and 366" } }
   ```
   (`code` is a stable machine-readable string of your choosing; `message` is human-readable.)
3. **Status codes** — `400` for an invalid `day` (e.g. `0`, `367`, `abc`); `502` when the YouVersion API is unreachable or returns an unexpected error.
4. **Caching** — YouVersion's verse-of-the-day calendar is static. Requesting the same `day` + `version` twice must **not** result in a second round-trip to YouVersion. An in-memory cache is fine.
5. **Scope boundary** — do **not** build authentication, user accounts, or a database. We mean it — extra scope here costs you, it doesn't impress us.
6. **Tests** — required, runnable with a single documented command, and must cover at least:
   - the happy path,
   - an invalid `day`,
   - a YouVersion failure (e.g. upstream 500 or timeout).

   Tests must **not** call the live YouVersion API — mock/stub/fake it.
7. **The app key must never appear** in your API's responses or its log output.

## Optional stretch (only if you're under the time box)

`GET /versions` — returns the English Bible versions available to this app key (id, abbreviation, title), backed by `GET /v1/bibles?language_ranges[]=en*`.

## What we're evaluating

Meeting the numbered requirements above, meaningful tests, how you handled the parts the spec left open, and code a teammate would enjoy maintaining. Not: framework choice, deployment, or visual polish (there's no UI here).
