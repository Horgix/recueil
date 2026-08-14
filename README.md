# Recueil — A Pebble Index 01 ring webhook receiver

> _Recueil_ is a self-hosted webhook receiver that collects, transcribes, and stores voice recordings from your [Pebble Index 01](https://repebble.com/index) ring.

## What it does

When you record a voice note on your [Pebble Index 01](https://repebble.com) ring, the Pebble app can forward it to a webhook of your choice. **Recueil** is that webhook; a self-hosted receiver that:

1. **Authenticates** incoming requests via a Bearer token
2. **Stores** the raw audio (M4A) and/or the transcription produced by the Pebble app to local disk
3. **Re-transcribes** the audio locally using [audio.cpp](https://github.com/nicoboss/audio.cpp) (optional), if you want transcription to happen on your own hardware instead of, or in addition to, the app's built-in one
4. **Appends** a structured JSON record of every recording to a `recordings.ndjson` log file

```
Pebble Index 01 ring → Pebble app → webhook → recueil
```

Example of resulting `recordings.ndjson` entry (pretty formatted for clarity):

```json
{"
  "recording_source": "ring",
  "recorded_at": "2025-12-31T23:59:59Z",
  "audio": {
    "size_bytes": 106574,
    "path": "/tmp/pebble-index-data/audio/2025-12-31T23-59-59Z.mp4"
  },
  "transcriptions": [
    {
      "done_by": "PebbleApp",
      "content": "¨This is test."
    },
    {
      "done_by": "AudioCpp",
      "content": "This is a test."
    }
  ]
}
```

## Storage layout

When `LOCAL_FILES_STORAGE_DIRECTORY` is set, recueil writes the following structure:

```
<storage_dir>/
  audio/               # raw MP4 files as sent by the pebble app, one per recording
    2026-12-31T23-59-59Z.mp4
  transcriptions/      # plain text transcriptions as sent by the Pebble app
    2026-12-31T23-59-59Z.txt
  recordings.ndjson    # append-only newline-delimited JSON log of all recordings
```

Files are named after the timestamp at which the recording was made on the ring.

## Configuration

All configuration is via environment variables.

| Variable | Required | Default | Description |
|---|---|---|---|
| `AUTH_TOKEN` | yes | — | Bearer token expected in the `Authorization` header |
| `PORT` | no | `8080` | Port to listen on |
| `LOCAL_FILES_STORAGE_DIRECTORY` | no | — | Directory to store audio and transcription files. If unset, nothing is persisted to disk and Recueil can just be used to log your webhook for debugging purpose |
| `AUDIOCPP_HOST` | no | — | Host of a running audio.cpp instance (e.g. `localhost:8080`). Enables local re-transcription |
| `AUDIOCPP_MODEL_ID` | if `AUDIOCPP_HOST` is set | — | Model ID to pass to audio.cpp |
| `AUDIOCPP_LANGUAGE` | no | — | Language hint passed to audio.cpp (e.g. `en`, `fr`) |

> `AUDIOCPP_HOST` requires `LOCAL_FILES_STORAGE_DIRECTORY` to be set, as the audio file must be saved to disk before it can be re-transcribed.

## Name

**recueil** is a French word for a gathered collection; an anthology of things picked up over time. It comes from *recueillir*: to collect, but also to shelter and take in. That felt right for a service that quietly receives your spoken thoughts from a Pebble Index ring, transcribes them, and keeps them safe; building, over time, a personal _recueil_ of your voice notes.
