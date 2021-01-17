FROM rust:latest as builder

WORKDIR /opt/app
COPY . .

RUN cargo build --release

FROM rust:slim

RUN apt-get update && apt-get install -y \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/app
COPY --from=builder /opt/app/target/release/music-bio ./
COPY --from=builder /opt/app/.env ./

CMD ./music-bio
