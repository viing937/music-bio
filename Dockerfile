FROM rust as planner
WORKDIR /opt/app
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM rust as cacher
WORKDIR /opt/app
RUN cargo install cargo-chef
COPY --from=planner /opt/app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json


FROM rust as builder
WORKDIR /opt/app
COPY . .
COPY --from=cacher /opt/app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release


FROM rust
RUN apt-get update && apt-get install -y \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /opt/app
COPY --from=builder /opt/app/target/release/music-bio ./
COPY --from=builder /opt/app/.env ./
CMD ./music-bio
