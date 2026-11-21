//! Meta's messaging protocol as one library for its three assets: a WhatsApp business number, a
//! Facebook Page and an Instagram professional account. [`signature`] proves a webhook call,
//! [`handshake`] answers the subscription, [`whatsapp`], [`messenger`] and [`instagram`] read the
//! webhook bodies, and [`send`] writes the bodies a send posts. It has no transport: the caller
//! hands it bytes and posts what it writes.
//!
//! Reading is strict in the type of each field it takes and blind to every other, because Meta adds
//! fields to live payloads without a new API version. A word outside a closed set reads as that
//! set's `Other`, so no call is refused over a word newer than this crate, and the caller keeps the
//! raw body. Writing is exact: each send body holds only the fields Meta documents.

use std::fmt;
use std::time::Duration;

pub mod handshake;
pub mod instagram;
pub mod messenger;
pub mod send;
pub mod signature;
pub mod whatsapp;

/// How long a call waits for its answer before the platform takes it as failed: Messenger's
/// [five seconds](https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks), the one bound Meta states on any of the three channels.
pub const ANSWER_WITHIN: Duration = Duration::from_secs(5);

/// Why a webhook call's signature proves nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unproven {
    /// The app secret is empty, so any body signed under it would prove; nothing is compared.
    EmptySecret,
    /// The header is not `sha256=<64 hex digits>`, or the body cannot take the signed form.
    Malformed,
    /// The signature is another body's, or another secret's.
    Mismatch,
}

impl fmt::Display for Unproven {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySecret => f.write_str("the app secret is empty, so no signature proves the call"),
            Self::Malformed => f.write_str("the call carries no well-formed signature"),
            Self::Mismatch => f.write_str("the signature is not the body's under the app secret"),
        }
    }
}

impl std::error::Error for Unproven {}

/// What a refused send or a failed status means, where Meta's code says it. The numbers are Meta's
/// ([WhatsApp's](https://developers.facebook.com/documentation/business-messaging/whatsapp/support/error-codes),
/// [Messenger's and Instagram's](https://developers.facebook.com/documentation/business-messaging/messenger-platform/error-codes)), and a code
/// Meta has since given another meaning reads as that meaning: Messenger's `1545041` is a person not
/// available, and its closed window is `10` with `2534022` or `2018278`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reason {
    /// The person's window had closed: WhatsApp's `131047`; Messenger's and Instagram's `10` -
    /// `2534022` or `2018278`.
    WindowClosed,
    /// The person cannot be reached: Messenger's `551`, and `1545041` alone or under `200` or `551`.
    Unavailable,
    /// The person stopped the business's marketing messages: WhatsApp's `131050`.
    Stopped,
    /// Any other code.
    Other,
}

/// The [`Reason`] of Meta's `code`, with its `subcode` where it gave one.
#[must_use]
pub const fn reason(code: i64, subcode: Option<i64>) -> Reason {
    match (code, subcode) {
        (131_047, _) | (10, Some(2_534_022 | 2_018_278)) => Reason::WindowClosed,
        (1_545_041, _) | (200 | 551, Some(1_545_041)) | (551, None) => Reason::Unavailable,
        (131_050, _) => Reason::Stopped,
        _ => Reason::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Each code as Meta's pages name it today, and one Meta gave another meaning: Messenger's 1545041 once read
    // "Messaging window closed".
    #[test]
    fn a_code_reads_as_metas_current_pages_name_it() {
        assert_eq!(reason(131_047, None), Reason::WindowClosed);
        assert_eq!(reason(10, Some(2_534_022)), Reason::WindowClosed);
        assert_eq!(reason(10, Some(2_018_278)), Reason::WindowClosed);
        assert_eq!(reason(10, Some(2_018_065)), Reason::Other, "a window news messaging alone opens");
        assert_eq!(reason(10, None), Reason::Other);
        assert_eq!(reason(1_545_041, None), Reason::Unavailable);
        assert_eq!(reason(200, Some(1_545_041)), Reason::Unavailable);
        assert_eq!(reason(551, Some(1_545_041)), Reason::Unavailable);
        assert_eq!(reason(551, None), Reason::Unavailable);
        assert_eq!(reason(200, None), Reason::Other);
        assert_eq!(reason(131_050, None), Reason::Stopped);
        assert_eq!(reason(131_049, None), Reason::Other);
    }
}
