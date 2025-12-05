use anyhow::Result;
use async_imap::Session;
use base64::Engine; // needed for STANDARD.decode
use futures::StreamExt;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio_native_tls::native_tls::TlsConnector;
use tokio_util::compat::TokioAsyncReadCompatExt;

#[derive(Debug, Serialize)]
pub struct NewMessageMeta {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: Option<String>,
    pub size: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct MessageBodyMeta {
    pub uid: u32,
    pub subject: String,
    pub from: String,
    pub date: Option<String>,
    pub size: Option<u32>,
    pub flags: Vec<String>,
    pub body: String,
    pub html_body: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentMeta {
    pub uid: u32,
    pub part_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct AttachmentSummary { pub count: usize, pub top: Vec<String> }

pub struct SnapshotResult {
    pub uidvalidity: u32,
    pub last_uid: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct MessageState {
    pub uid: u32,
    pub flags: Vec<String>,
}

pub async fn initial_snapshot(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
) -> Result<SnapshotResult> {
    if port == 143 {
        let tcp = TcpStream::connect((host, port)).await?;
        let client = async_imap::Client::new(tcp.compat());
        let mut session: Session<_> = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
        let mailbox = session.select("INBOX").await?;
        let mut last_uid: u32 = 0;
        if let Ok(uids) = session.uid_search("ALL").await {
            for uid in uids {
                if uid > last_uid {
                    last_uid = uid;
                }
            }
        }
        let _ = session.logout().await;
        return Ok(SnapshotResult {
            uidvalidity: mailbox.uid_validity.unwrap_or(0) as u32,
            last_uid,
        });
    } else {
        let tcp = TcpStream::connect((host, port)).await?;
        let tls = TlsConnector::builder().build()?;
        let tls = tokio_native_tls::TlsConnector::from(tls);
        let tls_stream = tls.connect(host, tcp).await?;
        let client = async_imap::Client::new(tls_stream.compat());
        let mut session: Session<_> = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
        let mailbox = session.select("INBOX").await?;
        let uidvalidity = mailbox.uid_validity.unwrap_or(0) as u32;
        let mut last_uid: u32 = 0;
        if let Ok(uids) = session.uid_search("ALL").await {
            for uid in uids {
                if uid > last_uid {
                    last_uid = uid;
                }
            }
        }
        let _ = session.logout().await;
        return Ok(SnapshotResult {
            uidvalidity,
            last_uid,
        });
    }
}

pub async fn fetch_new_since(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    last_uid: u32,
) -> Result<(u32, Vec<NewMessageMeta>)> {
    if port == 143 {
        let tcp = TcpStream::connect((host, port)).await?;
        let client = async_imap::Client::new(tcp.compat());
        let mut session: Session<_> = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
        let mailbox = session.select("INBOX").await?;
        let all = session.uid_search("ALL").await?;
        let mut present_max: u32 = last_uid;
        for uid in &all {
            if *uid > present_max {
                present_max = *uid;
            }
        }
        let mut newer: Vec<u32> = all
            .into_iter()
            .filter(|u| *u >= last_uid.saturating_sub(10))
            .collect();
        newer.sort_unstable();
        tracing::debug!(
            last_uid,
            uid_next = mailbox.uid_next.map(|v| v as u32),
            present_max,
            newer_count = newer.len(),
            "imap.fetch_new_since using ALL to compute newer UIDs"
        );
        if newer.is_empty() {
            let _ = session.logout().await;
            return Ok((last_uid, Vec::new()));
        }
        let new_last = *newer.last().unwrap_or(&present_max);
        let seq = newer
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",");
        tracing::debug!(%seq, count = newer.len(), "imap.fetch_new_since fetching exact newer UIDs");
        let mut out = Vec::new();
        let mut fetches = session
            .uid_fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE")
            .await?;
        while let Some(item) = fetches.next().await {
            let f = item?;
            if let Some(uid) = f.uid {
                if uid <= last_uid {
                    continue;
                }
                let env = f.envelope();
                let subject = env
                    .and_then(|e| e.subject.as_ref())
                    .map(|b| decode_subject(b))
                    .unwrap_or_default();
                let from = env
                    .and_then(|e| e.from.as_ref())
                    .and_then(|v| v.get(0))
                    .map(format_address)
                    .unwrap_or_default();
                let date = f.internal_date().map(|d| d.to_rfc3339());
                let size = None;
                out.push(NewMessageMeta {
                    uid,
                    subject,
                    from,
                    date,
                    size,
                });
            }
        }
        drop(fetches);
        tracing::debug!(
            fetched = out.len(),
            new_last,
            "imap.fetch_new_since completed"
        );
        let _ = session.logout().await;
        return Ok((new_last, out));
    } else {
        let tcp = TcpStream::connect((host, port)).await?;
        let tls = TlsConnector::builder().build()?;
        let tls = tokio_native_tls::TlsConnector::from(tls);
        let tls_stream = tls.connect(host, tcp).await?;
        let client = async_imap::Client::new(tls_stream.compat());
        let mut session: Session<_> = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
        let mailbox = session.select("INBOX").await?;
        let all = session.uid_search("ALL").await?;
        let mut present_max: u32 = last_uid;
        for uid in &all {
            if *uid > present_max {
                present_max = *uid;
            }
        }
        let mut newer: Vec<u32> = all
            .into_iter()
            .filter(|u| *u >= last_uid.saturating_sub(10))
            .collect();
        newer.sort_unstable();
        tracing::debug!(
            last_uid,
            uid_next = mailbox.uid_next.map(|v| v as u32),
            present_max,
            newer_count = newer.len(),
            "imap.fetch_new_since using ALL to compute newer UIDs"
        );
        if newer.is_empty() {
            let _ = session.logout().await;
            return Ok((last_uid, Vec::new()));
        }
        let new_last = *newer.last().unwrap_or(&present_max);
        let seq = newer
            .iter()
            .map(|u| u.to_string())
            .collect::<Vec<_>>()
            .join(",");
        tracing::debug!(%seq, count = newer.len(), "imap.fetch_new_since fetching exact newer UIDs");
        let mut out = Vec::new();
        let mut fetches = session
            .uid_fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE")
            .await?;
        while let Some(item) = fetches.next().await {
            let f = item?;
            if let Some(uid) = f.uid {
                if uid <= last_uid {
                    continue;
                }
                let env = f.envelope();
                let subject = env
                    .and_then(|e| e.subject.as_ref())
                    .map(|b| decode_subject(b))
                    .unwrap_or_default();
                let from = env
                    .and_then(|e| e.from.as_ref())
                    .and_then(|v| v.get(0))
                    .map(format_address)
                    .unwrap_or_default();
                let date = f.internal_date().map(|d| d.to_rfc3339());
                let size = None;
                out.push(NewMessageMeta {
                    uid,
                    subject,
                    from,
                    date,
                    size,
                });
            }
        }
        drop(fetches);
        tracing::debug!(
            fetched = out.len(),
            new_last,
            "imap.fetch_new_since completed"
        );
        let _ = session.logout().await;
        return Ok((new_last, out));
    }
}

pub async fn fetch_message_body(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    uid: u32,
) -> Result<Option<MessageBodyMeta>> {
    fetch_message_body_in(host, port, email, password, uid, "INBOX").await
}

pub async fn fetch_message_body_in(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    uid: u32,
    folder: &str,
) -> Result<Option<MessageBodyMeta>> {
    if port == 143 {
        let tcp = TcpStream::connect((host, port)).await?;
        let client = async_imap::Client::new(tcp.compat());
        let mut session: Session<_> = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
        tracing::debug!(%folder, uid, "body_in: selecting folder");
        session.select(folder).await?;
        let _ = session.noop().await;
        let uid_str = uid.to_string();
        let mut head = session
            .uid_fetch(&uid_str, "UID ENVELOPE FLAGS BODYSTRUCTURE")
            .await?;
        let mut base: Option<(String, String, Option<String>, Vec<String>)> = None;
        let mut head_count: usize = 0;
        let mut head_uids: Vec<Option<u32>> = Vec::new();
        while let Some(item) = head.next().await {
            let f = item?;
            head_count += 1;
            head_uids.push(f.uid);
            if let Some(f_uid) = f.uid {
                if f_uid != uid {
                    continue;
                }
            }
            let env = f.envelope();
            let subject = env
                .and_then(|e| e.subject.as_ref())
                .map(|b| decode_subject(b))
                .unwrap_or_default();
            let from = env
                .and_then(|e| e.from.as_ref())
                .and_then(|v| v.get(0))
                .map(format_address)
                .unwrap_or_default();
            let date = f.internal_date().map(|d| d.to_rfc3339());
            let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
            base = Some((subject, from, date, flags));
        }
        tracing::debug!(%folder, uid, head_count, head_uids=?head_uids, "body_in: initial meta fetch result");
        drop(head);
        // Don't return early when base is missing; we'll try raw BODY[] and parse headers
        let (mut subject, mut from, mut date, flags) = match base {
            Some((s,f,d,fl)) => (s,f,d,fl),
            None => (String::new(), String::new(), None, Vec::new()),
        };
        let candidates = [
            "BODY.PEEK[]", // full raw message first to allow MIME parsing
            "BODY.PEEK[TEXT]",
            "BODY.PEEK[1.TEXT]",
            "BODY.PEEK[1.1.TEXT]",
            "BODY.PEEK[1]",
            "BODY.PEEK[1.1]",
        ];
        let mut body: Option<Vec<u8>> = None;
        let mut raw_full: Option<Vec<u8>> = None;
        let mut chosen: &str = "";
        for sect in &candidates {
            tracing::debug!(%folder, uid, section=%sect, "body_in: trying section");
            let mut part = session.uid_fetch(&uid_str, sect).await?;
            let mut got = None;
            while let Some(item) = part.next().await {
                let f = item?;
                if let Some(b) = f.body() {
                    if !b.is_empty() {
                        got = Some(b.to_vec());
                        break;
                    }
                }
            }
            drop(part);
            if let Some(v) = got {
                if *sect == "BODY.PEEK[]" { raw_full = Some(v.clone()); }
                if body.is_none() { chosen = sect; body = Some(v); }
                if body.is_some() && raw_full.is_some() { break; }
            }
        }
        // If ENVELOPE missing, try header fields from raw
        if subject.is_empty() || from.is_empty() || date.is_none() {
            if let Some(full) = raw_full.as_ref() {
                if let Ok(parsed) = mailparse::parse_mail(full) {
                    // Subject
                    if subject.is_empty() {
                        if let Some(h) = parsed.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("Subject")) {
                            subject = h.get_value().trim().to_string();
                        }
                    }
                    // From
                    if from.is_empty() {
                        if let Some(h) = parsed.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("From")) {
                            from = h.get_value().trim().to_string();
                        }
                    }
                    // Date
                    if date.is_none() {
                        if let Some(h) = parsed.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("Date")) {
                            let v = h.get_value().trim().to_string();
                            if !v.is_empty() { date = Some(v); }
                        }
                    }
                }
            }
        }
        let mut body_text = String::new();
        let mut html_opt: Option<String> = None;
        if let Some(bytes) = body {
            let mut s = String::from_utf8(bytes.clone())
                .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).to_string());
            if s.len() > 8000 { s.truncate(8000); s.push_str("\n...[truncated]..."); }
            tracing::debug!(%folder, uid, section=%chosen, len=s.len(), "body_in: got body");
            body_text = s;
        }
        // MIME parse for HTML (prefer full raw message if available)
        if let Some(full) = raw_full {
            if full.len() <= 512_000 { // safety cap
                if let Ok(parsed) = mailparse::parse_mail(&full) {
                    fn walk(parts: &mailparse::ParsedMail, plain: &mut Option<String>, html: &mut Option<String>) {
                        let ctype = parts.ctype.mimetype.to_lowercase();
                        if (ctype == "text/plain" || ctype == "text/html") && (parts.subparts.is_empty()) {
                            if let Ok(b) = parts.get_body() {
                                if ctype == "text/plain" && plain.is_none() { *plain = Some(b); }
                                else if ctype == "text/html" && html.is_none() { *html = Some(b); }
                            }
                        }
                        for sp in &parts.subparts { walk(sp, plain, html); }
                    }
                    let mut plain: Option<String> = None; let mut html: Option<String> = None;
                    walk(&parsed, &mut plain, &mut html);
                    if let Some(p) = plain { if body_text.is_empty() { body_text = p; } }
                    if let Some(h) = html {
                        let mut h2 = h;
                        if h2.len() > 16000 { h2.truncate(16000); h2.push_str("\n...[html truncated]..."); }
                        html_opt = Some(h2);
                    }
                }
            }
        }
        let _ = session.logout().await;
        return Ok(Some(MessageBodyMeta {
            uid,
            subject,
            from,
            date,
            size: None,
            flags,
            body: body_text,
            html_body: html_opt,
        }));
    } else {
        let tcp = TcpStream::connect((host, port)).await?;
        let tls = TlsConnector::builder().build()?;
        let tls = tokio_native_tls::TlsConnector::from(tls);
        let tls_stream = tls.connect(host, tcp).await?;
        let client = async_imap::Client::new(tls_stream.compat());
        let mut session: Session<_> = client
            .login(email, password)
            .await
            .map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
        tracing::debug!(%folder, uid, "body_in: selecting folder");
        session.select(folder).await?;
        let _ = session.noop().await;
        let uid_str = uid.to_string();
        let mut head = session
            .uid_fetch(&uid_str, "UID ENVELOPE FLAGS BODYSTRUCTURE")
            .await?;
        let mut base: Option<(String, String, Option<String>, Vec<String>)> = None;
        let mut head_count: usize = 0;
        let mut head_uids: Vec<Option<u32>> = Vec::new();
        while let Some(item) = head.next().await {
            let f = item?;
            head_count += 1;
            head_uids.push(f.uid);
            if let Some(f_uid) = f.uid {
                if f_uid != uid {
                    continue;
                }
            }
            let env = f.envelope();
            let subject = env
                .and_then(|e| e.subject.as_ref())
                .map(|b| decode_subject(b))
                .unwrap_or_default();
            let from = env
                .and_then(|e| e.from.as_ref())
                .and_then(|v| v.get(0))
                .map(format_address)
                .unwrap_or_default();
            let date = f.internal_date().map(|d| d.to_rfc3339());
            let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
            base = Some((subject, from, date, flags));
        }
        tracing::debug!(%folder, uid, head_count, head_uids=?head_uids, "body_in: initial meta fetch result");
        drop(head);
        // Don't return early when base is missing; we'll try raw BODY[] and parse headers
        let (mut subject, mut from, mut date, flags) = match base {
            Some((s,f,d,fl)) => (s,f,d,fl),
            None => (String::new(), String::new(), None, Vec::new()),
        };
        let candidates = [
            "BODY.PEEK[]", // full raw message first to allow MIME parsing
            "BODY.PEEK[TEXT]",
            "BODY.PEEK[1.TEXT]",
            "BODY.PEEK[1.1.TEXT]",
            "BODY.PEEK[1]",
            "BODY.PEEK[1.1]",
        ];
        let mut body: Option<Vec<u8>> = None;
        let mut raw_full: Option<Vec<u8>> = None;
        let mut chosen: &str = "";
        for sect in &candidates {
            tracing::debug!(%folder, uid, section=%sect, "body_in: trying section");
            let mut part = session.uid_fetch(&uid_str, sect).await?;
            let mut got = None;
            while let Some(item) = part.next().await {
                let f = item?;
                if let Some(b) = f.body() {
                    if !b.is_empty() {
                        got = Some(b.to_vec());
                        break;
                    }
                }
            }
            drop(part);
            if let Some(v) = got {
                if *sect == "BODY.PEEK[]" { raw_full = Some(v.clone()); }
                if body.is_none() { chosen = sect; body = Some(v); }
                if body.is_some() && raw_full.is_some() { break; }
            }
        }
        // If ENVELOPE missing, try header fields from raw
        if subject.is_empty() || from.is_empty() || date.is_none() {
            if let Some(full) = raw_full.as_ref() {
                if let Ok(parsed) = mailparse::parse_mail(full) {
                    // Subject
                    if subject.is_empty() {
                        if let Some(h) = parsed.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("Subject")) {
                            subject = h.get_value().trim().to_string();
                        }
                    }
                    // From
                    if from.is_empty() {
                        if let Some(h) = parsed.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("From")) {
                            from = h.get_value().trim().to_string();
                        }
                    }
                    // Date
                    if date.is_none() {
                        if let Some(h) = parsed.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("Date")) {
                            let v = h.get_value().trim().to_string();
                            if !v.is_empty() { date = Some(v); }
                        }
                    }
                }
            }
        }
        let mut body_text = String::new();
        let mut html_opt: Option<String> = None;
        if let Some(bytes) = body {
            let mut s = String::from_utf8(bytes.clone())
                .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).to_string());
            if s.len() > 8000 { s.truncate(8000); s.push_str("\n...[truncated]..."); }
            tracing::debug!(%folder, uid, section=%chosen, len=s.len(), "body_in: got body");
            body_text = s;
        }
        // MIME parse for HTML (prefer full raw message if available)
        if let Some(full) = raw_full {
            if full.len() <= 512_000 { // safety cap
                if let Ok(parsed) = mailparse::parse_mail(&full) {
                    fn walk(parts: &mailparse::ParsedMail, plain: &mut Option<String>, html: &mut Option<String>) {
                        let ctype = parts.ctype.mimetype.to_lowercase();
                        if (ctype == "text/plain" || ctype == "text/html") && (parts.subparts.is_empty()) {
                            if let Ok(b) = parts.get_body() {
                                if ctype == "text/plain" && plain.is_none() { *plain = Some(b); }
                                else if ctype == "text/html" && html.is_none() { *html = Some(b); }
                            }
                        }
                        for sp in &parts.subparts { walk(sp, plain, html); }
                    }
                    let mut plain: Option<String> = None; let mut html: Option<String> = None;
                    walk(&parsed, &mut plain, &mut html);
                    if let Some(p) = plain { if body_text.is_empty() { body_text = p; } }
                    if let Some(h) = html {
                        let mut h2 = h;
                        if h2.len() > 16000 { h2.truncate(16000); h2.push_str("\n...[html truncated]..."); }
                        html_opt = Some(h2);
                    }
                }
            }
        }
        let _ = session.logout().await;
        return Ok(Some(MessageBodyMeta {
            uid,
            subject,
            from,
            date,
            size: None,
            flags,
            body: body_text,
            html_body: html_opt,
        }));
    }
}

