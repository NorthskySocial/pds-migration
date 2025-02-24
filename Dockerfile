FROM rust:latest AS builder

# Copy local code to the container image.
WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs

## Install production dependencies and build a release artifact.
RUN cargo build --release

COPY src src

# You have to make the timestamp newer for it to build
RUN touch src/main.rs

RUN cargo build --release

FROM rust:slim

COPY --from=builder /app/target/release/PdsMigration .

ENTRYPOINT ["./PdsMigration"]