//! [Messenger webhooks](https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks): `{"object": "page", "entry": [{"id", "time", "messaging":
//! [{"sender", "recipient", "timestamp", <the event>}]}]}`. A message the Page sent, from any app,
//! arrives as a [`Message`] with `is_echo` set.

use serde::Deserialize;

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

/// A party to an event, by its Page-scoped id or the Page's id.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Party {
    pub id: String,
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
    pub delivery: Option<Delivery>,
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
    /// On an echo, the sending app; the Page inbox has an id of its own.
    #[serde(default)]
    pub app_id: Option<serde_json::Value>,
    #[serde(default)]
    pub reply_to: Option<ReplyTo>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub is_deleted: Option<bool>,
    /// The ad the person clicked, on their first message after it.
    #[serde(default)]
    pub referral: Option<Referral>,
}

/// [A Click-to-Messenger or Click-to-Direct ad's referral](https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks/webhook-events/messages):
/// `source` `ADS`, `type` `OPEN_THREAD`. Instagram's carries the same fields.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Referral {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub ad_id: Option<String>,
    /// The ad's `ref` parameter, where its owner set one.
    #[serde(rename = "ref", default)]
    pub reference: Option<String>,
}

impl Referral {
    /// Whether an ad opened the thread.
    #[must_use]
    pub fn is_ad(&self) -> bool {
        self.source.as_deref() == Some("ADS")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReplyTo {
    #[serde(default)]
    pub mid: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

/// Every message up to `watermark` was delivered, and `mids` names some of them.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Delivery {
    #[serde(default)]
    pub mids: Vec<String>,
    pub watermark: i64,
}

/// Every message up to `watermark` was read.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Read {
    pub watermark: i64,
}

/// A user's edit of their message, which Messenger allows five times.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MessageEdit {
    pub mid: String,
    pub text: String,
    pub num_edit: i64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)] // A test asserts; a failed expectation is the finding.
mod tests {
    use super::*;

    // Meta's envelope and text examples joined: https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks/webhook-events/messages
    const TEXT: &str = r#"{
  "object":"page",
  "entry":[
    {
      "id":"<PAGE_ID>",
      "time":1458692752478,
      "messaging":[
        {
          "sender":{ "id":"<PSID>" },
          "recipient":{ "id":"<PAGE_ID>" },
          "timestamp":1458692752478,
          "message":{
            "mid":"m_1457764197618:41d102a3e1ae206a38",
            "text":"hello, world!",
            "reply_to": { "mid":"m_1fTq8oLumEyIp3Q2MR-aY7IfLZDamVrALniheU", "is_self_reply": false }
          }
        }
      ]
    }
  ]
}"#;

    #[test]
    fn metas_text_example_parses_and_an_echo_a_delivery_a_read_and_an_edit_each_land() {
        let e: Envelope = serde_json::from_str(TEXT).unwrap();
        let m = &e.entry[0].messaging[0];
        assert_eq!((m.sender.id.as_str(), m.recipient.id.as_str()), ("<PSID>", "<PAGE_ID>"));
        let msg = m.message.as_ref().unwrap();
        assert_eq!(
            (msg.text.as_deref(), msg.reply_to.as_ref().and_then(|r| r.mid.as_deref())),
            (Some("hello, world!"), Some("m_1fTq8oLumEyIp3Q2MR-aY7IfLZDamVrALniheU"))
        );
        let echo = TEXT.replace("\"text\":\"hello, world!\",", "\"text\":\"hello, world!\", \"is_echo\": true, \"app_id\": 263902037430900,");
        let e: Envelope = serde_json::from_str(&echo).unwrap();
        assert_eq!(e.entry[0].messaging[0].message.as_ref().unwrap().is_echo, Some(true));
        let delivery = r#"{"object":"page","entry":[{"id":"1","time":1,"messaging":[{"sender":{"id":"u"},"recipient":{"id":"p"},"timestamp":1458668856253,"delivery":{"mids":["mid.1458668856218:ed81099e15d3f4f233"],"watermark":1458668856253}}]}]}"#;
        let e: Envelope = serde_json::from_str(delivery).unwrap();
        assert_eq!(e.entry[0].messaging[0].delivery.as_ref().unwrap().watermark, 1_458_668_856_253);
        let read = r#"{"object":"page","entry":[{"id":"1","time":1,"messaging":[{"sender":{"id":"u"},"recipient":{"id":"p"},"timestamp":2,"read":{"watermark":1458668856253}}]}]}"#;
        assert!(serde_json::from_str::<Envelope>(read).unwrap().entry[0].messaging[0].read.is_some());
        let edit = r#"{"object":"page","entry":[{"id":"1","time":1,"messaging":[{"sender":{"id":"u"},"recipient":{"id":"p"},"timestamp":2,"message_edit":{"mid":"m_1","text":"hello, world","num_edit":1}}]}]}"#;
        assert_eq!(serde_json::from_str::<Envelope>(edit).unwrap().entry[0].messaging[0].message_edit.as_ref().unwrap().num_edit, 1);
    }

    // Meta's "Message with Ads referral information" example in its envelope: https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks/webhook-events/messages
    #[test]
    fn a_first_message_after_an_ad_names_the_ad_that_opened_the_thread() {
        let from_ad = r#"{"object":"page","entry":[{"id":"<PAGE_ID>","time":1458692752478,"messaging":[{"sender":{"id":"<PSID>"},"recipient":{"id":"<PAGE_ID>"},"timestamp":1458692752478,"message":{"mid":"mid.1457764197618:41d102a3e1ae206a38","text":"hello, world!","referral":{"ref":"<REF_DATA_IF_SPECIFIED_IN_THE_AD>","ad_id":"<ID_OF_THE_AD>","source":"ADS","type":"OPEN_THREAD","ads_context_data":{"ad_title":"<TITLE_OF_THE_AD>","photo_url":"<URL>","video_url":"<URL>","post_id":"<ID_OF_THE_POST>","product_id":"<PRODUCT_ID>","flow_id":"<FLOW>"}}}}]}]}"#;
        let e: Envelope = serde_json::from_str(from_ad).unwrap();
        let referral = e.entry[0].messaging[0].message.as_ref().unwrap().referral.as_ref().unwrap();
        assert!(referral.is_ad());
        assert_eq!((referral.kind.as_deref(), referral.ad_id.as_deref()), (Some("OPEN_THREAD"), Some("<ID_OF_THE_AD>")));
        let e: Envelope = serde_json::from_str(TEXT).unwrap();
        assert_eq!(e.entry[0].messaging[0].message.as_ref().unwrap().referral, None);
    }
}
