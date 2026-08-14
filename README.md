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
{
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

## Dependencies

### `ffmpeg` (temporary)

Local re-transcription via audio.cpp currently requires `ffmpeg` to be installed and available on `$PATH`. It is used to convert the MP4 audio received from the Pebble app into a 16 kHz mono WAV before sending it to audio.cpp.

This is a temporary dependency. The plan is to replace it with a pure-Rust implementation (likely Symphonia + hound ?) so that no native tooling needs to be pre-installed.

## How to run Recueil yourself

### Docker (recommended)

`ffmpeg` is bundled in the image, so no extra setup is needed.

Minimal — stores nothing to disk, useful to inspect incoming webhooks:

```sh
docker run -d \
  --name recueil \
  -p 8080:8080 \
  -e AUTH_TOKEN=your-secret-token \
  ghcr.io/horgix/recueil:latest
```

With persistent storage:

```sh
docker run -d \
  --name recueil \
  -p 8080:8080 \
  -e AUTH_TOKEN=your-secret-token \
  -e LOCAL_FILES_STORAGE_DIRECTORY=/recueil-data \
  -v /path/to/your/data:/recueil-data \
  ghcr.io/horgix/recueil:latest
```

With persistent storage and local re-transcription via audio.cpp:

```sh
docker run -d \
  --name recueil \
  -p 8080:8080 \
  -e AUTH_TOKEN=your-secret-token \
  -e LOCAL_FILES_STORAGE_DIRECTORY=/recueil-data \
  -e AUDIOCPP_HOST=audiocpp-host:8080 \
  -e AUDIOCPP_MODEL_ID=your-model-id \
  -e AUDIOCPP_LANGUAGE=en \
  -v /path/to/your/data:/recueil-data \
  ghcr.io/horgix/recueil:latest
```

Recueil listens on `POST /webhook` on port `8080` (or whatever `PORT` is set to).

### Docker compose

Example with persistent storage, local re-transcription via audio.cpp, and Traefik for exposition:

```yaml
services:
  webhook-server:
    container_name: index-webhook-server
    image: ghcr.io/horgix/recueil:latest # Pin to a specific version in production
    # Optional
    # user: <redacted UID>:<redacted GID> # Dedicated user/group for the container to avoid running as root
    restart: unless-stopped
    networks:
      - traefik_proxy
      - recueil
    labels:
      traefik.enable: true
      traefik.http.routers.recueil.rule: Host(`<redacted host>`)
      traefik.http.services.recueil.loadbalancer.server.port: 8080
    environment:
      AUTH_TOKEN: <redacted>
      LOCAL_FILES_STORAGE_DIRECTORY: /recueil-data
      AUDIOCPP_HOST: audiocpp:8080
      AUDIOCPP_MODEL_ID: parakeet
      AUDIOCPP_LANGUAGE: en  # optional
    volumes:
      - recueil-data:/recueil-data
  audiocpp:
    container_name: index-audiocpp
    image: ghcr.io/0xshug0/audio.cpp:full-cpu
    # Optional
    # user: <redacted UID>:<redacted GID> # Dedicated user/group for the container to avoid running as root
    restart: unless-stopped
    networks:
      - recueil
    volumes:
      - /path/to/audio.cpp/models:/models:ro
      - /path/to/audio.cpp/config.json:/server.json:ro
    command: server --config /server.json

networks:
  traefik_proxy:
    external: true
  recueil:
```

With the following Audio.cpp requirements:

First, getting the model:

```bash
cd /path/to/
git clone https://github.com/0xShug0/audio.cpp.git
cd audio.cpp
python tools/model_manager_v2.py install parakeet_tdt_q8_0
```

Second, writing the configuration file for Audio.cpp in `/path/to/audio.cpp/config.json`:

```json
{
  "host": "0.0.0.0",
  "port": 8080,
  "backend": "cpu",
  "models": [
    {
      "id": "parakeet",
      "family": "parakeet_tdt",
      "path": "/models/Parakeet-TDT-0.6B-v3-GGUF/parakeet-tdt-0.6b-v3-q8_0.gguf",
      "task": "asr",
      "mode": "offline"
    }
  ]
}
```

### Binary

Download the latest binary from the [GitHub releases](https://github.com/Horgix/recueil/releases) page, then:

```sh
AUTH_TOKEN=your-secret-token \
./index-webhook
```

See the [Dependencies](#dependencies) section above if you intend to use local re-transcription via audio.cpp.

## Name

**recueil** is a French word for a gathered collection; an anthology of things picked up over time. It comes from *recueillir*: to collect, but also to shelter and take in. That felt right for a service that quietly receives your spoken thoughts from a Pebble Index ring, transcribes them, and keeps them safe; building, over time, a personal _recueil_ of your voice notes.
