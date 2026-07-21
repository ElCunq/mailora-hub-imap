use anyhow::Result;
use reqwest::Client;
use crate::mailcow::models::MailcowInstance;
use crate::services::crypto::decrypt_secret;

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredDomain {
    pub name: String,
    pub active: bool,
    pub quota: Option<i64>,
    pub mailbox_limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiscoveredMailbox {
    pub address: String,
    pub local_part: String,
    pub display_name: Option<String>,
    pub active: bool,
    pub quota: Option<i64>,
    pub used_quota: Option<i64>,
}

pub struct MailcowClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl MailcowClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let mut base_url = base_url.into();
        if base_url.ends_with('/') {
            base_url.pop();
        }
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url,
            api_key: api_key.into(),
        }
    }

    pub fn from_instance(instance: &MailcowInstance) -> Self {
        let api_key = decrypt_secret(&instance.api_key_encrypted)
            .unwrap_or_else(|_| instance.api_key_encrypted.clone());
        Self::new(&instance.base_url, api_key)
    }

    pub async fn fetch_domains(&self) -> Result<Vec<DiscoveredDomain>> {
        let url = format!("{}/api/v1/get/domain/all", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "Mailcow API GET /api/v1/get/domain/all failed with status {}: {}",
                status,
                text
            );
        }

        let val: serde_json::Value = resp.json().await?;
        Self::parse_domains_value(&val)
    }

    pub async fn fetch_mailboxes(&self) -> Result<Vec<DiscoveredMailbox>> {
        let url = format!("{}/api/v1/get/mailbox/all", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "Mailcow API GET /api/v1/get/mailbox/all failed with status {}: {}",
                status,
                text
            );
        }

        let val: serde_json::Value = resp.json().await?;
        Self::parse_mailboxes_value(&val)
    }

    pub fn parse_domains_value(val: &serde_json::Value) -> Result<Vec<DiscoveredDomain>> {
        let mut domains = Vec::new();
        if let Some(arr) = val.as_array() {
            for item in arr {
                if let Some(name) = item
                    .get("domain_name")
                    .or_else(|| item.get("domain"))
                    .and_then(|v| v.as_str())
                {
                    let active = bool_from_val(item.get("active")).unwrap_or(true);
                    let quota = i64_from_val(item.get("def_new_mailbox_quota"))
                        .or_else(|| i64_from_val(item.get("quota")));
                    let mailbox_limit = i64_from_val(item.get("max_num_mboxes_for_domain"));
                    domains.push(DiscoveredDomain {
                        name: name.to_string(),
                        active,
                        quota,
                        mailbox_limit,
                    });
                }
            }
        } else if let Some(map) = val.as_object() {
            if let Some("error") = map.get("type").and_then(|v| v.as_str()) {
                let msg = map
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                anyhow::bail!("Mailcow API error: {}", msg);
            }
            for (key, item) in map {
                if let Some(_obj) = item.as_object() {
                    let name = item
                        .get("domain_name")
                        .or_else(|| item.get("domain"))
                        .and_then(|v| v.as_str())
                        .unwrap_or(key);
                    let active = bool_from_val(item.get("active")).unwrap_or(true);
                    let quota = i64_from_val(item.get("def_new_mailbox_quota"))
                        .or_else(|| i64_from_val(item.get("quota")));
                    let mailbox_limit = i64_from_val(item.get("max_num_mboxes_for_domain"));
                    domains.push(DiscoveredDomain {
                        name: name.to_string(),
                        active,
                        quota,
                        mailbox_limit,
                    });
                }
            }
        }
        Ok(domains)
    }

    pub fn parse_mailboxes_value(val: &serde_json::Value) -> Result<Vec<DiscoveredMailbox>> {
        let mut mailboxes = Vec::new();
        if let Some(arr) = val.as_array() {
            for item in arr {
                if let Some(address) = item.get("username").and_then(|v| v.as_str()) {
                    let local_part = match item.get("local_part").and_then(|v| v.as_str()) {
                        Some(lp) => lp.to_string(),
                        None => address.split('@').next().unwrap_or("").to_string(),
                    };
                    let display_name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let active = bool_from_val(item.get("active")).unwrap_or(true);
                    let quota = i64_from_val(item.get("quota"));
                    let used_quota = item
                        .get("attributes")
                        .and_then(|attr| attr.get("quota_used"))
                        .and_then(|qu| i64_from_val(Some(qu)))
                        .or_else(|| i64_from_val(item.get("quota_used")));

                    mailboxes.push(DiscoveredMailbox {
                        address: address.to_string(),
                        local_part,
                        display_name,
                        active,
                        quota,
                        used_quota,
                    });
                }
            }
        } else if let Some(map) = val.as_object() {
            if let Some("error") = map.get("type").and_then(|v| v.as_str()) {
                let msg = map
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                anyhow::bail!("Mailcow API error: {}", msg);
            }
            for (key, item) in map {
                if let Some(_obj) = item.as_object() {
                    let address = item.get("username").and_then(|v| v.as_str()).unwrap_or(key);
                    let local_part = match item.get("local_part").and_then(|v| v.as_str()) {
                        Some(lp) => lp.to_string(),
                        None => address.split('@').next().unwrap_or("").to_string(),
                    };
                    let display_name = item
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let active = bool_from_val(item.get("active")).unwrap_or(true);
                    let quota = i64_from_val(item.get("quota"));
                    let used_quota = item
                        .get("attributes")
                        .and_then(|attr| attr.get("quota_used"))
                        .and_then(|qu| i64_from_val(Some(qu)))
                        .or_else(|| i64_from_val(item.get("quota_used")));

                    mailboxes.push(DiscoveredMailbox {
                        address: address.to_string(),
                        local_part,
                        display_name,
                        active,
                        quota,
                        used_quota,
                    });
                }
            }
        }
        Ok(mailboxes)
    }
}

