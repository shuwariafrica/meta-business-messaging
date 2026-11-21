# meta-business-messaging

Meta's business messaging protocol for the three assets a business answers customers on: a
WhatsApp business number, a Facebook Page and an Instagram professional account.

- `signature` proves a webhook call came from Meta under the app secret.
- `handshake` answers the webhook subscription.
- `whatsapp`, `messenger` and `instagram` read each channel's webhook bodies.
- `send` writes the bodies a send posts.

The crate has no transport, so it runs under any HTTP server or client: it reads the bytes a call
carries and returns values, and a send body is a `Serialize` value the caller writes with
`serde_json`.

It reads only the fields it uses, and a value newer than the crate reads as its set's `Other`, so a
payload Meta extends without a new API version still reads.

The API is documented at <https://docs.rs/meta-business-messaging>.

## Licence

Apache-2.0, in [LICENSE](LICENSE).
