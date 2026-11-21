//! The proof on every webhook call: `X-Hub-Signature-256: sha256=<hex>`, an HMAC-SHA-256 of the
//! payload under the app secret. A body is read only after [`verify`] has returned `Ok` for it: the
//! readers take any bytes, and nothing in their types records that the proof was made.
//!
//! A call names the [`Form`] its channel signs. WhatsApp
//! [signs the body as sent](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/create-webhook-endpoint),
//! [`Form::Raw`]; Messenger [signs its escaped form](https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks),
//! [`Form::EscapedUnicode`]. Which form Instagram signs is unverified: its
//! [page](https://developers.facebook.com/documentation/instagram-platform/webhooks/setup) states no
//! escaped form, and this crate has proven no signature Meta made for it.

use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

use crate::Unproven;

/// The header a call's proof rides, as HTTP compares header names: in lowercase.
pub const HEADER: &str = "x-hub-signature-256";

/// The form of the body the platform signed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    /// The body as sent.
    Raw,
    /// The body through [`escape_unicode`].
    EscapedUnicode,
}

/// Whether `header` proves `body` under `app_secret`, the HMAC compared in constant time.
///
/// # Errors
/// [`Unproven::EmptySecret`] where `app_secret` is empty, before anything is compared: HMAC admits an
/// empty key, and anyone can sign under it. [`Unproven::Malformed`] for a header not of the form
/// `sha256=<64 hex digits>`, or a body that is not UTF-8 where [`Form::EscapedUnicode`] is signed;
/// [`Unproven::Mismatch`] where the HMAC differs.
pub fn verify(app_secret: &[u8], header: &str, body: &[u8], form: Form) -> Result<(), Unproven> {
    if app_secret.is_empty() {
        return Err(Unproven::EmptySecret);
    }
    let hex = header.strip_prefix("sha256=").ok_or(Unproven::Malformed)?;
    let expected = decode_hex(hex).ok_or(Unproven::Malformed)?;
    let signed: Vec<u8> = match form {
        Form::Raw => body.to_vec(),
        Form::EscapedUnicode => escape_unicode(std::str::from_utf8(body).map_err(|_| Unproven::Malformed)?).into_bytes(),
    };
    let mut mac = Hmac::<Sha256>::new_from_slice(app_secret).map_err(|_| Unproven::Malformed)?;
    mac.update(&signed);
    mac.verify_slice(&expected).map_err(|_| Unproven::Mismatch)
}

/// `text` with each character beyond ASCII written as `\uXXXX` over its UTF-16 units in lowercase
/// hex, so U+00E4 is `\u00e4` and U+1F600 is `\ud83d\ude00`.
#[must_use]
pub fn escape_unicode(text: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        if ch.is_ascii() {
            out.push(ch);
        } else {
            let mut units = [0_u16; 2];
            for unit in ch.encode_utf16(&mut units) {
                // Writing into a String cannot fail.
                let _ = write!(out, "\\u{unit:04x}");
            }
        }
    }
    out
}

fn decode_hex(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0_u8; 32];
    for (i, pair) in hex.as_bytes().chunks(2).enumerate() {
        // `from_str_radix` alone reads a leading `+` as a sign.
        if !pair.iter().all(u8::is_ascii_hexdigit) {
            return None;
        }
        let s = std::str::from_utf8(pair).ok()?;
        *out.get_mut(i)? = u8::from_str_radix(s, 16).ok()?;
    }
    Some(out)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // A test asserts; a failed expectation is the finding.
mod tests {
    use super::*;

    /// `sha256=<hex>` of `signed` under `secret`, as Meta writes the header.
    fn header(secret: &[u8], signed: &[u8]) -> String {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
        mac.update(signed);
        mac.finalize().into_bytes().iter().fold(String::from("sha256="), |mut acc, b| {
            use std::fmt::Write as _;
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }

    /// The published HMAC-SHA256 vector: key `key`, message `The quick brown fox jumps over the lazy dog`.
    const VECTOR: &str = "sha256=f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8";

    #[test]
    fn a_raw_body_is_proven_under_the_secret_and_refused_otherwise() {
        let body = b"The quick brown fox jumps over the lazy dog";
        assert_eq!(verify(b"key", VECTOR, body, Form::Raw), Ok(()));
        assert_eq!(verify(b"other", VECTOR, body, Form::Raw), Err(Unproven::Mismatch));
        assert_eq!(verify(b"key", VECTOR, b"The quick brown fox", Form::Raw), Err(Unproven::Mismatch));
        assert_eq!(verify(b"key", "sha1=abc", body, Form::Raw), Err(Unproven::Malformed));
        assert_eq!(verify(b"key", "sha256=zz", body, Form::Raw), Err(Unproven::Malformed));
        let signed_pair = VECTOR.replacen("f7bc", "f7+c", 1);
        assert_eq!(verify(b"key", &signed_pair, body, Form::Raw), Err(Unproven::Malformed), "a digit pair carries no sign");
        assert_eq!(verify(b"key", &VECTOR.to_uppercase().replace("SHA256", "sha256"), body, Form::Raw), Ok(()), "hex case is not part of the proof");
    }

    #[test]
    fn an_empty_app_secret_proves_nothing_though_the_body_is_signed_under_it() {
        let body = b"{\"object\":\"whatsapp_business_account\",\"entry\":[]}";
        assert_eq!(verify(b"", &header(b"", body), body, Form::Raw), Err(Unproven::EmptySecret));
        assert_eq!(verify(b"", &header(b"", body), body, Form::EscapedUnicode), Err(Unproven::EmptySecret));
        assert_eq!(verify(b"", "sha1=abc", body, Form::Raw), Err(Unproven::EmptySecret), "refused before the header is read");
    }

    #[test]
    fn messenger_signs_the_escaped_unicode_form_as_metas_example_states_it() {
        assert_eq!(escape_unicode("\u{e4}\u{f6}\u{e5}"), "\\u00e4\\u00f6\\u00e5");
        assert_eq!(escape_unicode("plain ascii {\"a\":1}"), "plain ascii {\"a\":1}");
        assert_eq!(escape_unicode("\u{1F600}"), "\\ud83d\\ude00", "a character beyond the BMP is its two UTF-16 units");
        // The escaped form is what is signed: the raw form's HMAC does not prove it, and vice versa.
        let body = "{\"text\":\"\u{e4}\u{f6}\u{e5}\"}";
        let header = header(b"secret", escape_unicode(body).as_bytes());
        assert_eq!(verify(b"secret", &header, body.as_bytes(), Form::EscapedUnicode), Ok(()));
        assert_eq!(verify(b"secret", &header, body.as_bytes(), Form::Raw), Err(Unproven::Mismatch));
        assert_eq!(verify(b"secret", &header, &[0xff, 0xfe], Form::EscapedUnicode), Err(Unproven::Malformed), "not UTF-8");
    }
}