fn bool_from_val(v: Option<&serde_json::Value>) -> Option<bool> {
    match v {
        Some(serde_json::Value::Bool(b)) => Some(*b),
        Some(serde_json::Value::Number(n)) => Some(n.as_i64().unwrap_or(0) != 0),
        Some(serde_json::Value::String(s)) => match s.as_str() {
            "1" | "true" | "yes" => Some(true),
            "0" | "false" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

fn i64_from_val(v: Option<&serde_json::Value>) -> Option<i64> {
    match v {
        Some(serde_json::Value::Number(n)) => n.as_i64(),
        Some(serde_json::Value::String(s)) => s.parse::<i64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_domains_value_array() {
        let json_val = json!([
            {
                "domain_name": "example.com",
                "active": 1,
                "def_new_mailbox_quota": 2048,
                "max_num_mboxes_for_domain": 50
            },
            {
                "domain_name": "disabled.org",
                "active": "0",
                "quota": "1024"
            }
        ]);

        let domains = MailcowClient::parse_domains_value(&json_val).unwrap();
        assert_eq!(domains.len(), 2);
        assert_eq!(domains[0].name, "example.com");
        assert_eq!(domains[0].active, true);
        assert_eq!(domains[0].quota, Some(2048));
        assert_eq!(domains[0].mailbox_limit, Some(50));

        assert_eq!(domains[1].name, "disabled.org");
        assert_eq!(domains[1].active, false);
        assert_eq!(domains[1].quota, Some(1024));
    }

    #[test]
    fn test_parse_mailboxes_value() {
        let json_val = json!([
            {
                "username": "user@example.com",
                "name": "John Doe",
                "active": 1,
                "quota": 5000,
                "attributes": {
                    "quota_used": 1234
                }
            }
        ]);

        let mailboxes = MailcowClient::parse_mailboxes_value(&json_val).unwrap();
        assert_eq!(mailboxes.len(), 1);
        assert_eq!(mailboxes[0].address, "user@example.com");
        assert_eq!(mailboxes[0].local_part, "user");
        assert_eq!(mailboxes[0].display_name.as_deref(), Some("John Doe"));
        assert_eq!(mailboxes[0].quota, Some(5000));
        assert_eq!(mailboxes[0].used_quota, Some(1234));
    }
}
