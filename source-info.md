# Webhooks

_From https://help.repebble.com/en/articles/15724406-index-advanced-features-mcp-webhook#h_9be218b88b_

You can set up a webhook which will be sent on each recording, in addition to the agent execution. This means note creation, actions, etc. will still run but you can store your text/audio on your own server or have it sent elsewhere.

You can find webhook settings under the main Index tab settings. Currently, only a single webhook can be configured.

## Settings explained

- Webhook URL: The URL of the webhook itself.
- Headers: Any headers to be sent along with the request, this used to be limited to just a non-standard auth header but now can be anything. Using a standard Authorization header with some form of token is recommended for basic security.
- Send: You can choose to send just the recording, just the transcription, or both.
- Trigger: The button combination which will trigger webhooks (or you can pick all)

## Request format

The webhook request will be a HTTPS POST to your configured URL in multipart/form-data format.

### Multipart fields

Included when

Content type

- `audio`: Included when webhook is set to send audio/both; content type: `audio/mp4`
- `transcription`: Included when transcription is available (succeeded) and webhook is set to send text/both; content type: Text
- `recordedAt`: Always included; content type: Recording timestamp as milliseconds since unix epoch
- `client`: Always included; content type: Text (`ring`)

### Request headers

- `Content-Type`: Always included; `multipart/form-data; boundary=<uuid>`
- `X-Audio-Size`: Included when audio is present; Size of M4A payload in bytes
- User-defined headers: Always included; e.g. Authorization: Bearer 123abc
