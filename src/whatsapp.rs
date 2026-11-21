//! [WhatsApp webhooks](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/overview): `{"object": "whatsapp_business_account", "entry":
//! [{"id", "changes": [{"value", "field"}]}]}`, where a `messages` value carries the business
//! number's metadata with either inbound messages and their contacts or the statuses of sent
//! messages. A user is named by a [business-scoped id](https://developers.facebook.com/documentation/business-messaging/whatsapp/business-scoped-user-ids) (`user_id`,
//! `from_user_id`) beside or instead of the number (`wa_id`, `from`).

use serde::Deserialize;

/// The fields whose values this crate reads as a [`Value`].
const READ: [&str; 3] = ["messages", "smb_message_echoes", "user_preferences"];

/// One change of a webhook body: the Messaging account it is of, its field, and its value where the field is one this
/// crate reads - `messages`, `smb_message_echoes` or `user_preferences`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Changed {
    /// The entry's id: the Messaging account's.
    pub account: String,
    pub field: String,
    pub value: Option<Value>,
}

/// Every change of a webhook body, in the order sent.
///
/// # Errors
/// `serde_json`'s, where the body is no envelope or a value of a read field is no [`Value`].
pub fn changes(body: &[u8]) -> Result<Vec<Changed>, serde_json::Error> {
    #[derive(Deserialize)]
    struct Raw {
        entry: Vec<RawEntry>,
    }
    #[derive(Deserialize)]
    struct RawEntry {
        #[serde(default)]
        id: String,
        #[serde(default)]
        changes: Vec<RawChange>,
    }
    #[derive(Deserialize)]
    struct RawChange {
        field: String,
        value: serde_json::Value,
    }
    let raw: Raw = serde_json::from_slice(body)?;
    let mut out = Vec::new();
    for entry in raw.entry {
        for change in entry.changes {
            let value = if READ.contains(&change.field.as_str()) { Some(serde_json::from_value(change.value)?) } else { None };
            out.push(Changed { account: entry.id.clone(), field: change.field, value });
        }
    }
    Ok(out)
}

