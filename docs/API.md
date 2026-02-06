# Kremis HTTP API

REST API for Kremis - a minimal, deterministic, graph-based cognitive substrate.

**Version:** 0.2.0
**License:** Apache 2.0
**Base URL:** `http://localhost:8080`

---

## Table of Contents

1. [Introduction](#introduction)
2. [Authentication](#authentication)
3. [Rate Limiting](#rate-limiting)
4. [Input Validation](#input-validation)
5. [Endpoint Reference](#endpoint-reference)
   - [GET /health](#get-health)
   - [GET /status](#get-status)
   - [GET /stage](#get-stage)
   - [POST /signal](#post-signal)
   - [POST /query](#post-query)
   - [POST /export](#post-export)
6. [Error Codes](#error-codes)
7. [Examples](#examples)

---

## Introduction

The Kremis HTTP API provides programmatic access to a graph-based cognitive substrate. It allows you to:

- **Ingest signals** - Store grounded observations (Entity | Attribute | Value)
- **Query the graph** - Lookup entities, traverse connections, find paths
- **Monitor status** - Check health, graph statistics, and developmental stage
- **Export data** - Export the entire graph in binary format

All endpoints return JSON responses (except error responses which may return plain text).

---

## Authentication

Authentication is **optional** and controlled by the `KREMIS_API_KEY` environment variable.

### Enabling Authentication

Set the environment variable on the server:

```bash
export KREMIS_API_KEY="your-secret-api-key"
```

### Making Authenticated Requests

When authentication is enabled, include the API key in the `Authorization` header:

```
Authorization: Bearer <your-api-key>
```

### Unauthenticated Endpoints

The `/health` endpoint is always accessible without authentication, regardless of the `KREMIS_API_KEY` setting.

### Authentication Errors

If authentication is required but missing or invalid, the server returns:

- **Status:** `401 Unauthorized`
- **Body:** `"Unauthorized"` (plain text)

---

## Rate Limiting

The server implements rate limiting to prevent abuse.

### Default Limit

- **100 requests per second** per client

### Configuration

Configure the rate limit via environment variable:

```bash
export KREMIS_RATE_LIMIT=200  # requests per second
```

### Rate Limit Exceeded

When the rate limit is exceeded, the server returns:

- **Status:** `429 Too Many Requests`
- **Body:** `"Too Many Requests"` (plain text)

---

## Input Validation

The API enforces the following limits:

| Field | Limit |
|-------|-------|
| `attribute` | Max 256 bytes |
| `value` | Max 64 KB (65,536 bytes) |
| `depth` | Max 100 |
| `nodes` (intersect query) | Max 100 items |

Requests exceeding these limits return `400 Bad Request` with a descriptive error message.

---

## Endpoint Reference

### GET /health

Health check endpoint. Always accessible without authentication.

**Tags:** Health

#### Response

**Status:** `200 OK`

```json
{
  "status": "ok",
  "version": "0.2.0"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Health status (always `"ok"` if server is running) |
| `version` | string | Server version |

---

### GET /status

Returns current graph statistics.

**Tags:** Health
**Authentication:** Required (if enabled)

#### Response

**Status:** `200 OK`

```json
{
  "node_count": 42,
  "edge_count": 35,
  "stable_edges": 10,
  "density_millionths": 1234
}
```

| Field | Type | Description |
|-------|------|-------------|
| `node_count` | integer | Total number of nodes in the graph |
| `edge_count` | integer | Total number of edges |
| `stable_edges` | integer | Edges above the stability threshold |
| `density_millionths` | integer | Graph density expressed in millionths |

---

### GET /stage

Returns the current developmental stage of the graph.

**Tags:** Health
**Authentication:** Required (if enabled)

#### Developmental Stages

| Stage | Name | Description |
|-------|------|-------------|
| S0 | Signal Segmentation | Initial stage, building basic structure |
| S1 | Pattern Crystallization | Patterns emerging from repeated signals |
| S2 | Causal Chaining | Strong interconnections forming |
| S3 | Recursive Optimization | Stable, well-connected graph |

#### Response

**Status:** `200 OK`

```json
{
  "stage": "S1",
  "name": "Pattern Crystallization",
  "progress_percent": 45,
  "stable_edges_needed": 100,
  "stable_edges_current": 45
}
```

| Field | Type | Description |
|-------|------|-------------|
| `stage` | string | Current stage (`S0`, `S1`, `S2`, `S3`) |
| `name` | string | Human-readable stage name |
| `progress_percent` | integer | Progress towards next stage (0-100) |
| `stable_edges_needed` | integer | Stable edges required for next stage |
| `stable_edges_current` | integer | Current count of stable edges |

---

### POST /signal

Ingest a new signal into the graph. A signal represents a grounded observation in the form: **Entity | Attribute | Value**.

**Tags:** Signals
**Authentication:** Required (if enabled)

#### Request Body

```json
{
  "entity_id": 1,
  "attribute": "name",
  "value": "Alice"
}
```

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `entity_id` | integer (u64) | Yes | - | Entity identifier |
| `attribute` | string | Yes | Max 256 bytes, non-empty | Attribute name |
| `value` | string | Yes | Max 64 KB, non-empty | Attribute value |

#### Response

**Status:** `200 OK` (success)

```json
{
  "success": true,
  "node_id": 0,
  "error": null
}
```

**Status:** `400 Bad Request` (validation error)

```json
{
  "success": false,
  "node_id": null,
  "error": "Attribute length 300 exceeds maximum 256 bytes"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether ingestion succeeded |
| `node_id` | integer or null | Created/existing node ID (if successful) |
| `error` | string or null | Error message (if failed) |

---

### POST /query

Execute a query against the graph. Supports multiple query types.

**Tags:** Queries
**Authentication:** Required (if enabled)

#### Query Types

##### 1. Lookup

Find a node by entity ID.

```json
{
  "type": "lookup",
  "entity_id": 1
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Must be `"lookup"` |
| `entity_id` | integer (u64) | Yes | Entity to look up |

##### 2. Traverse

Traverse from a node up to a specified depth.

```json
{
  "type": "traverse",
  "node_id": 0,
  "depth": 3
}
```

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `type` | string | Yes | Must be `"traverse"` | - |
| `node_id` | integer (u64) | Yes | - | Starting node |
| `depth` | integer | Yes | 0-100 | Maximum traversal depth |

##### 3. Traverse Filtered

Traverse with a minimum edge weight filter.

```json
{
  "type": "traverse_filtered",
  "node_id": 0,
  "depth": 3,
  "min_weight": 5
}
```

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `type` | string | Yes | Must be `"traverse_filtered"` | - |
| `node_id` | integer (u64) | Yes | - | Starting node |
| `depth` | integer | Yes | 0-100 | Maximum traversal depth |
| `min_weight` | integer (i64) | Yes | - | Minimum edge weight to include |

##### 4. Strongest Path

Find the path with the highest total weight between two nodes.

```json
{
  "type": "strongest_path",
  "start": 0,
  "end": 5
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | string | Yes | Must be `"strongest_path"` |
| `start` | integer (u64) | Yes | Starting node ID |
| `end` | integer (u64) | Yes | Target node ID |

##### 5. Intersect

Find nodes connected to all input nodes.

```json
{
  "type": "intersect",
  "nodes": [0, 1, 2]
}
```

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `type` | string | Yes | Must be `"intersect"` | - |
| `nodes` | array of u64 | Yes | Max 100 items | Node IDs to intersect |

##### 6. Related

Get the related subgraph from a starting node.

```json
{
  "type": "related",
  "node_id": 0,
  "depth": 2
}
```

| Field | Type | Required | Constraints | Description |
|-------|------|----------|-------------|-------------|
| `type` | string | Yes | Must be `"related"` | - |
| `node_id` | integer (u64) | Yes | - | Starting node |
| `depth` | integer | Yes | 0-100 | Maximum depth |

#### Response

**Status:** `200 OK`

Found:
```json
{
  "success": true,
  "found": true,
  "path": [0],
  "edges": [],
  "error": null
}
```

Not found:
```json
{
  "success": true,
  "found": false,
  "path": [],
  "edges": [],
  "error": null
}
```

Validation error:
```json
{
  "success": false,
  "found": false,
  "path": [],
  "edges": [],
  "error": "Depth 150 exceeds maximum 100"
}
```

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether query executed successfully |
| `found` | boolean | Whether results were found |
| `path` | array of u64 | Node path (for path queries) |
| `edges` | array of Edge | Edges in result (for traversal queries) |
| `error` | string or null | Error message (if failed) |

**Edge Object:**

```json
{
  "from": 0,
  "to": 1,
  "weight": 10
}
```

| Field | Type | Description |
|-------|------|-------------|
| `from` | integer (u64) | Source node ID |
| `to` | integer (u64) | Target node ID |
| `weight` | integer (i64) | Edge weight |

---

### POST /export

Export the graph in canonical binary format.

**Tags:** Export
**Authentication:** Required (if enabled)

> **Note:** Export is supported for both in-memory and persistent (redb) backends.
> For persistent backends, a graph snapshot is built by iterating all nodes and edges.

#### Response

**Status:** `200 OK`

```json
{
  "success": true,
  "data": "S1JFWA...",
  "checksum": 12345678,
  "error": null
}
```

| Field | Type | Description |
|-------|------|-------------|
| `success` | boolean | Whether export succeeded |
| `data` | string or null | Base64-encoded graph data |
| `checksum` | integer (u64) or null | Data checksum for verification |
| `error` | string or null | Error message (if failed) |

**Status:** `500 Internal Server Error` (snapshot failure)

```json
{
  "success": false,
  "data": null,
  "checksum": null,
  "error": "Failed to build graph snapshot: ..."
}
```

---

## Error Codes

| Status Code | Description | Response Type |
|-------------|-------------|---------------|
| `200` | Success | JSON |
| `400` | Bad Request - Invalid input | JSON with `error` field |
| `401` | Unauthorized - Missing or invalid API key | Plain text: `"Unauthorized"` |
| `429` | Too Many Requests - Rate limit exceeded | Plain text: `"Too Many Requests"` |
| `500` | Internal Server Error | JSON with `error` field |

---

## Examples

### cURL Examples

#### Health Check

```bash
curl http://localhost:8080/health
```

#### Get Status (with authentication)

```bash
curl -H "Authorization: Bearer your-api-key" \
     http://localhost:8080/status
```

#### Get Developmental Stage

```bash
curl -H "Authorization: Bearer your-api-key" \
     http://localhost:8080/stage
```

#### Ingest a Signal

```bash
curl -X POST http://localhost:8080/signal \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"entity_id": 1, "attribute": "name", "value": "Alice"}'
```

#### Lookup Query

```bash
curl -X POST http://localhost:8080/query \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"type": "lookup", "entity_id": 1}'
```

#### Traverse Query

```bash
curl -X POST http://localhost:8080/query \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"type": "traverse", "node_id": 0, "depth": 3}'
```

#### Traverse with Weight Filter

```bash
curl -X POST http://localhost:8080/query \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"type": "traverse_filtered", "node_id": 0, "depth": 3, "min_weight": 5}'
```

#### Find Strongest Path

```bash
curl -X POST http://localhost:8080/query \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"type": "strongest_path", "start": 0, "end": 5}'
```

#### Intersect Nodes

```bash
curl -X POST http://localhost:8080/query \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"type": "intersect", "nodes": [0, 1, 2]}'
```

#### Get Related Subgraph

```bash
curl -X POST http://localhost:8080/query \
     -H "Authorization: Bearer your-api-key" \
     -H "Content-Type: application/json" \
     -d '{"type": "related", "node_id": 0, "depth": 2}'
```

#### Export Graph

```bash
curl -X POST http://localhost:8080/export \
     -H "Authorization: Bearer your-api-key"
```

### Complete Workflow Example

```bash
# 1. Check server health
curl http://localhost:8080/health
# {"status":"ok","version":"0.2.0"}

# 2. Ingest some signals
curl -X POST http://localhost:8080/signal \
     -H "Content-Type: application/json" \
     -d '{"entity_id": 1, "attribute": "name", "value": "Alice"}'
# {"success":true,"node_id":0,"error":null}

curl -X POST http://localhost:8080/signal \
     -H "Content-Type: application/json" \
     -d '{"entity_id": 2, "attribute": "name", "value": "Bob"}'
# {"success":true,"node_id":1,"error":null}

curl -X POST http://localhost:8080/signal \
     -H "Content-Type: application/json" \
     -d '{"entity_id": 1, "attribute": "knows", "value": "Bob"}'
# {"success":true,"node_id":0,"error":null}

# 3. Check graph status
curl http://localhost:8080/status
# {"node_count":2,"edge_count":0,"stable_edges":0,"density_millionths":0}

# 4. Query the graph
curl -X POST http://localhost:8080/query \
     -H "Content-Type: application/json" \
     -d '{"type": "lookup", "entity_id": 1}'
# {"success":true,"found":true,"path":[0],"edges":[],"error":null}

# 5. Check developmental stage
curl http://localhost:8080/stage
# {"stage":"S0","name":"Signal Segmentation","progress_percent":0,"stable_edges_needed":100,"stable_edges_current":0}
```

---

## See Also

- [GitHub Repository](https://github.com/M2Dr3g0n/kremis) - Source code and issues