fn format_address(a: &async_imap::imap_proto::Address<'_>) -> String {
    let name = a.name.as_ref().map(|n| decode_subject(n)).unwrap_or_default();
    let mailbox = a.mailbox.as_ref().map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
    let host = a.host.as_ref().map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
    let mut s = String::new();
    if !name.is_empty() { s.push_str(&name); s.push(' '); }
    if !mailbox.is_empty() || !host.is_empty() { s.push('<'); s.push_str(&mailbox); if !host.is_empty(){ s.push('@'); s.push_str(&host);} s.push('>'); }
    s.trim().to_string()
}

pub(crate) fn decode_subject(raw: &[u8]) -> String {
    let mut composed = b"Subject: ".to_vec(); composed.extend_from_slice(raw);
    match mailparse::parse_header(&composed) { Ok((h,_)) => h.get_value().trim().to_string(), Err(_) => String::from_utf8_lossy(raw).trim().to_string() }
}

pub async fn list_attachments(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    folder: &str,
    uid: u32,
) -> Result<Vec<AttachmentMeta>> {
    use futures::StreamExt;
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let client = async_imap::Client::new(tls_stream.compat());
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    session.select(folder).await?;
    let uid_str = uid.to_string();
    let mut fetches = session.uid_fetch(&uid_str, "UID BODY.PEEK[]").await?;
    let mut raw: Option<Vec<u8>> = None;
    while let Some(item) = fetches.next().await {
        let f = item?;
        if f.uid == Some(uid) {
            if let Some(b) = f.body() { raw = Some(b.to_vec()); break; }
        }
    }
    drop(fetches);
    let _ = session.logout().await;
    let mut out = Vec::new();
    let raw = match raw { Some(r) => r, None => return Ok(out) };
    // Increase cap to handle larger emails with attachments
    if raw.len() > 15_000_000 {
        tracing::debug!(uid, folder=%folder, size=raw.len(), "list_attachments: raw too large, skipping parse");
        return Ok(out);
    }
    if let Ok(parsed) = mailparse::parse_mail(&raw) {
        fn walk(pm: &mailparse::ParsedMail, prefix: &str, out: &mut Vec<AttachmentMeta>, uid: u32) {
            if pm.subparts.is_empty() {
                let ctype = pm.ctype.mimetype.to_lowercase();
                let (fname, disp, _cid) = extract_filename_from_headers(&pm.headers);
                let is_multipart = ctype.starts_with("multipart/");
                let is_textual = ctype == "text/plain" || ctype == "text/html";
                let has_name = fname.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
                let disp_is_attachment = disp.as_deref().map(|d| d.eq_ignore_ascii_case("attachment")).unwrap_or(false);
                let disp_is_inline = disp.as_deref().map(|d| d.eq_ignore_ascii_case("inline")).unwrap_or(false);
                let treat_as_attachment =
                    (disp_is_attachment) ||
                    (disp_is_inline && has_name) ||
                    (!is_multipart && (has_name || !is_textual));
                if treat_as_attachment {
                    out.push(AttachmentMeta {
                        uid,
                        part_id: prefix.to_string(),
                        filename: fname,
                        content_type: Some(ctype),
                        size: Some(pm.get_body_raw().map(|b| b.len() as u64).unwrap_or(0)),
                    });
                }
                return;
            }
            for (idx, sp) in pm.subparts.iter().enumerate() {
                let part_id = if prefix.is_empty() { format!("{}", idx+1) } else { format!("{}.{}", prefix, idx+1) };
                walk(sp, &part_id, out, uid);
            }
        }
        if parsed.subparts.is_empty() {
            let ctype = parsed.ctype.mimetype.to_lowercase();
            let (fname, disp, _cid) = extract_filename_from_headers(&parsed.headers);
            let is_multipart = ctype.starts_with("multipart/");
            let is_textual = ctype == "text/plain" || ctype == "text/html";
            let has_name = fname.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
            let disp_is_attachment = disp.as_deref().map(|d| d.eq_ignore_ascii_case("attachment")).unwrap_or(false);
            let disp_is_inline = disp.as_deref().map(|d| d.eq_ignore_ascii_case("inline")).unwrap_or(false);
            let treat_as_attachment =
                (disp_is_attachment) ||
                (disp_is_inline && has_name) ||
                (!is_multipart && (has_name || !is_textual));
            if treat_as_attachment {
                out.push(AttachmentMeta { uid, part_id: "1".into(), filename: fname, content_type: Some(ctype), size: Some(parsed.get_body_raw().map(|b| b.len() as u64).unwrap_or(0)) });
            }
        } else {
            for (idx, sp) in parsed.subparts.iter().enumerate() {
                let part_id = format!("{}", idx+1);
                walk(sp, &part_id, &mut out, uid);
            }
        }
    }
    Ok(out)
}