/// The values of a webhook body's changes on the fields this crate reads - `messages`, `smb_message_echoes` and
/// `user_preferences` - in the order sent. A change on any other field the app subscribes to (an account's, a
/// template's) is left out unread.
///
/// # Errors
/// `serde_json`'s, where the body is no envelope or a value of a read field is no [`Value`].
pub fn values(body: &[u8]) -> Result<Vec<Value>, serde_json::Error> {
    changes(body).map(|all| all.into_iter().filter_map(|c| c.value).collect())
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Envelope {
    pub object: String,
    pub entry: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Entry {
    /// The WhatsApp Business Account's id, its `waba_id`.
    pub id: String,
    pub changes: Vec<Change>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Change {
    pub value: Value,
    /// `messages` for what [`Value`] reads. Meta sends other fields for account, template and
    /// quality changes.
    pub field: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Value {
    pub messaging_product: String,
    pub metadata: Metadata,
    #[serde(default)]
    pub contacts: Vec<Contact>,
    #[serde(default)]
    pub messages: Vec<Message>,
    #[serde(default)]
    pub statuses: Vec<Status>,
    #[serde(default)]
    pub errors: Vec<Error>,
    /// A user's stop or resumption of marketing messages, on the `user_preferences` field.
    #[serde(default)]
    pub user_preferences: Vec<Preference>,
    /// [What the business sent from the WhatsApp Business app](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/smb_message_echoes),
    /// on the `smb_message_echoes` field.
    #[serde(default)]
    pub message_echoes: Vec<Echo>,
}

/// A message the business sent outside the Cloud API, to `to`, the number as the business entered it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Echo {
    pub to: String,
    #[serde(flatten)]
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Metadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

/// A user. Meta omits `wa_id` for a user with a username who has not exchanged a message with the
/// business number in 30 days and is not in its contact book.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Contact {
    #[serde(default)]
    pub wa_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    pub profile: Profile,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Profile {
    pub name: String,
    #[serde(default)]
    pub username: Option<String>,
}

/// An inbound message, or a system notice about the user. `from` is absent where
/// [`Contact::wa_id`] is.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Message {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub from_user_id: Option<String>,
    /// The `wamid`: a redelivery repeats it and a status names it.
    pub id: String,
    /// Unix seconds, as text.
    pub timestamp: String,
    #[serde(rename = "type")]
    pub kind: Kind,
    #[serde(default)]
    pub text: Option<Text>,
    #[serde(default)]
    pub context: Option<Context>,
    #[serde(default)]
    pub edit: Option<Edit>,
    #[serde(default)]
    pub revoke: Option<Revoke>,
    #[serde(default)]
    pub image: Option<Media>,
    #[serde(default)]
    pub video: Option<Media>,
    #[serde(default)]
    pub audio: Option<Media>,
    #[serde(default)]
    pub document: Option<Media>,
    #[serde(default)]
    pub sticker: Option<Media>,
    /// The ad the user clicked to send this message, on their first message after it.
    #[serde(default)]
    pub referral: Option<Referral>,
    /// What changed about the user, on a [`Kind::System`] message.
    #[serde(default)]
    pub system: Option<System>,
    /// [A place she shared](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/location).
    #[serde(default)]
    pub location: Option<Location>,
    /// [The people whose details she shared](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/contacts).
    #[serde(default)]
    pub contacts: Vec<Shared>,
    /// [A template's quick-reply button she pressed](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/button).
    #[serde(default)]
    pub button: Option<Button>,
    /// [Her answer to a reply-button or list message](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/interactive).
    #[serde(default)]
    pub interactive: Option<Interactive>,
    /// [Her reaction to a message](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/reaction).
    #[serde(default)]
    pub reaction: Option<Reaction>,
    /// [An order she placed from the catalogue](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/order).
    #[serde(default)]
    pub order: Option<Order>,
    #[serde(default)]
    pub errors: Vec<Error>,
}

impl Message {
    /// The media the message carries, where its kind is one: the bytes are fetched by `id`, its words
    /// beside it are `caption`.
    #[must_use]
    pub fn media(&self) -> Option<&Media> {
        match self.kind {
            Kind::Image => self.image.as_ref(),
            Kind::Video => self.video.as_ref(),
            Kind::Audio => self.audio.as_ref(),
            Kind::Document => self.document.as_ref(),
            Kind::Sticker => self.sticker.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Location {
    pub latitude: serde_json::Number,
    pub longitude: serde_json::Number,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
}

/// One person's shared details: the name as formatted and the numbers.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Shared {
    #[serde(default)]
    pub name: Option<SharedName>,
    #[serde(default)]
    pub phones: Vec<SharedPhone>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SharedName {
    #[serde(default)]
    pub formatted_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SharedPhone {
    #[serde(default)]
    pub phone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Button {
    pub text: String,
    #[serde(default)]
    pub payload: Option<String>,
}

/// An interactive answer: `button_reply` or `list_reply` by `type`, each the option's id and title.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Interactive {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub button_reply: Option<Chosen>,
    #[serde(default)]
    pub list_reply: Option<Chosen>,
}

/// A reaction: `emoji` is absent where she took her reaction back.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Reaction {
    pub message_id: String,
    #[serde(default)]
    pub emoji: Option<String>,
}

/// An order from the business's catalogue, with the words she sent beside it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Order {
    pub catalog_id: String,
    #[serde(default)]
    pub text: Option<String>,
    pub product_items: Vec<Ordered>,
}

/// One product of an order: the catalogue's id for it, how many, and its price in `currency` as the catalogue states it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Ordered {
    pub product_retailer_id: String,
    pub quantity: serde_json::Number,
    pub item_price: serde_json::Number,
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Chosen {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// [Where a media id's bytes are](https://developers.facebook.com/documentation/business-messaging/whatsapp/business-phone-numbers/media):
/// a URL valid for five minutes, downloaded under the same access token, with the bytes' type and SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct MediaUrl {
    pub url: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

/// Whether downloaded bytes are the ones a [`MediaUrl`] names by its digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Digested {
    /// The platform states no digest.
    Unstated,
    /// The bytes' SHA-256 is the stated one.
    Matches,
    /// The bytes' SHA-256 is another, or the statement reads as no SHA-256 digest.
    Differs,
}

impl MediaUrl {
    /// `bytes` against the digest stated: in base64 as Meta's examples write it, or in hex.
    #[must_use]
    pub fn digested(&self, bytes: &[u8]) -> Digested {
        use sha2::Digest as _;
        let Some(stated) = self.sha256.as_deref().map(str::trim) else { return Digested::Unstated };
        let digest = sha2::Sha256::digest(bytes);
        let hex = digest.iter().fold(String::with_capacity(64), |mut hex, b| {
            use std::fmt::Write as _;
            let _ = write!(hex, "{b:02x}");
            hex
        });
        if stated.trim_end_matches('=') == base64(&digest).trim_end_matches('=') || stated.eq_ignore_ascii_case(&hex) {
            Digested::Matches
        } else {
            Digested::Differs
        }
    }
}

/// `bytes` in base64's standard alphabet, padded (RFC 4648 section 4).
fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let [a, b, c] = [0, 1, 2].map(|k| chunk.get(k).copied().unwrap_or(0));
        let n = u32::from(a) << 16 | u32::from(b) << 8 | u32::from(c);
        for (k, shift) in [18, 12, 6, 0].into_iter().enumerate() {
            let sextet = usize::try_from((n >> shift) & 0x3f).unwrap_or(0);
            out.push(if k <= chunk.len() { char::from(ALPHABET.get(sextet).copied().unwrap_or(b'=')) } else { '=' });
        }
    }
    out
}

/// A media body: the platform holds its bytes under `id` for the business number to fetch.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Media {
    pub id: String,
    pub mime_type: String,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
}

/// [A Click-to-WhatsApp ad's referral](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/text).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Referral {
    /// `ad` for an ad.
    pub source_type: String,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    /// Omitted for an ad in WhatsApp Status.
    #[serde(default)]
    pub ctwa_clid: Option<String>,
}

/// [A system message](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/system):
/// the user's number or business-scoped id changed. `wa_id` is the new number, absent where it cannot
/// be shared; `previous_user_id` is absent where the earlier id is unavailable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct System {
    #[serde(rename = "type")]
    pub kind: SystemKind,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub wa_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub previous_user_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemKind {
    /// A new number, which `wa_id` carries.
    UserChangedNumber,
    /// A new business-scoped id, with no number.
    UserChangedUserId,
    #[serde(other)]
    Other,
}

/// [A user's marketing preference](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/user_preferences).
/// `timestamp` is Unix seconds as a number, unlike a message's.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Preference {
    #[serde(default)]
    pub wa_id: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    /// `marketing_messages`.
    pub category: String,
    pub value: Choice,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Choice {
    Stop,
    Resume,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Text,
    Image,
    Video,
    Audio,
    Document,
    Sticker,
    Location,
    Contacts,
    Interactive,
    Button,
    Reaction,
    Order,
    System,
    Edit,
    Revoke,
    Unsupported,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Text {
    pub body: String,
}

/// What the message answers, or that it was forwarded.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Context {
    #[serde(default)]
    pub from: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub forwarded: Option<bool>,
    #[serde(default)]
    pub frequently_forwarded: Option<bool>,
}

/// A user's edit of their message, which WhatsApp allows for fifteen minutes.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Edit {
    pub original_message_id: String,
    pub message: Box<Message>,
}

