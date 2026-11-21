//! The [subscription handshake](https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/create-webhook-endpoint): Meta calls the callback with
//! `GET ?hub.mode=subscribe&hub.challenge=<challenge>&hub.verify_token=<the caller's>` and takes
//! status 200 with the challenge as the body for a subscription, and anything else as a refusal.
//! Every channel does the same. WhatsApp's page calls the challenge "A random string that Meta will
//! generate"; [Messenger's](https://developers.facebook.com/documentation/business-messaging/messenger-platform/webhooks)
//! and [Instagram's](https://developers.facebook.com/documentation/instagram-platform/webhooks/setup) call it an
//! `int`. [`answer`] returns it as the call carries it.

/// The three query parameters as the call carries them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verification<'a> {
    /// `hub.mode`.
    pub mode: &'a str,
    /// `hub.challenge`.
    pub challenge: &'a str,
    /// `hub.verify_token`.
    pub verify_token: &'a str,
}

/// The three parameters read from a call's query string, each percent-decoded as a form's value is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub mode: String,
    pub challenge: String,
    pub verify_token: String,
}

impl Query {
    /// The parameters of `query` (the text after `?`), the last of a repeated name standing; none
    /// where one of the three is missing or a value does not decode to UTF-8.
    #[must_use]
    pub fn read(query: &str) -> Option<Self> {
        let (mut mode, mut challenge, mut verify_token) = (None, None, None);
        for pair in query.split('&') {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            let slot = match decoded(name)?.as_str() {
                "hub.mode" => &mut mode,
                "hub.challenge" => &mut challenge,
                "hub.verify_token" => &mut verify_token,
                _ => continue,
            };
            *slot = Some(decoded(value)?);
        }
        Some(Self { mode: mode?, challenge: challenge?, verify_token: verify_token? })
    }

    /// The parameters as [`answer`] reads them.
    #[must_use]
    pub fn verification(&self) -> Verification<'_> {
        Verification { mode: &self.mode, challenge: &self.challenge, verify_token: &self.verify_token }
    }
}

/// `text` with `+` read as a space and each `%XX` as its byte; none for a malformed escape or bytes
/// that are not UTF-8.
fn decoded(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text.as_bytes();
    while let Some((&first, tail)) = rest.split_first() {
        rest = tail;
        match first {
            b'+' => bytes.push(b' '),
            b'%' => {
                let (hex, tail) = rest.split_first_chunk::<2>()?;
                if !hex.iter().all(u8::is_ascii_hexdigit) {
                    return None;
                }
                bytes.push(u8::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?);
                rest = tail;
            }
            other => bytes.push(other),
        }
    }
    String::from_utf8(bytes).ok()
}

/// Why the handshake is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// `hub.mode` is not `subscribe`.
    Mode,
    /// `hub.verify_token` is not the caller's.
    Token,
}

/// The body to answer with status 200: the challenge, where the call carries `verify_token`.
///
/// # Errors
/// [`Refused`], the call's first fault in the order of its variants.
pub fn answer<'a>(call: &Verification<'a>, verify_token: &str) -> Result<&'a str, Refused> {
    if call.mode != "subscribe" {
        return Err(Refused::Mode);
    }
    // The token is a secret. A length that differs is public and ends the comparison; tokens of one length are compared
    // whole, every byte read whatever the first difference.
    if call.verify_token.len() != verify_token.len() || call.verify_token.bytes().zip(verify_token.bytes()).fold(0_u8, |acc, (a, b)| acc | (a ^ b)) != 0 {
        return Err(Refused::Token);
    }
    Ok(call.challenge)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::panic)] // A test asserts; a failed expectation is the finding.
mod tests {
    use super::*;

    // Meta's example: https://developers.facebook.com/documentation/business-messaging/whatsapp/webhooks/create-webhook-endpoint
    #[test]
    fn the_challenge_is_answered_under_the_callers_token_and_nothing_else() {
        let call = Verification { mode: "subscribe", challenge: "1158201444", verify_token: "vibecoding" };
        assert_eq!(answer(&call, "vibecoding"), Ok("1158201444"));
        assert_eq!(answer(&call, "vibecodin"), Err(Refused::Token));
        assert_eq!(answer(&call, "vibecoding2"), Err(Refused::Token));
        assert_eq!(answer(&Verification { mode: "unsubscribe", ..call }, "vibecoding"), Err(Refused::Mode));
        // WhatsApp's page calls the challenge "a random string", so it is answered as sent, digits or not.
        assert_eq!(answer(&Verification { challenge: "a1b2-c3", ..call }, "vibecoding"), Ok("a1b2-c3"));
    }

    #[test]
    fn the_query_is_read_as_a_form_encodes_it_and_refused_where_a_parameter_is_missing_or_malformed() {
        let read = Query::read("hub.mode=subscribe&hub.challenge=1158201444&hub.verify_token=vibecoding").unwrap();
        assert_eq!(answer(&read.verification(), "vibecoding"), Ok("1158201444"));
        let escaped = Query::read("hub.verify_token=a%2Bb+c%C3%A4&hub.mode=subscribe&other=x&hub.challenge=7").unwrap();
        assert_eq!(escaped.verify_token, "a+b c\u{e4}", "a token's escapes decoded, `+` a space");
        assert_eq!(Query::read("hub.mode=subscribe&hub.challenge=7"), None, "no token");
        assert_eq!(Query::read("hub.mode=subscribe&hub.challenge=7&hub.verify_token=%G1"), None, "not an escape");
        assert_eq!(Query::read("hub.mode=subscribe&hub.challenge=7&hub.verify_token=%"), None, "a cut escape");
        assert_eq!(Query::read("hub.mode=subscribe&hub.challenge=7&hub.verify_token=%FF"), None, "not UTF-8");
        assert_eq!(Query::read(""), None);
    }
}
