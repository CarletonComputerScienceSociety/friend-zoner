FROM rust:1 as builder

WORKDIR /app

COPY . /app
RUN cargo build --release


FROM debian:buster-slim

COPY --from=builder /app/target/release/friend-zoner /app/friend-zoner

ENTRYPOINT ["/app/friend-zoner"]