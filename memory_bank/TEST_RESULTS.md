# Test Results (v1.1.0-dev)

## Auto-Discovery (Magic Login)
- **ISPDB (Mozilla):**
  - Gmail: `imap.gmail.com:993` / `smtp.gmail.com:587` successfully discovered.
  - Outlook: `outlook.office365.com` successfully discovered.
- **DNS SRV:**
  - Tested with local simulation; fallback logic works.
- **Manual Fallback:**
  - "Manuel Kurulum" toggle works correctly when discovery fails or is skipped.

## UI Tests
- **Dark Theme:**
  - `add_account.html` matches `app.html` palette.
  - No visual glitches observed in recent Chrome/Firefox.
- **Navigation:**
  - Sidebar button correctly redirects to `add_account.html`.
  - Back link in `add_account.html` returns to `app.html`.

## Stability Tests
- **Sync Loop:**
  - Simulated network error (5 consecutive failures) triggered the circuit breaker.
  - Logs stopped spamming, `error_count` reset logic verified.
- **Port Conflict:**
  - `cargo run` automatically kills previous process on port 3030. Server starts successfully.
- **Settings Save:**
  - Validated that `PATCH /accounts/:id` now returns the expected JSON structure.
  - Frontend correctly handles the response and updates the UI.
