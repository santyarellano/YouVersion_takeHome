# Verse of the Day API

For a visual diagram and component walkthrough, see [design.md](design.md).

---

## 1. Setup & Run Instructions

### Prerequisites
* [Rust](https://www.rust-lang.org/) (version 1.70 or newer).

### Configuration
1. Copy the example settings file:
   ```bash
   cp .env.example .env
   ```
2. Open `.env` and add your YouVersion app key if you plan to connect to the live service:
   ```bash
   YOUVERSION_APP_KEY=your_key_here
   ```

### Running the Tests
To run all automated tests:
```bash
cargo test
```
* Runs all 21 tests.
* Tests run completely locally using a simulated client — no network calls or API keys required.

### Starting the Server
To start the API:
```bash
cargo run
```
The server will start on `http://127.0.0.1:3000`.

### Example Requests
* **Get today's verse (defaults to World English Bible):**
  ```bash
  curl http://127.0.0.1:3000/votd
  ```

* **Get a specific day and Bible translation:**
  ```bash
  curl "http://127.0.0.1:3000/votd?day=195&version=206"
  ```

* **List available English Bible translations:**
  ```bash
  curl http://127.0.0.1:3000/versions
  ```

---

## 2. Decisions & Assumptions

* **Default Day (UTC):** When the caller leaves out the `day` parameter, we calculate today's day of the year using UTC time. This ensures consistent, predictable results no matter what time zone the user or server is in.
* **Default Bible Version (206):** When the caller leaves out the `version` parameter, we default to version `206` (World English Bible). This just based on the references to the instructions as it is the same version.
* **Two-Tier Cache:** Since YouVersion's calendar is static (Day 195 is always Revelation 3:20 regardless of translation), we store the day's verse reference separately from the translation text. If one person asks for Day 195 in the World English Bible and another asks for Day 195 in the American Standard Version, our system already knows the reference and skips that first network step.

---

## 3. What I'd Do Next

* **Cache Limits & Expiration:** Right now, cached verses stay in memory while the server runs. With more time, I would add a maximum memory limit and automatic cleanup for entries that haven't been viewed in a while.
* **Rate Limiting:** Add a protective limit to prevent any single client from sending too many requests at once, protecting both our server and YouVersion.
* **Graceful Shutdown:** Allow the server to finish handling any active requests before turning off when stopped.
