FROM rust:latest as builder

RUN cargo new --bin /opt/app
WORKDIR /opt/app

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

COPY . .
RUN cargo build --release

FROM rust:latest

RUN apt-get update && apt-get install -y \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/app
COPY --from=builder /opt/app/target/release/music-bio ./
COPY --from=builder /opt/app/.env ./

CMD ./music-bio