pub async fn fetch_attachment_part(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    folder: &str,
    uid: u32,
    target_part: &str,
) -> Result<Option<(Vec<u8>, Option<String>, Option<String>)>> {
    use futures::StreamExt;
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let client = async_imap::Client::new(tls_stream.compat());
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    session.select(folder).await?;
    let uid_str = uid.to_string();
    let mut fetches = session.uid_fetch(&uid_str, "UID BODY.PEEK[]").await?;
    let mut raw: Option<Vec<u8>> = None;
    while let Some(item) = fetches.next().await { let f = item?; if f.uid == Some(uid) { if let Some(b) = f.body() { raw = Some(b.to_vec()); break; } } }
    drop(fetches);
    let _ = session.logout().await;
    let raw = match raw { Some(r) => r, None => return Ok(None) };
    // Increase cap here as well for larger attachments embedded in emails
    if raw.len() > 25_000_000 { return Ok(None); }
    if let Ok(parsed) = mailparse::parse_mail(&raw) {
        // Helper: locate by numeric part id
        fn locate<'a>(pm: &'a mailparse::ParsedMail<'a>, prefix: &str, target: &str) -> Option<&'a mailparse::ParsedMail<'a>> {
            if prefix == target { return Some(pm); }
            if pm.subparts.is_empty() { return None; }
            for (idx, sp) in pm.subparts.iter().enumerate() {
                let part_id = if prefix.is_empty() { format!("{}", idx+1) } else { format!("{}.{}", prefix, idx+1) };
                if let Some(found) = locate(sp, &part_id, target) { return Some(found); }
            }
            None
        }
        // Helper: locate by Content-ID
        fn locate_by_cid<'a>(pm: &'a mailparse::ParsedMail<'a>, prefix: &str, cid: &str) -> Option<&'a mailparse::ParsedMail<'a>> {
            let wanted = cid.trim().trim_matches(['<','>']).to_lowercase();
            // Check current
            if let Some(h) = pm.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("Content-ID")) {
                let val = h.get_value();
                let norm = val.trim().trim_matches(['<','>']).to_lowercase();
                if norm == wanted { return Some(pm); }
            }
            // Recurse
            for (idx, sp) in pm.subparts.iter().enumerate() {
                let part_id = if prefix.is_empty() { format!("{}", idx+1) } else { format!("{}.{}", prefix, idx+1) };
                if let Some(found) = locate_by_cid(sp, &part_id, cid) { return Some(found); }
            }
            None
        }
        // Decide which search to use
        let looks_numeric = target_part.chars().all(|c| c.is_ascii_digit() || c == '.');
        let candidate = if looks_numeric {
            if parsed.subparts.is_empty() {
                if target_part == "1" { Some(&parsed) } else { None }
            } else {
                let mut found: Option<&mailparse::ParsedMail> = None;
                for (idx, sp) in parsed.subparts.iter().enumerate() {
                    let part_id = format!("{}", idx+1);
                    if let Some(x) = locate(sp, &part_id, target_part) { found = Some(x); break; }
                }
                found
            }
        } else {
            // CID
            if let Some(x) = locate_by_cid(&parsed, "", target_part) { Some(x) } else { None }
        };
        if let Some(found) = candidate {
            let ctype = found.ctype.mimetype.to_lowercase();
            let (fname, _disp, _cid) = extract_filename_from_headers(&found.headers);
            let mut bytes = found.get_body_raw().unwrap_or_default();
            let enc = found.headers.iter().find(|h| h.get_key_ref().eq_ignore_ascii_case("Content-Transfer-Encoding")).map(|h| h.get_value().to_lowercase());
            if let Some(encv) = enc { if encv.contains("base64") { if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&bytes) { bytes = decoded; } } }
            return Ok(Some((bytes, Some(ctype), fname)));
        }
    }
    Ok(None)
}

