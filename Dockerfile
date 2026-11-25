# Rust edition 2024: requires Rust 1.85.0 or newer.
FROM rust:1.100-bookworm

WORKDIR /app

# Copy dependency manifests
COPY Cargo.toml Cargo.lock ./

# Copy application source
COPY src ./src

COPY migrations ./migrations

# Compile the application
RUN cargo build

EXPOSE 3000

CMD ["cargo", "run"]
