# Kremis CLI Reference

Command-line interface for Kremis, the deterministic graph substrate for AI grounding.

> **Source of truth:** `apps/kremis/src/cli/mod.rs`

---

## Global Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--verbose` | `-v` | Enable verbose output | Off |
| `--quiet` | `-q` | Suppress banner output | Off |
| `--database <path>` | `-D` | Path to the graph database | `kremis.db` |
| `--backend <type>` | `-B` | Storage backend: `file` or `redb` | `redb` |
| `--json-mode` | | Output in JSON format | Off |

---

## Commands

### init

Initialize a new empty database.

```bash
kremis init [--force]
```

| Option | Description |
|--------|-------------|
| `--force`, `-f` | Force initialization even if database exists |

### server

Start the HTTP API server.

```bash
kremis server [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--host <host>` | `-H` | Host address to bind to | `127.0.0.1` |
| `--port <port>` | `-p` | Port to listen on | `8080` |

### status

Show graph status (node count, edge count, density).

```bash
kremis status
```

### stage

Show the current developmental stage of the graph.

```bash
kremis stage [--detailed]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--detailed` | `-d` | Show detailed progress information |

### ingest

Ingest signals from a file.

```bash
kremis ingest -f <FILE> [-t <FORMAT>]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--file <path>` | `-f` | Path to the input file | (required) |
| `--format <fmt>` | `-t` | Input format: `json` or `text` | `json` |

**JSON format** — array of signal objects:

```json
[
  {"entity_id": 1, "attribute": "name", "value": "Alice"},
  {"entity_id": 2, "attribute": "name", "value": "Bob"}
]
```

**Text format** — colon-separated `entity_id:attribute:value` per line:

```text
1:name:Alice
2:name:Bob
```

### query

Execute a query on the graph.

```bash
kremis query -t <TYPE> [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--query-type <type>` | `-t` | Query type (see below) | (required) |
| `--start <id>` | `-s` | Start node ID | — |
| `--end <id>` | `-e` | End node ID (for path) | — |
| `--depth <n>` | `-d` | Traversal depth | `3` |
| `--entity <id>` | | Entity ID (for lookup) | — |
| `--nodes <ids>` | | Comma-separated node IDs (for intersect) | — |
| `--min-weight <w>` | | Minimum edge weight filter | — |

**Query types:**

| Type | Required Options | Description |
|------|-----------------|-------------|
| `lookup` | `--entity` | Find node by entity ID |
| `traverse` | `--start`, `--depth` | BFS traversal from node (add `--min-weight` for filtered) |
| `path` | `--start`, `--end` | Find strongest path |
| `intersect` | `--nodes` | Find common connections |

**Examples:**

```bash
kremis query -t lookup --entity 1
kremis query -t traverse -s 0 -d 3
kremis query -t path -s 0 -e 5
kremis query -t intersect --nodes "0,1,2"
kremis query -t traverse -s 0 -d 3 --min-weight 5
```

### export

Export graph in canonical format.

```bash
kremis export -o <FILE> [-t <FORMAT>]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--output <path>` | `-o` | Output file path | (required) |
| `--format <fmt>` | `-t` | Export format: `canonical` or `json` | `canonical` |

### import

Import graph from canonical format.

```bash
kremis import -i <FILE>
```

| Option | Short | Description |
|--------|-------|-------------|
| `--input <path>` | `-i` | Input file path |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `KREMIS_API_KEY` | If set, enables Bearer token authentication | (disabled) |
| `KREMIS_RATE_LIMIT` | Requests per second rate limit | `100` |
| `KREMIS_CORS_ORIGINS` | Comma-separated allowed origins, or `*` | localhost only |

---

## Examples

```bash
# Initialize and start
kremis init
kremis server --port 8080

# Ingest signals
kremis ingest -f data.json -t json

# Query
kremis query -t lookup --entity 1
kremis query -t traverse -s 0 -d 3

# Status
kremis status
kremis stage --detailed

# Export/Import
kremis export -o graph.bin -t canonical
kremis import -i graph.bin

# JSON mode for scripting
kremis --json-mode status
```

---

## See Also

- [API Documentation](./API.md) — HTTP API reference
