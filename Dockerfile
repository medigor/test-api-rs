FROM docker.io/library/rust:1-bullseye AS build
RUN apt-get -y update && \
    apt-get -y install musl musl-dev musl-tools && \
    rustup target add x86_64-unknown-linux-musl
ENV RUST_BACKTRACE=1 \
    CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

WORKDIR /usr/src/app
RUN USER=root cargo init
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release --target x86_64-unknown-linux-musl
COPY ./src ./src
RUN touch src/main.rs && cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=build /usr/src/app/target/x86_64-unknown-linux-musl/release/test-api .
# COPY --from=build /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ENV RUST_BACKTRACE=1
ENTRYPOINT ["/test-api"]
