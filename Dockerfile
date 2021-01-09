FROM rust:latest as builder

WORKDIR /root/music-bio
COPY . .

RUN cargo build --release

FROM alpine:latest
WORKDIR /root/music-bio
COPY --from=builder /root/music-bio/target/release/music-bio ./
COPY --from=builder /root/music-bio/.env ./

CMD ./music-bio
