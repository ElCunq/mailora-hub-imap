use anyhow::Result;
use tokio::net::TcpStream;
use async_imap::Session;
use tokio_util::compat::TokioAsyncReadCompatExt;
use tokio_native_tls::native_tls::TlsConnector;
use serde::Serialize;
use futures::StreamExt;

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
}

#[derive(Debug, Serialize)]
pub struct AttachmentMeta {
    pub uid: u32,
    pub part_id: String,
    pub filename: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
}

pub struct SnapshotResult {
    pub uidvalidity: u32,
    pub last_uid: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct MessageState { pub uid: u32, pub flags: Vec<String> }

pub async fn initial_snapshot(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
) -> Result<SnapshotResult> {
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;

    let mailbox = session.select("INBOX").await?;
    let uidvalidity = mailbox.uid_validity.unwrap_or(0) as u32;

    let mut last_uid: u32 = 0;
    if let Ok(uids) = session.uid_search("ALL").await {
        for uid in uids { if uid > last_uid { last_uid = uid; } }
    }

    let _ = session.logout().await;
    Ok(SnapshotResult { uidvalidity, last_uid })
}

pub async fn fetch_new_since(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    last_uid: u32,
) -> Result<(u32, Vec<NewMessageMeta>)> {
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    let mailbox = session.select("INBOX").await?;

    // Most robust: compute newer UIDs from ALL
    let all = session.uid_search("ALL").await?;
    let mut present_max: u32 = last_uid;
    for uid in &all { if *uid > present_max { present_max = *uid; } }
    let mut newer: Vec<u32> = all.into_iter().filter(|u| *u > last_uid).collect();
    newer.sort_unstable();
    tracing::debug!(last_uid, uid_next = mailbox.uid_next.map(|v| v as u32), present_max, newer_count = newer.len(), "imap.fetch_new_since using ALL to compute newer UIDs");

    if newer.is_empty() {
        let _ = session.logout().await;
        return Ok((last_uid, Vec::new()));
    }

    let new_last = *newer.last().unwrap_or(&present_max);
    let seq = newer.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
    tracing::debug!(%seq, count = newer.len(), "imap.fetch_new_since fetching exact newer UIDs");

    let mut out = Vec::new();
    let mut fetches = session.uid_fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE").await?;
    while let Some(item) = fetches.next().await {
        let f = item?;
        if let Some(uid) = f.uid {
            if uid <= last_uid { continue; }
            let env = f.envelope();
            let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
            let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
            let date = f.internal_date().map(|d| d.to_rfc3339());
            let size = None;
            out.push(NewMessageMeta { uid, subject, from, date, size });
        }
    }
    drop(fetches);
    tracing::debug!(fetched = out.len(), new_last, "imap.fetch_new_since completed");
    let _ = session.logout().await;
    Ok((new_last, out))
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
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;

    tracing::debug!(%folder, uid, "body_in: selecting folder");
    session.select(folder).await?;
    // Clear pipeline
    let _ = session.noop().await;

    // First fetch envelope/flags/bodystructure (use bare UID, not a range)
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
        if let Some(f_uid) = f.uid { if f_uid != uid { continue; } }
        let env = f.envelope();
        let subject = env
            .and_then(|e| e.subject.as_ref())
            .map(|b| String::from_utf8_lossy(b).to_string())
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

    let mut seen_present = false;

    // Fallbacks if base metadata not found
    if base.is_none() {
        tracing::debug!(%folder, uid, "body_in: no base metadata, trying search+refetch");
        let _ = session.noop().await;
        let q = format!("UID {}", uid);
        match session.uid_search(&q).await {
            Ok(list) => {
                tracing::debug!(%folder, uid, search_count=list.len(), search_list=?list, "body_in: search result");
                if list.iter().any(|&u| u == uid) { seen_present = true; }
                if seen_present {
                    // Re-select to ensure correct context
                    let _ = session.select(folder).await;
                    let _ = session.noop().await;
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

                    // Try multiple UID query forms
                    let uid_queries = vec![
                        uid.to_string(),
                        format!("{}:{}", uid, uid),
                        list.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(","),
                    ];
                    let mut refetch_count_total = 0usize;
                    let mut refetch_uids_all: Vec<Option<u32>> = Vec::new();
                    for uq in uid_queries {
                        tracing::debug!(%folder, uid, qry=%uq, "body_in: refetch with uid_query");
                        if let Ok(mut h2) = session.uid_fetch(&uq, "UID ENVELOPE FLAGS BODYSTRUCTURE").await {
                            let mut any = false;
                            while let Some(item) = h2.next().await {
                                let f = match item { Ok(v) => v, Err(e) => { tracing::debug!(%folder, uid, "body_in: refetch item err: {e}"); continue; } };
                                refetch_count_total += 1;
                                refetch_uids_all.push(f.uid);
                                let env = f.envelope();
                                let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
                                let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
                                let date = f.internal_date().map(|d| d.to_rfc3339());
                                let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
                                base = Some((subject, from, date, flags));
                                any = true;
                            }
                            drop(h2);
                            if any { break; }
                        }
                    }
                    tracing::debug!(%folder, uid, refetch_count=refetch_count_total, refetch_uids=?refetch_uids_all, "body_in: refetch aggregate result");

                    if base.is_none() {
                        if let Ok(mut s2) = session.fetch(uid.to_string(), "UID ENVELOPE FLAGS BODYSTRUCTURE").await {
                            let mut seq_count: usize = 0;
                            let mut seq_uids: Vec<Option<u32>> = Vec::new();
                            while let Some(item) = s2.next().await {
                                let f = item?; seq_count += 1; seq_uids.push(f.uid);
                                let env = f.envelope();
                                let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
                                let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
                                let date = f.internal_date().map(|d| d.to_rfc3339());
                                let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect();
                                base = Some((subject, from, date, flags));
                            }
                            tracing::debug!(%folder, uid, seq_count, seq_uids=?seq_uids, "body_in: refetch SEQ fetch result");
                            drop(s2);
                        }
                    }
                }
            }
            Err(e) => { tracing::debug!(%folder, uid, "body_in: search failed: {e}"); }
        }
    }

    if base.is_none() {
        tracing::debug!(%folder, uid, seen_present, "body_in: still no base metadata");
        let _ = session.logout().await;
        if seen_present { return Err(anyhow::anyhow!("uid_present_but_fetch_empty")); }
        return Ok(None);
    }

    let (subject, from, date, flags) = base.unwrap();

    // Try a list of likely sections
    let candidates = [
        "BODY.PEEK[TEXT]",
        "BODY.PEEK[1.TEXT]",
        "BODY.PEEK[1.1.TEXT]",
        "BODY.PEEK[1]",
        "BODY.PEEK[1.1]",
    ];

    let mut body: Option<Vec<u8>> = None;
    let mut chosen: &str = "";
    for sect in &candidates {
        tracing::debug!(%folder, uid, section=%sect, "body_in: trying section");
        let mut part = session.uid_fetch(&uid_str, sect).await?;
        let mut got = None;
        while let Some(item) = part.next().await {
            let f = item?;
            if let Some(b) = f.body() { if !b.is_empty() { got = Some(b.to_vec()); break; } }
        }
        drop(part);
        if let Some(v) = got { chosen = sect; body = Some(v); break; }
    }
    let mut body_text = String::new();
    if let Some(bytes) = body {
        let mut s = String::from_utf8(bytes.clone()).unwrap_or_else(|_| String::from_utf8_lossy(&bytes).to_string());
        if s.len() > 8000 { s.truncate(8000); s.push_str("\n...[truncated]..."); }
        tracing::debug!(%folder, uid, section=%chosen, len=s.len(), "body_in: got body");
        body_text = s;
    } else { tracing::debug!(%folder, uid, "body_in: no body found with fallbacks"); }

    let _ = session.logout().await;
    Ok(Some(MessageBodyMeta { uid, subject, from, date, size: None, flags, body: body_text }))
}

pub async fn list_attachments(
    _host: &str,
    _port: u16,
    _email: &str,
    _password: &str,
    _uid: u32,
) -> Result<Vec<AttachmentMeta>> {
    Ok(vec![])
}

pub async fn snapshot_state(host:&str, port:u16, email:&str, password:&str) -> Result<Vec<MessageState>> {
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    session.select("INBOX").await?;
    let mut fetches = session.uid_fetch("1:*", "UID FLAGS").await?;
    let mut out = Vec::new();
    while let Some(item) = fetches.next().await { let f = item?; if let Some(uid)=f.uid { let flags: Vec<String> = f.flags().map(|fl| format!("{:?}", fl)).collect(); out.push(MessageState{ uid, flags }); } }
    drop(fetches); let _ = session.logout().await; Ok(out)
}

pub async fn fetch_meta_for_uids(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    uids: &[u32],
) -> Result<Vec<NewMessageMeta>> {
    fetch_meta_for_uids_in(host, port, email, password, uids, "INBOX").await
}

pub async fn fetch_meta_for_uids_in(
    host: &str,
    port: u16,
    email: &str,
    password: &str,
    uids: &[u32],
    folder: &str,
) -> Result<Vec<NewMessageMeta>> {
    if uids.is_empty() { return Ok(vec![]); }
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    session.select(folder).await?;
    let mut ids: Vec<u32> = uids.to_vec();
    ids.sort_unstable();
    let seq = ids.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
    tracing::debug!(folder=%folder, %seq, count = ids.len(), "imap.fetch_meta_for_uids_in fetching");
    let mut out = Vec::new();
    let mut stream = session.uid_fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE").await?;
    while let Some(item) = stream.next().await {
        let f = item?;
        let uid = f.uid.unwrap_or(0);
        let env = f.envelope();
        let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
        let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
        let date = f.internal_date().map(|d| d.to_rfc3339());
        let size = None;
        out.push(NewMessageMeta { uid, subject, from, date, size });
    }
    drop(stream);
    let _ = session.logout().await;
    Ok(out)
}

pub async fn initial_snapshot_in(host:&str, port:u16, email:&str, password:&str, folder:&str) -> Result<SnapshotResult> {
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;

    let mailbox = session.select(folder).await?;
    let uidvalidity = mailbox.uid_validity.unwrap_or(0) as u32;

    let mut last_uid: u32 = 0;
    if let Ok(uids) = session.uid_search("ALL").await {
        for uid in uids { if uid > last_uid { last_uid = uid; } }
    }
    let _ = session.logout().await;
    Ok(SnapshotResult { uidvalidity, last_uid })
}

pub async fn fetch_new_since_in(host:&str, port:u16, email:&str, password:&str, last_uid:u32, folder:&str) -> Result<(u32, Vec<NewMessageMeta>)> {
    let tcp = TcpStream::connect((host, port)).await?;
    let tls = TlsConnector::builder().build()?;
    let tls = tokio_native_tls::TlsConnector::from(tls);
    let tls_stream = tls.connect(host, tcp).await?;
    let compat = tls_stream.compat();
    let client = async_imap::Client::new(compat);
    let mut session: Session<_> = client.login(email, password).await.map_err(|e| anyhow::anyhow!("login failed: {:?}", e))?;
    let mailbox = session.select(folder).await?;

    let all = session.uid_search("ALL").await?;
    let mut present_max: u32 = last_uid;
    for uid in &all { if *uid > present_max { present_max = *uid; } }
    let mut newer: Vec<u32> = all.into_iter().filter(|u| *u > last_uid).collect();
    newer.sort_unstable();
    tracing::debug!(folder=%folder, last_uid, uid_next = mailbox.uid_next.map(|v| v as u32), present_max, newer_count = newer.len(), "imap.fetch_new_since_in using ALL to compute newer UIDs");

    if newer.is_empty() {
        let _ = session.logout().await;
        return Ok((last_uid, Vec::new()));
    }

    let new_last_candidate = *newer.last().unwrap_or(&present_max);
    let seq = newer.iter().map(|u| u.to_string()).collect::<Vec<_>>().join(",");
    tracing::debug!(folder=%folder, %seq, count = newer.len(), "imap.fetch_new_since_in fetching exact newer UIDs");

    // Flush pending updates
    let _ = session.noop().await;

    let mut out = Vec::new();
    let mut fetches = session.uid_fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE").await?;
    while let Some(item) = fetches.next().await {
        let f = item?;
        if let Some(uid) = f.uid { if uid > last_uid {
            let env = f.envelope();
            let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
            let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
            let date = f.internal_date().map(|d| d.to_rfc3339());
            let size = None;
            out.push(NewMessageMeta { uid, subject, from, date, size });
        }}
    }
    drop(fetches);

    if out.is_empty() {
        // Try sequence-number FETCH for the same ids
        tracing::debug!(folder=%folder, %seq, "imap.fetch_new_since_in trying sequence FETCH fallback");
        let mut sfetch = session.fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE").await?;
        while let Some(item) = sfetch.next().await {
            let f = item?;
            if let Some(uid) = f.uid { if uid > last_uid {
                let env = f.envelope();
                let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
                let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
                let date = f.internal_date().map(|d| d.to_rfc3339());
                let size = None;
                out.push(NewMessageMeta { uid, subject, from, date, size });
            }}
        }
        drop(sfetch);
    }

    if out.is_empty() {
        // Retry once after short delay
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        let _ = session.noop().await;
        let mut fetches2 = session.uid_fetch(&seq, "UID ENVELOPE FLAGS INTERNALDATE").await?;
        while let Some(item) = fetches2.next().await {
            let f = item?;
            if let Some(uid) = f.uid { if uid > last_uid {
                let env = f.envelope();
                let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
                let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
                let date = f.internal_date().map(|d| d.to_rfc3339());
                let size = None;
                out.push(NewMessageMeta { uid, subject, from, date, size });
            }}
        }
        drop(fetches2);
    }

    if out.is_empty() {
        // Final fallback: range fetch between last_uid+1 and candidate
        let start = last_uid.saturating_add(1);
        if new_last_candidate >= start {
            let range = format!("{}:{}", start, new_last_candidate);
            tracing::debug!(folder=%folder, %range, "imap.fetch_new_since_in fallback range fetch");
            let mut fetches3 = session.uid_fetch(&range, "UID ENVELOPE FLAGS INTERNALDATE").await?;
            while let Some(item) = fetches3.next().await {
                let f = item?;
                if let Some(uid) = f.uid { if uid > last_uid {
                    let env = f.envelope();
                    let subject = env.and_then(|e| e.subject.as_ref()).map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
                    let from = env.and_then(|e| e.from.as_ref()).and_then(|v| v.get(0)).map(format_address).unwrap_or_default();
                    let date = f.internal_date().map(|d| d.to_rfc3339());
                    let size = None;
                    out.push(NewMessageMeta { uid, subject, from, date, size });
                }}
            }
            drop(fetches3);
        }
    }

    if out.is_empty() {
        // As a last resort, if SEARCH reported newer UIDs but FETCH yielded nothing, emit minimal items
        if !newer.is_empty() {
            for uid in newer.into_iter().filter(|u| *u > last_uid) {
                out.push(NewMessageMeta { uid, subject: String::new(), from: String::new(), date: None, size: None });
            }
        }
    }

    let new_last = if out.is_empty() { last_uid } else { out.iter().map(|m| m.uid).max().unwrap_or(last_uid) };
    tracing::debug!(folder=%folder, fetched = out.len(), new_last, "imap.fetch_new_since_in completed");
    let _ = session.logout().await;
    Ok((new_last, out))
}

fn format_address(a: &async_imap::imap_proto::Address<'_>) -> String {
    let name = a.name.as_ref().map(|n| String::from_utf8_lossy(n).trim().to_string()).unwrap_or_default();
    let mailbox = a.mailbox.as_ref().map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
    let host = a.host.as_ref().map(|b| String::from_utf8_lossy(b).to_string()).unwrap_or_default();
    let mut s = String::new();
    if !name.is_empty() { s.push_str(&name); s.push(' '); }
    if !mailbox.is_empty() || !host.is_empty() {
        s.push('<');
        s.push_str(&mailbox);
        if !host.is_empty() { s.push('@'); s.push_str(&host); }
        s.push('>');
    }
    s.trim().to_string()
}