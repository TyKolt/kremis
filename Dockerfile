# ---- Build ----
# Pinned to bookworm so glibc matches the bookworm runtime stage below.
FROM rust:1.89-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/ apps/
RUN cargo build --release -p kremis -p kremis-mcp \
 && strip target/release/kremis target/release/kremis-mcp

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends curl ca-certificates \
 && rm -rf /var/lib/apt/lists/*
RUN groupadd -r kremis && useradd -r -g kremis kremis
COPY --from=build /src/target/release/kremis     /usr/local/bin/kremis
COPY --from=build /src/target/release/kremis-mcp /usr/local/bin/kremis-mcp
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh \
 && mkdir -p /data && chown kremis:kremis /data
USER kremis
WORKDIR /data
EXPOSE 8080
VOLUME ["/data"]
HEALTHCHECK --interval=30s --timeout=3s CMD curl -f http://localhost:8080/health || exit 1
# Default: boot the embedded HTTP server and run the MCP stdio bridge in
# the foreground (suitable for `docker run -i` and MCP registry checks).
# To run only the HTTP API: `docker run --entrypoint kremis ... server -H 0.0.0.0 -D /data/kremis.db`.
ENTRYPOINT ["docker-entrypoint.sh"]
