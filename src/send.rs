//! The send bodies. Meta's answer to a send [accepts the message and promises nothing further](https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/send-messages):
//! what became of it arrives by webhook. No channel has an idempotency key, so the caller never
//! repeats a send whose outcome it does not know.

use serde::{Deserialize, Serialize};

/// The Graph API's refusal of a send: `{"error": {...}}`, on every channel.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Refused {
    pub error: GraphError,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GraphError {
    pub message: String,
    pub code: i64,
    #[serde(default)]
    pub error_subcode: Option<i64>,
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub fbtrace_id: Option<String>,
}

impl GraphError {
    /// What the code and subcode mean, where Meta's codes say it.
    #[must_use]
    pub const fn reason(&self) -> crate::Reason {
        crate::reason(self.code, self.error_subcode)
    }
}

/// WhatsApp: `POST /<PHONE_NUMBER_ID>/messages`.
pub mod whatsapp {
    use serde::Serialize;

    /// A free-form text, which WhatsApp admits within its service window, or a template, which it
    /// admits outside it.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Request {
        pub messaging_product: &'static str,
        pub recipient_type: &'static str,
        /// The user's number. With no leading `+`, WhatsApp prepends the business's country code.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to: Option<String>,
        /// The user's [business-scoped id](https://developers.facebook.com/documentation/business-messaging/whatsapp/business-scoped-user-ids),
        /// where the request names no number.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub recipient: Option<String>,
        #[serde(rename = "type")]
        pub kind: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub text: Option<Text>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub template: Option<Template>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image: Option<Media>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub video: Option<Media>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub audio: Option<Media>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub document: Option<Media>,
        /// The `wamid` this replies to.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub context: Option<Context>,
        /// Up to 512 characters repeated on this message's status webhooks. An echo, not an
        /// idempotency key.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub biz_opaque_callback_data: Option<String>,
    }

    /// A text body, at most 4,096 characters.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Text {
        pub body: String,
        pub preview_url: bool,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Context {
        pub message_id: String,
    }

    /// Media the number uploaded, by the id the upload returned
    /// ([image](https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/image-messages),
    /// [video](https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/video-messages),
    /// [audio](https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/audio-messages),
    /// [document](https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/document-messages)).
    /// A caption rides an image, a video or a document, at most 1,024 characters; a file's name a document alone.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Media {
        pub id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub caption: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub filename: Option<String>,
    }

    /// Which media message a send is.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum MediaKind {
        Image,
        Video,
        Audio,
        Document,
    }

    /// Meta's answer to an upload to `POST /<PHONE_NUMBER_ID>/media`: the id a message names the media by
    /// (<https://developers.facebook.com/documentation/business-messaging/whatsapp/business-phone-numbers/media>).
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
    pub struct Uploaded {
        pub id: String,
    }

    /// A template Meta approved, by name and language.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Template {
        pub name: String,
        pub language: Language,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        pub components: Vec<Component>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Language {
        pub code: String,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Component {
        #[serde(rename = "type")]
        pub kind: String,
        pub parameters: Vec<Parameter>,
    }

    /// A text parameter: named under `parameter_name`, or positional without it.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize)]
    pub struct Parameter {
        #[serde(rename = "type")]
        pub kind: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[expect(clippy::struct_field_names, reason = "the field carries Meta's own name for it")]
        pub parameter_name: Option<String>,
        pub text: String,
    }

    /// Whom a request is for.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum To {
        /// A phone number.
        Number(String),
        /// A business-scoped user id.
        User(String),
    }

    impl To {
        fn fields(self) -> (Option<String>, Option<String>) {
            match self {
                Self::Number(number) => (Some(number), None),
                Self::User(id) => (None, Some(id)),
            }
        }
    }

    /// Meta's acceptance of a send: the `wamid` it accepted the message under, which every later status names.
    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
    pub struct Accepted {
        pub messages: Vec<AcceptedMessage>,
    }

    #[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
    pub struct AcceptedMessage {
        pub id: String,
    }

    impl Accepted {
        /// The id of the one message the request sent.
        #[must_use]
        pub fn id(&self) -> Option<&str> {
            self.messages.first().map(|m| m.id.as_str())
        }
    }

    impl Request {
        #[must_use]
        pub fn text(to: To, body: String, context: Option<String>, callback: Option<String>) -> Self {
            let (to, recipient) = to.fields();
            Self {
                messaging_product: "whatsapp",
                recipient_type: "individual",
                to,
                recipient,
                kind: "text",
                text: Some(Text { body, preview_url: false }),
                template: None,
                image: None,
                video: None,
                audio: None,
                document: None,
                context: context.map(|message_id| Context { message_id }),
                biz_opaque_callback_data: callback,
            }
        }

        /// A media message of `kind` naming uploaded `media`. Audio carries no caption and only a document a file's
        /// name: the caller gives neither where the message has no field for it.
        #[must_use]
        pub fn media(to: To, kind: MediaKind, media: &Media, context: Option<String>) -> Self {
            let (to, recipient) = to.fields();
            let one = |of: MediaKind| (of == kind).then(|| media.clone());
            Self {
                messaging_product: "whatsapp",
                recipient_type: "individual",
                to,
                recipient,
                kind: match kind {
                    MediaKind::Image => "image",
                    MediaKind::Video => "video",
                    MediaKind::Audio => "audio",
                    MediaKind::Document => "document",
                },
                text: None,
                template: None,
                image: one(MediaKind::Image),
                video: one(MediaKind::Video),
                audio: one(MediaKind::Audio),
                document: one(MediaKind::Document),
                context: context.map(|message_id| Context { message_id }),
                biz_opaque_callback_data: None,
            }
        }

        #[must_use]
        pub fn template(to: To, template: Template, callback: Option<String>) -> Self {
            let (to, recipient) = to.fields();
            Self {
                messaging_product: "whatsapp",
                recipient_type: "individual",
                to,
                recipient,
                kind: "template",
                text: None,
                template: Some(template),
                image: None,
                video: None,
                audio: None,
                document: None,
                context: None,
                biz_opaque_callback_data: callback,
            }
        }
    }
}

