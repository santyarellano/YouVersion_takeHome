# YouVersion Associate Engineer Take-Home — Start Here

Thanks for your interest in joining the team! This take-home is your chance to show us how you build software. Read this whole document before you start — **how well you follow the instructions is part of the assessment.**

## Choose ONE project

- **[Project A — Verse of the Day API](project-a-api.md)** (backend)
- **[Project B — Verse of the Day App](project-b-frontend.md)** (frontend)

Pick whichever plays to your strengths. Neither is "worth more" than the other.

## Ground rules

**Time box: 2–4 hours.** We respect your time, and we've scoped this accordingly. A small, well-tested slice beats a large unfinished one. If you hit 4 hours, stop and write down what you'd do next in your README — that's a perfectly good submission.

**Stack: your choice.** Use any language or framework you're comfortable in. For context: our API services are primarily Python and our web apps use React. Using those helps us picture you in the role, but a strong submission in any stack beats a weak one in ours.

**AI tools are allowed.** Use Copilot, Claude, ChatGPT, whatever helps you. One honest warning: in the follow-up interview you will walk us through your code, modify it live, and answer questions about why it works the way it does. Submit only code you fully understand. Code you can't explain will hurt you far more than a smaller submission you know cold.

**Questions are encouraged.** If something in the spec is unclear, email us — asking a good clarifying question is a *positive* signal, not a penalty. If you'd rather keep momentum, make a reasonable assumption and document it (see below). Either approach scores well. Silently guessing does not.

## What to submit

A **public GitHub repository** — when you're done, reply to your assignment email with the link. Your repo's **README.md must include**:

1. **Setup & run instructions** — a teammate on a fresh machine should be able to run your project and your tests by following them exactly.
2. **Decisions & Assumptions** — a section listing the judgment calls you made where the spec left room for interpretation, and *why* you chose what you chose.
3. **What I'd do next** — what you'd add or improve with more time.

## YouVersion Platform API reference

Both projects use the real YouVersion Platform API. Full docs: https://developers.youversion.com/

- **Base URL:** `https://api.youversion.com`
- **Auth:** send the app key in the `x-yvp-app-key` header on every request
- **App key (provided for this exercise):** Set via `YOUVERSION_APP_KEY` in your local `.env` file (see `.env.example`).

### Endpoints you'll need

**1. Verse of the day for a given day of the year** (day is 1–366; Jan 1 = day 1)

```
GET /v1/verse_of_the_days/{day}
```
```json
{ "day": 195, "passage_id": "REV.3.20" }
```

(`GET /v1/verse_of_the_days` with no day returns the whole year's calendar as `{ "data": [ { "day": 1, "passage_id": "..." }, ... ] }`. Note that some `passage_id`s span multiple verses, e.g. `"ISA.43.18-19"`.)

**2. List available Bible versions**

```
GET /v1/bibles?language_ranges[]=en*
```

Returns `{ "data": [ ... ] }` where each item includes `id`, `abbreviation`, `title`, and `language_tag`.

**3. Get passage text for a version**

```
GET /v1/bibles/{version_id}/passages/{passage_id}?format=text
```

For example, `GET /v1/bibles/12/passages/REV.3.20?format=text` returns:

```json
{
  "id": "REV.3.20",
  "content": "Behold, I stand at the door and knock: if any man hear my voice and open the door, I will come in to him, and will sup with him, and he with me.",
  "reference": "Revelation 3:20"
}
```

### Important notes

- The provided key can access **public-domain versions** such as `206` (World English Bible), `12` (American Standard Version), and `3034` (Berean Standard Bible). Some versions — e.g. `111` (NIV) — return `403 Access denied`. This is expected licensing behavior, not a bug in your code.
- Verify your setup before you start building:

```bash
curl "https://api.youversion.com/v1/verse_of_the_days/195" \
  -H "x-yvp-app-key: $YOUVERSION_APP_KEY"
```

Good luck — we're looking forward to seeing what you build.
