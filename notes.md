# Webhook log example

▶ POST /

Headers:
  X-Real-Ip: A.B.C.D
  Accept-Encoding: gzip,deflate
  X-Forwarded-Proto: https
  Authorization: Bearer <redacted>
  Content-Type: multipart/form-data; boundary=da8b3d6e-5fb9-461d-9305-fc0fef762441
  X-Forwarded-Host: <redacted>
  X-Forwarded-Server: <redacted>
  Content-Length: 24530
  Accept-Charset: UTF-8
  X-Forwarded-Port: 443
  X-Audio-Size: 24095
  X-Forwarded-For: A.B.C.D
  Accept: application/json
  User-Agent: CoreApp/1.7.0.1-6-gd67ab989

Multipart fields:
  [text] recordedAt = 1786360210629
  [text] client = ring
  [file] audio: content-type=audio/mp4, size=24095 bytes


# Behavior

If "both" send but transcription fails, the Webhook is never called.
If retried and transcription succeeds, it is sent at that moment.

# ffmpeg CLI to map to Rust for conversion to WAV

ffmpeg -i 2026-08-10T134732Z.mp4 -ar 16000 -ac 1 2026-08-10T134732Z.wav