/// Messenger, and Instagram through a Page: `POST /<PAGE_ID>/messages`. Instagram Login posts the
/// same body to `graph.instagram.com/<IG_ID>/messages`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageRequest {
    pub recipient: Recipient,
    pub messaging_type: MessagingType,
    pub message: PageMessage,
    /// The message this answers, the page's or the person's: Messenger's
    /// [reply](https://developers.facebook.com/documentation/business-messaging/messenger-platform/send-messages) and
    /// Instagram's [reply](https://developers.facebook.com/documentation/instagram-platform/instagram-api-with-instagram-login/messaging-api).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<ReplyTo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReplyTo {
    pub mid: String,
}

/// Meta's acceptance of a Page's send: the id it accepted the message under.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PageAccepted {
    pub message_id: String,
}

/// The id the platform scoped to the person: a Page-scoped or an Instagram-scoped id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recipient {
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MessagingType {
    Response,
    Update,
}

/// An attachment a Page sends with its bytes in the same request, as form fields beside the file (`filedata`), never
/// saved for reuse
/// ([saving assets](https://developers.facebook.com/documentation/business-messaging/messenger-platform/send-messages/saving-assets)):
/// the `message` field's value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageAttachment {
    pub attachment: Attachment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub kind: AttachmentKind,
    pub payload: AttachmentPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentKind {
    Image,
    Video,
    Audio,
    File,
}

/// The payload of an attachment whose bytes ride the request: nothing to name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttachmentPayload {}

impl PageAttachment {
    #[must_use]
    pub const fn of(kind: AttachmentKind) -> Self {
        Self { attachment: Attachment { kind, payload: AttachmentPayload {} } }
    }
}

/// A text body, at most 1,000 bytes on Instagram.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PageMessage {
    pub text: String,
}

