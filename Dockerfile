FROM rust:1.88-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --release -p ush

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    jq \
    procps \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/ush /usr/local/bin/ush
COPY examples /usr/local/share/ush/examples

ENV SHELL=/usr/local/bin/ush
WORKDIR /workspace

SHELL ["/usr/local/bin/ush", "-c"]
CMD ["/usr/local/bin/ush"]
