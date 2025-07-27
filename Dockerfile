FROM rust:1.88

WORKDIR /app
COPY . .

RUN cargo build --release

EXPOSE 3000

ENTRYPOINT ["./target/release/protocol-rs"]