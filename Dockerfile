FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:trixie-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/pinbreak .
COPY ./wiki ./wiki
ENV PORT=3000
ENV HOST=0.0.0.0
ENV PB_LOG=DEBUG
EXPOSE 3000
ENTRYPOINT ["./pinbreak", "serve"]
