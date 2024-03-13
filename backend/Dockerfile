FROM rust:1.76 as builder

USER root

WORKDIR /usr/src/app

COPY . .

RUN apt-get update && \
    apt-get install -y protobuf-compiler

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder /usr/src/app/target/release/messaging /usr/local/bin/messaging

CMD ["grpc"]