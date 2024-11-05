FROM lukemathwalker/cargo-chef:latest-rust-1.82.0-slim AS chef
WORKDIR /app
RUN apt update \
    && apt install lld clang -y \
    && apt install pkg-config -y \
    && apt install libssl-dev -y

FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/blog-app blog-app
COPY configuration configuration
CMD ["./blog-app"]