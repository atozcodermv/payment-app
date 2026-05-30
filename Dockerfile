FROM public.ecr.aws/docker/library/rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release --locked || cargo build --release

FROM public.ecr.aws/docker/library/debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/invoice-service /usr/local/bin/invoice-service
COPY --from=builder /app/target/release/mock-psp /usr/local/bin/mock-psp
COPY migrations ./migrations
COPY openapi.yaml ./openapi.yaml
EXPOSE 8080 8081
CMD ["invoice-service"]
