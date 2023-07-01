FROM rust:1.70 as builder
WORKDIR /usr/src/slack-bot-rust
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
ENV RUST_BACKTRACE=1
WORKDIR /opt/app_name

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    update-ca-certificates && \
    rm -rf /var/lib/apt/lists/*

RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/slack-bot-rust .
CMD ["./slack-bot-rust"]

