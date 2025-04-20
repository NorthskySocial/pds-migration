FROM rust:latest AS builder

# Copy local code to the container image.
WORKDIR /app

COPY Cargo.toml rust-toolchain.toml ./
COPY pdsmigration-common pdsmigration-common
COPY pdsmigration-gui pdsmigration-gui
COPY pdsmigration-web pdsmigration-web

RUN cargo build --release --package pdsmigration-web

FROM rust:slim

COPY --from=builder /app/target/release/pdsmigration-web/ .

ENTRYPOINT ["./pdsmigration-web"]

LABEL org.opencontainers.image.source=https://github.com/NorthskySocial/pds-migration
LABEL org.opencontainers.image.description="PDS migration tool"
LABEL org.opencontainers.image.licenses=MIT