FROM rust:1.75-slim AS builder

WORKDIR /app

# Copiar archivos de dependencias
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copiar código fuente
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release

# Imagen final
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-cloud-run-test /app/app

EXPOSE 8080

ENV PORT=8080

CMD ["/app/app"]