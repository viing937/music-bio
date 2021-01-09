FROM rust:latest as builder

WORKDIR /opt/app
COPY . .

RUN cargo build --release

FROM rust:slim
WORKDIR /opt/app
COPY --from=builder /opt/app/target/release/music-bio ./
COPY --from=builder /opt/app/.env ./

CMD ./music-bio
