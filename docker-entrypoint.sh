#!/bin/sh
set -e

DB="${KREMIS_DATABASE:-/data/kremis.db}"
HOST="${KREMIS_HOST:-127.0.0.1}"
PORT="${KREMIS_PORT:-8080}"

# Initialize database on first run. Logs go to stderr; stdout is reserved
# for the MCP stdio transport.
if [ ! -f "$DB" ]; then
    kremis init --database "$DB" >&2
fi

# Start the Kremis HTTP server in background, then wait until it answers
# /health before launching the MCP bridge.
kremis server --host "$HOST" --port "$PORT" --database "$DB" >&2 &
SERVER_PID=$!

i=0
while [ $i -lt 50 ]; do
    if curl -sf "http://${HOST}:${PORT}/health" >/dev/null 2>&1; then
        break
    fi
    i=$((i + 1))
    sleep 0.2
done

if ! curl -sf "http://${HOST}:${PORT}/health" >/dev/null 2>&1; then
    echo "kremis server failed to become ready within 10s" >&2
    kill -TERM "$SERVER_PID" 2>/dev/null || true
    exit 1
fi

# Forward termination signals to the background server.
trap 'kill -TERM "$SERVER_PID" 2>/dev/null; exit 0' INT TERM

# Replace the shell with the MCP bridge so it receives signals directly
# and owns stdio for JSON-RPC.
exec env KREMIS_URL="http://${HOST}:${PORT}" kremis-mcp
