# Build
FROM --platform=$BUILDPLATFORM rust:1.67.1 as builder
ARG TARGETARCH
COPY platform.sh .
RUN bash platform.sh
# TODO: aarch64 cant find `aarch64-linux-musl-gcc`
RUN apt-get update && apt-get install -y musl-tools
RUN rustup target add $(cat /.platform)

WORKDIR /usr/src/libgen-bot-rs
COPY . .
ENV OPENSSL_STATIC=1
RUN cargo install --target $(cat /.platform) --path .

# Bundle
FROM --platform=$BUILDPLATFORM alpine
ARG TARGETARCH
WORKDIR /app
COPY --from=builder /usr/local/cargo/bin/libgen-bot-rs .
COPY log.yml .
ENTRYPOINT [ "./libgen-bot-rs" ]