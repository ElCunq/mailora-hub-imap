# Agent Rules (mailora-hub-imap)

- Prefer UID-based IMAP operations; do not advance cursors on empty FETCH.
- When SEARCH shows newer UIDs but FETCH is empty, emit minimal additions and retry later for enrichment.
- Always SELECT the target mailbox before FETCH; consider NOOP to flush pipeline.
- For /body, fetch BODYSTRUCTURE and choose the best text part; fallback through [TEXT] → [1.TEXT] → [1.1.TEXT] → [1] → [1.1].
- Exclude Spam/Junk/Çöp from default scans.
- Add tracing at mailbox/UID boundaries: selecting, search results, fetch counts, chosen section.
- Expose folder in change events so UI can fetch body from the correct mailbox.
- Return JSON error bodies for non-404 errors during IMAP fetch anomalies (optional improvement).
