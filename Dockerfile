FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/cargo-chec /usr/local/bin/
ENTRYPOINT ["cargo-chec"]