/// A user's revocation of their message, which WhatsApp allows for two days.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Revoke {
    pub original_message_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Status {
    /// The sent message's `wamid`.
    pub id: String,
    #[expect(clippy::struct_field_names, reason = "the field carries Meta's own name for it")]
    pub status: StatusKind,
    pub timestamp: String,
    #[serde(default)]
    pub recipient_id: Option<String>,
    #[serde(default)]
    pub recipient_user_id: Option<String>,
    #[serde(default)]
    pub biz_opaque_callback_data: Option<String>,
    #[serde(default)]
    pub pricing: Option<Pricing>,
    #[serde(default)]
    pub errors: Vec<Error>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusKind {
    Sent,
    Delivered,
    Read,
    Failed,
    Played,
    #[serde(other)]
    Other,
}

/// [What Meta charged](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/status), in its own words: `type` `regular` where the
/// message is billable - a service message past the number's monthly free tier among them -,
/// `free_customer_service` within that tier. On `sent`, and on one of `delivered` and `read`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Pricing {
    #[serde(default)]
    pub billable: Option<bool>,
    #[expect(clippy::struct_field_names, reason = "the field carries Meta's own name for it")]
    pub pricing_model: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub category: String,
    /// `paid_exempt` for a government or non-profit beyond the free tier.
    #[serde(default)]
    pub subtype: Option<String>,
}

