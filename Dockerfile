FROM rust:1.97.1-alpine3.24 AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.24.1
EXPOSE 8080
RUN apk add --no-cache ca-certificates ffmpeg
WORKDIR /app
COPY --from=builder /app/target/release/index-webhook .
CMD ["./index-webhook"]
