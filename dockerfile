FROM rust:latest AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/rustforge .
COPY frontend/ frontend/
COPY themes/ themes/
COPY plugins/ plugins/
COPY migrations/ migrations/
RUN mkdir uploads
EXPOSE 3000
CMD ["./rustforge"]