//! [Instagram messaging webhooks](https://developers.facebook.com/documentation/instagram-platform/webhooks/examples) under either login: `{"object": "instagram",
//! "entry": [{"id", "time", "messaging": [{"sender", "recipient", "timestamp", <the event>}]}]}`.
//! A message the account sent has `is_echo`. One the user deleted has `is_deleted`, and Meta's terms
//! oblige the app to delete it on that word.

use serde::Deserialize;

pub use crate::messenger::{Party, Referral};

/// One webhook envelope; see [`parse`] for the body.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Envelope {
    pub object: String,
    pub entry: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entry {
    pub id: String,
    pub time: i64,
    #[serde(default)]
    pub messaging: Vec<Messaging>,
}

/// One event. `timestamp` is in milliseconds, and one of the optional fields names the event.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Messaging {
    pub sender: Party,
    pub recipient: Party,
    pub timestamp: i64,
    #[serde(default)]
    pub message: Option<Message>,
    #[serde(default)]
    pub read: Option<Read>,
    #[serde(default)]
    pub message_edit: Option<MessageEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Message {
    pub mid: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub is_echo: Option<bool>,
    /// On an echo, the sending app, where the echo names one as Messenger's does.
    #[serde(default)]
    pub app_id: Option<serde_json::Value>,
    #[serde(default)]
    pub is_deleted: Option<bool>,
    #[serde(default)]
    pub is_unsupported: Option<bool>,
    #[serde(default)]
    pub reply_to: Option<serde_json::Value>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// The Click-to-Direct ad the person clicked, on their first message after it.
    #[serde(default)]
    pub referral: Option<Referral>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// A `messaging_seen` event.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Read {
    pub mid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MessageEdit {
    pub mid: String,
    pub text: String,
    pub num_edit: i64,
}

/// Reads a webhook body: Instagram Login sends an array of envelopes, a Page's route sends one.
///
/// # Errors
/// `serde_json`'s, where the body is neither.
pub fn parse(body: &[u8]) -> Result<Vec<Envelope>, serde_json::Error> {
    match serde_json::from_slice::<Vec<Envelope>>(body) {
        Ok(many) => Ok(many),
        Err(_) => serde_json::from_slice::<Envelope>(body).map(|one| vec![one]),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)] // A test asserts; a failed expectation is the finding.
mod tests {
    use super::*;

    // Meta's messaging envelope (https://developers.facebook.com/documentation/instagram-platform/webhooks/examples), with a message.
    const RECEIVED: &str = r#"[{"object":"instagram","entry":[{"id":"17841400000000000","time":1739321024000,"messaging":[{"sender":{"id":"1234567890123456"},"recipient":{"id":"17841400000000000"},"timestamp":1739321024000,"message":{"mid":"aWdfZAG1faXRlbToxOklHTWVzc2FnZAUlEOjE3ODQxNDAwMDAwMDAwMDA6MzQ1NjcyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMg","text":"Do you have it in blue?"}}]}]}]"#;

    #[test]
    fn metas_envelope_parses_as_an_array_or_alone_and_an_echo_a_deletion_and_a_seen_each_land() {
        let many = parse(RECEIVED.as_bytes()).unwrap();
        let m = &many[0].entry[0].messaging[0];
        assert_eq!((m.sender.id.as_str(), m.message.as_ref().unwrap().text.as_deref()), ("1234567890123456", Some("Do you have it in blue?")));
        let one = parse(RECEIVED.trim_start_matches('[').trim_end_matches(']').as_bytes()).unwrap();
        assert_eq!(one, many);
        let echo = RECEIVED.replace("\"text\":\"Do you have it in blue?\"", "\"text\":\"Yes\",\"is_echo\":true");
        assert_eq!(parse(echo.as_bytes()).unwrap()[0].entry[0].messaging[0].message.as_ref().unwrap().is_echo, Some(true));
        let deleted = RECEIVED.replace("\"text\":\"Do you have it in blue?\"", "\"is_deleted\":true");
        assert_eq!(parse(deleted.as_bytes()).unwrap()[0].entry[0].messaging[0].message.as_ref().unwrap().is_deleted, Some(true));
        let seen = r#"{"object":"instagram","entry":[{"id":"1","time":1,"messaging":[{"sender":{"id":"u"},"recipient":{"id":"a"},"timestamp":2,"read":{"mid":"m_1"}}]}]}"#;
        assert_eq!(parse(seen.as_bytes()).unwrap()[0].entry[0].messaging[0].read.as_ref().unwrap().mid, "m_1");
        assert!(parse(b"{}").is_err());
    }

    // Instagram's Click-to-Direct referral as Meta's examples list it. Unified Login's payloads take the Facebook Login
    // format and its message ids another form than Instagram Login's (the migration guide,
    // https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-instagram-login/migration-guide),
    // so a message id is read as the platform's own identifier, never decoded: here Messenger's example id.
    // https://developers.facebook.com/documentation/instagram-platform/webhooks/examples
    #[test]
    fn a_message_after_an_ad_names_the_ad_and_a_unified_login_message_id_reads_as_any_other() {
        let unified = r#"{"object":"instagram","entry":[{"id":"17841400000000000","time":1739321024000,"messaging":[{"sender":{"id":"1234567890123456"},"recipient":{"id":"17841400000000000"},"timestamp":1739321024000,"message":{"mid":"mid.1457764197618:41d102a3e1ae206a38","text":"Is it in blue?","referral":{"ref":"spring","ad_id":"120226305854810726","source":"ADS","type":"OPEN_THREAD","ads_context_data":{"ad_title":"Spring kangas"}}}}]}]}"#;
        let message = parse(unified.as_bytes()).unwrap()[0].entry[0].messaging[0].message.clone().unwrap();
        assert_eq!(message.mid, "mid.1457764197618:41d102a3e1ae206a38");
        let referral = message.referral.unwrap();
        assert!(referral.is_ad());
        assert_eq!((referral.reference.as_deref(), referral.ad_id.as_deref()), (Some("spring"), Some("120226305854810726")));
    }
}
