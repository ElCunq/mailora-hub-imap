# Known Issues (as of 2025-11-06)

- Gmail Sent UID resolution
  - Symptom: After SMTP send, UID often remains null initially and finalize may return found=false within first 10–60s.
  - Cause: Gmail indexes the SMTP-sent copy into IMAP labels (Sent/All Mail) asynchronously; IMAP UID becomes available after a delay.
  - Current behavior: We skip APPEND for Gmail, return fast, and run background finalize (X-GM-RAW + Message-Id + quick scan, including [Gmail]/All Mail). A 60s retry loop is scheduled. If still missing, message remains pending and will be resolved on the next sync.
  - Status: Pending improvement (consider UIDPLUS alternatives not applicable; rely on sync).

- TODO: Capture APPENDUID when UIDPLUS is available on non-Gmail providers (optimize immediate UID return).
