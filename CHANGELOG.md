# Changelog

<!-- One section per release, newest first, headed by the version exactly as its tag names it
after the `v`: the tag v0.1.0 is "## 0.1.0". The release takes the section's text, to the next
"## " heading, as its notes; a tag with no section of its own is released with notes that say
so. -->

## 0.1.0-alpha.1

The first pre-release: Meta's business messaging protocol for a WhatsApp business number, a
Facebook Page and an Instagram professional account. The crate has no transport: it reads the
bytes a call carries and returns values, and a send body is a `Serialize` value the caller writes
with `serde_json`.

This pre-release's interface will change before 0.1.0 in these places: how reading a body is tied
to its proof, so that a body cannot be read unproven; how a batch reads when one of its items
cannot be read; the public envelope types; the form of a send body; which form Instagram signs;
and which sets of words and fields may grow without breaking a caller.

- `signature::verify` proves a webhook call's `X-Hub-Signature-256` under the app secret, the HMAC
  compared in constant time, over the form the channel signs: `Form::Raw` for WhatsApp,
  `Form::EscapedUnicode` for Messenger. It refuses an empty app secret before comparing anything.
- `handshake::Query::read` reads the subscription call's query, and `handshake::answer` returns the
  challenge where the mode is `subscribe` and the verify token is the caller's. It refuses an empty
  verify token of the caller's before comparing anything.
- `whatsapp::changes` and `whatsapp::values` read a WhatsApp body: inbound messages and their
  contacts, the statuses of sent messages with their pricing and errors, system changes and
  marketing preferences. A media message's file is checked against its stated digest.
- `messenger` reads a Page's messages, echoes, deliveries, reads, edits and ad referrals;
  `instagram` reads an account's messages, echoes, deletions, edits, reads and referrals, under either
  login.
- `send` writes WhatsApp text, media and template requests and a Page's responses and attachments,
  and reads Meta's acceptances and refusals.
- `reason` says what a refused send's or a failed status's code means where Meta's code says it, and
  `ANSWER_WITHIN` is the five seconds Messenger allows a call's answer.
- A word outside a closed set reads as that set's `Other`, so a payload Meta extends without a new
  API version still reads.
