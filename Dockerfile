FROM rust:1.78 as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN apt update
RUN apt install musl musl-tools -y
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --target=x86_64-unknown-linux-musl --release
RUN mv ./target/x86_64-unknown-linux-musl/release/libgen-bot-rs ./libgen-bot-rs

# Runtime image
FROM debian:bullseye-slim
RUN apt update; apt install -y ca-certificates
WORKDIR /app
# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/libgen-bot-rs /app/libgen-bot-rs
EXPOSE 3000
CMD ["./libgen-bot-rs"]
