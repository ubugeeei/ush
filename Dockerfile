FROM rust:1.88-bookworm AS builder

WORKDIR /app
COPY . .
RUN cargo build --locked --release -p ush -p ush_lsp

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    jq \
    procps \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --create-home --uid 1000 --shell /usr/local/bin/ush ush

COPY --from=builder /app/target/release/ush /usr/local/bin/ush
COPY --from=builder /app/target/release/ush_lsp /usr/local/bin/ush_lsp
COPY examples /usr/local/share/ush/examples

ENV SHELL=/usr/local/bin/ush
WORKDIR /workspace
RUN chown ush:ush /workspace

# Drop privileges. Anyone who needs root inside the container can still
# pass `--user 0` at run time.
USER ush

SHELL ["/usr/local/bin/ush", "-c"]
CMD ["/usr/local/bin/ush"]