/// A platform error on a message, a status or a value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Error {
    pub code: i64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    #[expect(clippy::struct_field_names, reason = "the field carries Meta's own name for it")]
    pub error_data: Option<ErrorData>,
}

impl Error {
    /// What the code means, where Meta's code says it.
    #[must_use]
    pub const fn reason(&self) -> crate::Reason {
        crate::reason(self.code, None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ErrorData {
    #[serde(default)]
    pub details: Option<String>,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::indexing_slicing, clippy::panic)] // A test asserts; a failed expectation is the finding.
mod tests {
    use super::*;

    // Meta's example of a sent status: https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/status
    const SENT: &str = r#"{
  "object": "whatsapp_business_account",
  "entry": [
    {
      "id": "102290129340398",
      "changes": [
        {
          "value": {
            "messaging_product": "whatsapp",
            "metadata": {
              "display_phone_number": "15550783881",
              "phone_number_id": "106540352242922"
            },
            "statuses": [
              {
                "id": "wamid.HBgLMTY1MDM4Nzk0MzkVAgASGBQzQUFERjg0NDEzNDdFODU3MUMxMAA=",
                "status": "sent",
                "timestamp": "1750030073",
                "recipient_id": "16505551234"
              }
            ]
          },
          "field": "messages"
        }
      ]
    }
  ]
}"#;

    // Meta's inbound text example (https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/text) with the
    // business-scoped fields beside the number and a field this crate does not know.
    const TEXT: &str = r#"{
  "object": "whatsapp_business_account",
  "entry": [{
    "id": "102290129340398",
    "changes": [{
      "value": {
        "messaging_product": "whatsapp",
        "metadata": {"display_phone_number": "15550783881", "phone_number_id": "106540352242922"},
        "contacts": [{"profile": {"name": "Sheena Nelson", "username": "sheena"}, "wa_id": "16505551234", "user_id": "US.13491208655302741918", "identity_key_hash": "DF2lS5v2W6x="}],
        "messages": [{
          "from": "+16505551234",
          "from_user_id": "US.13491208655302741918",
          "id": "wamid.HBgLMTY1MDM4Nzk0MzkVAgASGBQzQUFERjg0NDEzNDdFODU3MUMxMAA=",
          "timestamp": "1739321024",
          "type": "text",
          "text": {"body": "Is it available in another color?"},
          "a_field_meta_adds_next_quarter": {"x": 1}
        }]
      },
      "field": "messages"
    }]
  }]
}"#;

