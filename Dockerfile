FROM docker.io/debian:bullseye-slim AS base
WORKDIR /app
EXPOSE 8080

FROM docker.io/rust:1-bullseye AS build
WORKDIR /usr/src/app
RUN USER=root cargo init
COPY ./Cargo.toml .
COPY ./Cargo.lock .
RUN cargo build --release
COPY ./src ./src
RUN touch src/main.rs && cargo build --release

FROM base AS final
WORKDIR /app
COPY --from=build /usr/src/app/target/release/test-api .
ENV RUST_BACKTRACE=full
ENTRYPOINT ["/app/test-api"]
