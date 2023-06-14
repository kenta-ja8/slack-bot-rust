# Slack Bot Rust

## run
```
cargo run
```

## Docker
```
docker build ./ -t asia-northeast1-docker.pkg.dev/${PROJECT_ID}/${REPOSITORY_NAME}/slack-bot-rust --platform linux/amd64
docker run --rm -it --env-file ./.env slack-bot-rust
docker push asia-northeast1-docker.pkg.dev/${PROJECT_ID}/${REPOSITORY_NAME}/slack-bot-rust
```
