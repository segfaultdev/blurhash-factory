FROM rust:1-alpine3.12

WORKDIR /opt/blurhash
COPY . .

ENV ALLOWED_HOSTS static.juabali.com
ENV MAX_IMAGE_SIZE 1048576

RUN apk add --no-cache --update \
    pkgconfig \
    build-base \
    libressl-dev \
    protobuf

RUN rustup component add rustfmt
RUN cargo build --release

CMD ["./target/release/blurhash-factory"]