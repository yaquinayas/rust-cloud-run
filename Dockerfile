# Stage 1: Build
FROM rust:1.86.0-slim as builder

WORKDIR /app
COPY . .

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
 && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

# Stage 2: Runtime (¡USANDO BOOKWORM-SLIM!)
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-cloud-run /app

ENV PORT=8080
EXPOSE 8080

CMD ["/app"]