impl PageRequest {
    /// A reply within the 24-hour window, answering the message `context` names where it names one.
    #[must_use]
    pub fn response(to: String, text: String, context: Option<String>) -> Self {
        Self {
            recipient: Recipient { id: to },
            messaging_type: MessagingType::Response,
            message: PageMessage { text },
            reply_to: context.map(|mid| ReplyTo { mid }),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)] // A test asserts; a failed expectation is the finding.
mod tests {
    use super::*;

    #[test]
    fn a_refusal_reads_as_the_graph_states_it_and_means_what_its_codes_say() {
        let closed = r#"{"error":{"message":"This message is sent outside of allowed window.","code":10,"error_subcode":2534022}}"#;
        let refused: Refused = serde_json::from_str(closed).unwrap();
        assert_eq!((refused.error.code, refused.error.error_subcode, refused.error.reason()), (10, Some(2_534_022), crate::Reason::WindowClosed));
        let unavailable = r#"{"error":{"message":"This person isn't available right now.","code":551,"error_subcode":1545041}}"#;
        assert_eq!(serde_json::from_str::<Refused>(unavailable).unwrap().error.reason(), crate::Reason::Unavailable);
        assert!(serde_json::from_str::<Refused>(r#"{"messaging_product":"whatsapp"}"#).is_err(), "an acceptance is no refusal");
    }

    // Meta's acceptances: https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/send-messages ("Response syntax") and
    // https://developers.facebook.com/documentation/business-messaging/messenger-platform/get-started.
    #[test]
    fn an_acceptance_names_the_id_every_later_status_names() {
        let whatsapp = r#"{"messaging_product":"whatsapp","contacts":[{"input":"+16505551234","wa_id":"16505551234"}],"messages":[{"id":"wamid.HBgLMTY1MDUwNzY1MjAVAgARGBI5QTNDQTVCM0Q0Q0Q2RTY3RTcA"}]}"#;
        assert_eq!(serde_json::from_str::<whatsapp::Accepted>(whatsapp).unwrap().id(), Some("wamid.HBgLMTY1MDUwNzY1MjAVAgARGBI5QTNDQTVCM0Q0Q0Q2RTY3RTcA"));
        let page =
            r#"{"recipient_id":"1008372609250235","message_id":"m_AG5Hz2Uq7tuwNEhXfYYKj8mJEM_QPpz5jdCK48PnKAjSdjfipqxqMvK8ma6AC8fplwlqLP_5cgXIbu7I3rBN0P"}"#;
        assert_eq!(
            serde_json::from_str::<PageAccepted>(page).unwrap().message_id,
            "m_AG5Hz2Uq7tuwNEhXfYYKj8mJEM_QPpz5jdCK48PnKAjSdjfipqxqMvK8ma6AC8fplwlqLP_5cgXIbu7I3rBN0P"
        );
        assert!(serde_json::from_str::<whatsapp::Accepted>(r#"{"error":{"message":"m","code":100}}"#).is_err(), "a refusal is no acceptance");
    }

    // Meta's examples: https://developers.facebook.com/documentation/business-messaging/whatsapp/messages/image-messages,
    // .../document-messages and .../audio-messages; https://developers.facebook.com/documentation/business-messaging/messenger-platform/send-messages/saving-assets.
    #[test]
    fn the_media_bodies_are_metas_documented_shapes_byte_for_byte() {
        let to = || whatsapp::To::Number("+16505551234".into());
        let media = |id: &str, caption: Option<&str>, filename: Option<&str>| whatsapp::Media {
            id: id.into(),
            caption: caption.map(str::to_owned),
            filename: filename.map(str::to_owned),
        };
        let image = whatsapp::Request::media(to(), whatsapp::MediaKind::Image, &media("1479537139650973", Some("The best succulent ever?"), None), None);
        assert_eq!(
            serde_json::to_string(&image).unwrap(),
            r#"{"messaging_product":"whatsapp","recipient_type":"individual","to":"+16505551234","type":"image","image":{"id":"1479537139650973","caption":"The best succulent ever?"}}"#
        );
        let document = whatsapp::Request::media(
            to(),
            whatsapp::MediaKind::Document,
            &media("1376223850470843", Some("Your order confirmation (PDF)"), Some("order_abc123.pdf")),
            Some("wamid.X".into()),
        );
        assert_eq!(
            serde_json::to_string(&document).unwrap(),
            r#"{"messaging_product":"whatsapp","recipient_type":"individual","to":"+16505551234","type":"document","document":{"id":"1376223850470843","caption":"Your order confirmation (PDF)","filename":"order_abc123.pdf"},"context":{"message_id":"wamid.X"}}"#
        );
        let audio = whatsapp::Request::media(to(), whatsapp::MediaKind::Audio, &media("1013859600285441", None, None), None);
        assert_eq!(
            serde_json::to_string(&audio).unwrap(),
            r#"{"messaging_product":"whatsapp","recipient_type":"individual","to":"+16505551234","type":"audio","audio":{"id":"1013859600285441"}}"#
        );
        assert_eq!(serde_json::from_str::<whatsapp::Uploaded>(r#"{"id":"1037543291543636"}"#).unwrap().id, "1037543291543636");
        assert_eq!(serde_json::to_string(&PageAttachment::of(AttachmentKind::Image)).unwrap(), r#"{"attachment":{"type":"image","payload":{}}}"#);
        assert_eq!(serde_json::to_string(&PageAttachment::of(AttachmentKind::File)).unwrap(), r#"{"attachment":{"type":"file","payload":{}}}"#);
    }

    #[test]
    fn the_send_bodies_are_metas_documented_shapes_byte_for_byte() {
        let text =
            whatsapp::Request::text(whatsapp::To::Number("+16505551234".into()), "It is, from Thursday.".into(), Some("wamid.X".into()), Some("01a1".into()));
        assert_eq!(
            serde_json::to_string(&text).unwrap(),
            r#"{"messaging_product":"whatsapp","recipient_type":"individual","to":"+16505551234","type":"text","text":{"body":"It is, from Thursday.","preview_url":false},"context":{"message_id":"wamid.X"},"biz_opaque_callback_data":"01a1"}"#
        );
        let template = whatsapp::Request::template(
            whatsapp::To::Number("+16505551234".into()),
            whatsapp::Template {
                name: "booking_confirmation".into(),
                language: whatsapp::Language { code: "en".into() },
                components: vec![whatsapp::Component {
                    kind: "body".into(),
                    parameters: vec![whatsapp::Parameter { kind: "text", parameter_name: Some("first_name".into()), text: "Amina".into() }],
                }],
            },
            None,
        );
        assert_eq!(
            serde_json::to_string(&template).unwrap(),
            r#"{"messaging_product":"whatsapp","recipient_type":"individual","to":"+16505551234","type":"template","template":{"name":"booking_confirmation","language":{"code":"en"},"components":[{"type":"body","parameters":[{"type":"text","parameter_name":"first_name","text":"Amina"}]}]}}"#
        );
        // A business-scoped id in `recipient`, with no `to`: https://developers.facebook.com/documentation/business-messaging/whatsapp/business-scoped-user-ids
        assert_eq!(
            serde_json::to_string(&whatsapp::Request::text(whatsapp::To::User("US.13491208655302741918".into()), "Yes".into(), None, None)).unwrap(),
            r#"{"messaging_product":"whatsapp","recipient_type":"individual","recipient":"US.13491208655302741918","type":"text","text":{"body":"Yes","preview_url":false}}"#
        );
        assert_eq!(
            serde_json::to_string(&PageRequest::response("1234567890123456".into(), "Yes".into(), None)).unwrap(),
            r#"{"recipient":{"id":"1234567890123456"},"messaging_type":"RESPONSE","message":{"text":"Yes"}}"#
        );
        assert_eq!(
            serde_json::to_string(&PageRequest::response("1234567890123456".into(), "Yes".into(), Some("m_IN".into()))).unwrap(),
            r#"{"recipient":{"id":"1234567890123456"},"messaging_type":"RESPONSE","message":{"text":"Yes"},"reply_to":{"mid":"m_IN"}}"#
        );
    }
}