fn percent_decode_simple(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let h1 = bytes[i + 1];
            let h2 = bytes[i + 2];
            let v = (match (h1 as char).to_digit(16) { Some(v) => v, None => { out.push(bytes[i]); i += 1; continue; } } << 4)
                | match (h2 as char).to_digit(16) { Some(v) => v, None => { out.push(bytes[i]); i += 1; continue; } };
            out.push(v as u8);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).to_string()
}

fn decode_rfc2231(value: &str) -> String {
    // format: charset'lang'value (value percent-encoded)
    let parts: Vec<&str> = value.splitn(3, '\'').collect();
    if parts.len() == 3 {
        let charset = parts[0].trim();
        let val_raw = parts[2];
        let decoded = percent_decode_simple(val_raw);
        // basic charset handling: utf-8 or fallback
        if charset.eq_ignore_ascii_case("utf-8") || charset.is_empty() {
            return decoded;
        }
        // best effort: return decoded bytes interpreted as utf-8
        return decoded;
    }
    percent_decode_simple(value)
}

fn extract_filename_from_headers(headers: &[mailparse::MailHeader]) -> (Option<String>, Option<String>, Option<String>) {
    // returns (filename, disposition, content_id)
    let mut filename: Option<String> = None;
    let mut disposition: Option<String> = None;
    let mut content_id: Option<String> = None;

    for h in headers {
        let key = h.get_key();
        if key.eq_ignore_ascii_case("Content-ID") {
            let v = h.get_value();
            let norm = v.trim().trim_matches(['<','>']).to_string();
            if !norm.is_empty() { content_id = Some(norm); }
            continue;
        }
        if key.eq_ignore_ascii_case("Content-Disposition") || key.eq_ignore_ascii_case("Content-Type") {
            let raw = h.get_value();
            // split into type + params
            let mut iter = raw.split(';');
            if key.eq_ignore_ascii_case("Content-Disposition") {
                if let Some(first) = iter.next() { disposition = Some(first.trim().to_lowercase()); }
            } else {
                // keep previous disposition
                let _ = iter.next();
            }
            // collect extended/segmented params
            use std::collections::BTreeMap;
            let mut segments: BTreeMap<(String, bool), BTreeMap<usize, String>> = BTreeMap::new();
            let mut simple_params: Vec<(String,String,bool)> = Vec::new(); // (name, value, extended)
            for token in iter {
                let t = token.trim();
                if t.is_empty() { continue; }
                let mut kv = t.splitn(2, '=');
                let k = kv.next().unwrap_or("").trim().trim_matches('"');
                let vraw = kv.next().unwrap_or("").trim().trim_matches('"');
                if k.is_empty() { continue; }
                // detect filename*0*, filename*1*, filename*
                let kl = k.to_lowercase();
                if let Some(base) = kl.strip_suffix('*') {
                    // extended single
                    simple_params.push((base.to_string(), decode_rfc2231(vraw), true));
                } else if let Some(idxpos) = kl.rfind('*') {
                    let (base, rest) = kl.split_at(idxpos);
                    if rest.starts_with('*') {
                        // possibly *0* or *1*
                        let num_str = &rest[1..rest.len()-1].trim_end_matches('*');
                        if let Ok(num) = num_str.parse::<usize>() {
                            segments.entry((base.to_string(), true)).or_default().insert(num, percent_decode_simple(vraw));
                        }
                    } else {
                        simple_params.push((kl.clone(), vraw.to_string(), false));
                    }
                } else {
                    simple_params.push((kl.clone(), vraw.to_string(), false));
                }
            }
            // reconstruct segmented extended params
            for ((base, _), parts) in segments.into_iter() {
                let mut s = String::new();
                for (_i, val) in parts.into_iter() { s.push_str(&val); }
                simple_params.push((base, s, true));
            }
            // choose filename priority: filename*, name*, filename, name
            let mut cand: Option<String> = None;
            for (n, v, ext) in simple_params.iter() {
                if (n == "filename" || n == "name") && *ext {
                    cand = Some(v.clone()); break;
                }
            }
            if cand.is_none() {
                for (n, v, _ext) in simple_params.iter() {
                    if n == "filename" || n == "name" { cand = Some(v.clone()); break; }
                }
            }
            if filename.is_none() {
                if let Some(v) = cand {
                    if !v.is_empty() { filename = Some(v); }
                }
            }
        }
    }

    (filename, disposition.map(|d| d.split_whitespace().next().unwrap_or("").to_string()), content_id)
}