    #[test]
    fn metas_examples_parse_and_an_unknown_field_or_word_never_refuses_the_call() {
        let e: Envelope = serde_json::from_str(SENT).unwrap();
        assert_eq!(e.object, "whatsapp_business_account");
        let s = &e.entry[0].changes[0].value.statuses[0];
        assert_eq!((s.status, s.recipient_id.as_deref(), s.pricing.as_ref()), (StatusKind::Sent, Some("16505551234"), None));
        let e: Envelope = serde_json::from_str(TEXT).unwrap();
        let v = &e.entry[0].changes[0].value;
        assert_eq!(v.contacts[0].user_id.as_deref(), Some("US.13491208655302741918"));
        assert_eq!(v.contacts[0].profile.username.as_deref(), Some("sheena"));
        let m = &v.messages[0];
        assert_eq!((m.kind, m.text.as_ref().map(|t| t.body.as_str())), (Kind::Text, Some("Is it available in another color?")));
        assert_eq!(m.from_user_id.as_deref(), Some("US.13491208655302741918"));
        // A status word, a type and a pricing the crate does not know: the fact still lands as stated.
        let other = SENT.replace("\"status\": \"sent\"", "\"status\": \"held_for_quality_assessment\"");
        let e: Envelope = serde_json::from_str(&other).unwrap();
        assert_eq!(e.entry[0].changes[0].value.statuses[0].status, StatusKind::Other);
        let priced = SENT.replace(
            "\"recipient_id\": \"16505551234\"",
            "\"recipient_id\": \"16505551234\", \"pricing\": {\"billable\": true, \"pricing_model\": \"PMP\", \"type\": \"regular\", \"category\": \"service\"}, \"errors\": [{\"code\": 131047, \"title\": \"Re-engagement message\", \"message\": \"Re-engagement message\", \"error_data\": {\"details\": \"More than 24 hours\"}}]",
        );
        let e: Envelope = serde_json::from_str(&priced).unwrap();
        let s = &e.entry[0].changes[0].value.statuses[0];
        assert_eq!(s.pricing.as_ref().map(|p| (p.kind.as_str(), p.category.as_str())), Some(("regular", "service")));
        assert_eq!((s.errors[0].code, s.errors[0].reason()), (131_047, crate::Reason::WindowClosed));
        let number_absent = TEXT.replace("\"wa_id\": \"16505551234\", ", "").replace("\"from\": \"+16505551234\",", "");
        let e: Envelope = serde_json::from_str(&number_absent).unwrap();
        assert_eq!(e.entry[0].changes[0].value.messages[0].from, None, "a username's message carries no number past the lookback");
        let edited = TEXT.replace("\"type\": \"text\"", "\"type\": \"edit\"").replace(
            "\"text\": {\"body\": \"Is it available in another color?\"}",
            "\"edit\": {\"original_message_id\": \"wamid.X\", \"message\": {\"id\": \"wamid.Y\", \"timestamp\": \"1739321100\", \"type\": \"text\", \"text\": {\"body\": \"In blue?\"}}}",
        );
        let e: Envelope = serde_json::from_str(&edited).unwrap();
        let m = &e.entry[0].changes[0].value.messages[0];
        assert_eq!((m.kind, m.edit.as_ref().map(|x| x.original_message_id.as_str())), (Kind::Edit, Some("wamid.X")));
        assert!(serde_json::from_str::<Envelope>("{\"object\": \"whatsapp_business_account\"}").is_err(), "no entries is not a body");
    }

    // Meta's image example: https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/image
    const IMAGE: &str = r#"{"object":"whatsapp_business_account","entry":[{"id":"102290129340398","changes":[{"value":{"messaging_product":"whatsapp","metadata":{"display_phone_number":"15550783881","phone_number_id":"106540352242922"},"contacts":[{"profile":{"name":"Sheena Nelson"},"wa_id":"16505551234"}],"messages":[{"from":"16505551234","id":"wamid.HBgLMTY1MDM4Nzk0MzkVAgASGBQzQTRBNjU5OUFFRTAzODEwMTQ0RgA=","timestamp":"1744344496","type":"image","image":{"caption":"Taj Mahal","mime_type":"image/jpeg","sha256":"SfInY0gGKTsJlUWbwxC1k+FAD0FZHvzwfpvO0zX0GUI=","id":"1003383421387256","url":"https://lookaside.fbsbx.com/whatsapp_business/attachments/?mid=133..."}}]},"field":"messages"}]}]}"#;

    // Meta's two system examples: https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/messages/system
    const NUMBER_CHANGED: &str = r#"{"object":"whatsapp_business_account","entry":[{"id":"102290129340398","changes":[{"value":{"messaging_product":"whatsapp","metadata":{"display_phone_number":"15550783881","phone_number_id":"106540352242922"},"messages":[{"from":"16505551234","id":"wamid.HBgLMTk4MzU1NTE5NzQVAgASGAoxMTgyMDg2MjY3AA==","timestamp":"1750269342","system":{"body":"User Sheena Nelson changed from 16505551234 to 12195555358","wa_id":"12195555358","type":"user_changed_number","user_id":"US.13491208655302741918","parent_user_id":"US.ENT.506847293015824","previous_user_id":"US.10230984751029384756","previous_parent_user_id":"US.ENT.918273645500112"},"type":"system"}]},"field":"messages"}]}]}"#;
    const ID_CHANGED: &str = r#"{"object":"whatsapp_business_account","entry":[{"id":"102290129340398","changes":[{"value":{"messaging_product":"whatsapp","metadata":{"display_phone_number":"15550783881","phone_number_id":"106540352242922"},"messages":[{"id":"wamid.HBgLMTk4MzU1NTE5NzQVAgASGAoxMTgyMDg2MjY3AA==","timestamp":"1750269342","system":{"body":"User changed from US.10230984751029384756 to US.13491208655302741918","type":"user_changed_user_id","user_id":"US.13491208655302741918","parent_user_id":"US.ENT.506847293015824","previous_user_id":"US.10230984751029384756","previous_parent_user_id":"US.ENT.918273645500112"},"type":"system"}]},"field":"messages"}]}]}"#;

