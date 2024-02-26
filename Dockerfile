FROM rust:latest

WORKDIR /usr/src/event-listener

COPY . .
RUN cargo build

CMD cargo run