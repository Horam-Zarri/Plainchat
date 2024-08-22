ARG PROGRAM_NAME=rechat
FROM rust:1.80.1-alpine3.20

RUN apk add musl-dev
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {println!(\"Rust runs in docker.\");}" > src/main.rs 

RUN cargo build
RUN rm src/*.rs
RUN rm ./target/debug/deps/plainchat*

COPY ./src ./src
COPY ./migrations ./migrations
COPY ./.sqlx ./.sqlx


ENV SQLX_OFFLINE true
RUN cargo build


ENTRYPOINT [ "./target/debug/plainchat-server" ]