    // Meta's user_preferences syntax with the page's example values: https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/user_preferences
    const STOPPED: &str = r#"{"object":"whatsapp_business_account","entry":[{"id":"102290129340398","changes":[{"value":{"messaging_product":"whatsapp","metadata":{"display_phone_number":"15550783881","phone_number_id":"106540352242922"},"contacts":[{"profile":{"name":"Sheena Nelson"},"wa_id":"16505551234","user_id":"US.13491208655302741918"}],"user_preferences":[{"wa_id":"16505551234","user_id":"US.13491208655302741918","detail":"User requested to stop marketing messages","category":"marketing_messages","value":"stop","timestamp":1739321024}]},"field":"user_preferences"}]}]}"#;

    // An account's change beside a message: https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/reference/account_update
    #[test]
    fn a_change_on_a_field_this_crate_does_not_read_is_left_out_and_a_message_beside_it_read() {
        let body = r#"{"object":"whatsapp_business_account","entry":[{"id":"102290129340398","changes":[{"field":"account_update","value":{"phone_number":"15550783881","event":"VERIFIED_ACCOUNT"}},{"field":"messages","value":{"messaging_product":"whatsapp","metadata":{"display_phone_number":"15550783881","phone_number_id":"106540352242922"},"messages":[{"from":"16505551234","id":"wamid.1","timestamp":"1739321024","type":"text","text":{"body":"Hi"}}]}}]}]}"#;
        let read = values(body.as_bytes()).unwrap();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].messages[0].text.as_ref().map(|t| t.body.as_str()), Some("Hi"));
        assert!(
            values(br#"{"object":"whatsapp_business_account","entry":[{"id":"1","changes":[{"field":"messages","value":{}}]}]}"#).is_err(),
            "a read field's value is read whole"
        );
    }

    #[test]
    fn a_media_body_with_its_caption_a_changed_number_or_id_a_stop_and_an_ads_referral_each_read_as_meta_states_them() {
        let e: Envelope = serde_json::from_str(IMAGE).unwrap();
        let m = &e.entry[0].changes[0].value.messages[0];
        let media = m.media().unwrap();
        assert_eq!((media.id.as_str(), media.mime_type.as_str(), media.caption.as_deref()), ("1003383421387256", "image/jpeg", Some("Taj Mahal")));
        assert_eq!(m.text, None, "the words ride the media as its caption");
        let as_document = IMAGE.replace("\"type\":\"image\",\"image\"", "\"type\":\"document\",\"document\"");
        let e: Envelope = serde_json::from_str(&as_document).unwrap();
        assert_eq!(e.entry[0].changes[0].value.messages[0].media().map(|m| m.id.as_str()), Some("1003383421387256"));
        let text: Envelope = serde_json::from_str(TEXT).unwrap();
        assert_eq!(text.entry[0].changes[0].value.messages[0].media(), None);

        let e: Envelope = serde_json::from_str(NUMBER_CHANGED).unwrap();
        let system = e.entry[0].changes[0].value.messages[0].system.as_ref().unwrap();
        assert_eq!((system.kind, system.wa_id.as_deref()), (SystemKind::UserChangedNumber, Some("12195555358")));
        assert_eq!((system.user_id.as_deref(), system.previous_user_id.as_deref()), (Some("US.13491208655302741918"), Some("US.10230984751029384756")));
        let e: Envelope = serde_json::from_str(ID_CHANGED).unwrap();
        let m = &e.entry[0].changes[0].value.messages[0];
        assert_eq!((m.kind, m.from.as_deref()), (Kind::System, None), "no number with a changed id");
        let system = m.system.as_ref().unwrap();
        assert_eq!((system.kind, system.wa_id.as_deref()), (SystemKind::UserChangedUserId, None));
        let unknown = ID_CHANGED.replace("user_changed_user_id", "customer_identity_changed");
        assert_eq!(serde_json::from_str::<Envelope>(&unknown).unwrap().entry[0].changes[0].value.messages[0].system.as_ref().unwrap().kind, SystemKind::Other);

        let e: Envelope = serde_json::from_str(STOPPED).unwrap();
        let change = &e.entry[0].changes[0];
        assert_eq!(change.field, "user_preferences");
        let p = &change.value.user_preferences[0];
        assert_eq!(
            (p.value, p.category.as_str(), p.user_id.as_deref(), p.timestamp),
            (Choice::Stop, "marketing_messages", Some("US.13491208655302741918"), 1_739_321_024)
        );
        let resumed: Envelope = serde_json::from_str(&STOPPED.replace("\"value\":\"stop\"", "\"value\":\"resume\"")).unwrap();
        assert_eq!(resumed.entry[0].changes[0].value.user_preferences[0].value, Choice::Resume);

        let from_ad = TEXT.replace(
            "\"text\": {\"body\": \"Is it available in another color?\"},",
            "\"text\": {\"body\": \"Is it available in another color?\"}, \"referral\": {\"source_url\": \"https://fb.me/x\", \"source_id\": \"120226305854810726\", \"source_type\": \"ad\", \"headline\": \"Chat with us\", \"welcome_message\": {\"text\": \"Hi there!\"}},",
        );
        let e: Envelope = serde_json::from_str(&from_ad).unwrap();
        let referral = e.entry[0].changes[0].value.messages[0].referral.as_ref().unwrap();
        assert_eq!(
            (referral.source_type.as_str(), referral.source_id.as_deref(), referral.ctwa_clid.as_deref()),
            ("ad", Some("120226305854810726"), None),
            "a Status ad carries no click id"
        );

        let in_tier = SENT.replace(
            "\"recipient_id\": \"16505551234\"",
            "\"recipient_id\": \"16505551234\", \"pricing\": {\"pricing_model\": \"PMP\", \"type\": \"free_customer_service\", \"category\": \"service\"}",
        );
        let e: Envelope = serde_json::from_str(&in_tier).unwrap();
        let pricing = e.entry[0].changes[0].value.statuses[0].pricing.as_ref().unwrap();
        assert_eq!(
            (pricing.kind.as_str(), pricing.category.as_str(), pricing.billable, pricing.subtype.as_deref()),
            ("free_customer_service", "service", None, None)
        );
        let exempt = in_tier.replace("\"type\": \"free_customer_service\"", "\"type\": \"regular\", \"subtype\": \"paid_exempt\"");
        let e: Envelope = serde_json::from_str(&exempt).unwrap();
        assert_eq!(e.entry[0].changes[0].value.statuses[0].pricing.as_ref().unwrap().subtype.as_deref(), Some("paid_exempt"));
    }

    // The media lookup's response as Meta's page writes it, its digest the SHA-256 of "abc" (FIPS 180-2's first
    // vector): https://developers.facebook.com/documentation/business-messaging/whatsapp/business-phone-numbers/media
    #[test]
    fn a_downloads_bytes_match_the_digest_its_lookup_states_in_base64_or_hex_and_differ_from_any_other() {
        let lookup = |sha256: &str| -> MediaUrl {
            serde_json::from_str(&format!(
                r#"{{"messaging_product":"whatsapp","url":"https://lookaside.fbsbx.com/whatsapp_business/attachments/?mid=1","mime_type":"image/jpeg","sha256":{sha256},"file_size":"3","id":"1003383421387256"}}"#
            ))
            .unwrap()
        };
        assert_eq!(lookup(r#""ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=""#).digested(b"abc"), Digested::Matches);
        assert_eq!(lookup(r#""ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0""#).digested(b"abc"), Digested::Matches, "unpadded");
        assert_eq!(lookup(r#""BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD""#).digested(b"abc"), Digested::Matches, "in hex");
        assert_eq!(lookup(r#""ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0=""#).digested(b"abd"), Digested::Differs);
        assert_eq!(lookup(r#""a1b2c3d4e5f6...""#).digested(b"abc"), Digested::Differs, "no digest it can be");
        assert_eq!(lookup("null").digested(b"abc"), Digested::Unstated);
        assert_eq!(
            (base64(b""), base64(b"f"), base64(b"fo"), base64(b"foo")),
            (String::new(), "Zg==".into(), "Zm8=".into(), "Zm9v".into()),
            "RFC 4648's vectors"
        );
    }
}
