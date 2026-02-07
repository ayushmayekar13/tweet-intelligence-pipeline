# Tweet Intelligence Pipeline

A high‑performance Rust backend that ingests tweets in real time, embeds them using OpenAI, stores vectors in Qdrant, and serves similarity search results over WebSockets and HTTP APIs.

This project is designed for:
- Real‑time tweet ingestion (via browser extensions / WebSocket clients)
- Vector similarity search at scale
- Minimal infrastructure (Qdrant as the only database)

## Runtime Resource Consumption

(Running in a docker container)
- Idle Memory usage: <15 MB
- Idle CPU usage: 0.1% of 1CPU(Apple ARM M2 chip)


---

## Architecture Overview

```
Client (Chrome Extension)
        │
        ▼
Actix‑Web Server (Rust)
  ├─ WebSocket ingestion (/ws)
  ├─ JWT verification (Clerk)
  ├─ Batching of requests
  |
  │
  ▼
OpenAI Embeddings API
  │
  ▼
Qdrant Vector Database (Cloud / Self‑hosted)
  |(Cosine Similarity Search)
  |
Ingestion into a Hashmap for real time retrieval
  |
WebSocket
  |
  ▼
Client (Chrome Extension)
```

---

## Tech Stack

| Component | Technology |
|---------|------------|
| Language | Rust (edition 2021) version: 1.82.0 |
| Web Framework | Actix‑web |
| WebSockets | actix‑ws |
| Vector DB | Qdrant |
| Embeddings | OpenAI API |
| Auth | Clerk (JWKS) |
| Runtime | Tokio |
| Containerization | Docker |
| Deployment | AWS EC2 / ECR / Fargate |

---

## Features

- Real‑time tweet ingestion via WebSocket
- Batched vector embedding generation
- Qdrant‑backed similarity search
- Server‑side buffering and async fan‑out
- Zero polling (Notify‑based signaling)
- Dockerized for deployment

---

## Project Structure

```
Backend/
├── src/
│   ├── main.rs            # App entry point
│   ├── routes/            # HTTP & WS handlers
│   ├── models/            # Request / response structs
│   ├── qdrant_functions/  # Qdrant logic
│   ├── auth/              # JWT + JWKS handling
│   └── embedding.rs       # Embed test
│
├── Cargo.toml
├── Dockerfile
├── .gitignore
└── README.md
```

---

## Environment Variables

Create a `.env` file locally:

```env
OPENAI_API_KEY=sk-...
OPENAI_ENDPOINT=https://api.openai.com/v1/embeddings

QDRANT_API_KEY=...
QDRANT_ENDPOINT=https://<cluster>.cloud.qdrant.io

CLERK_JWKS=https://<clerk-domain>/.well-known/jwks.json
```

> ⚠️ `.env` is **not** included in Docker images or Git commits.

---

## Running Locally (Without Docker)

```bash
cargo run
```

Server will start on:
```
http://localhost:8080
```

Health check:
```bash
curl http://localhost:8080/health
```

---

## Running with Docker

### Build image
```bash
docker build -t tweet-backend .
```

### Run container
```bash
docker run -d \
  --name tweet-backend \
  --env-file .env \
  -p 8080:8080 \
  tweet-backend
```

Verify:
```bash
curl http://localhost:8080/health
```

---

## API Overview

### WebSocket: `/ws`

Used for:
- Sending tweets to be embedded
- Receiving similarity results

Payload example:
```json
{
  "tweets": [
    {
      "user_id": "user_123",
      "id": "tweet_456",
      "text": "I love Rust",
      "username": "alice"
    }
  ]
}
```

---

### POST `/search_payload`

Search tweets by user.

Request:
```json
{
  "user_id": "user_123",
  "limit": 5
}
```

Response:
```json
{
  "status": "success",
  "payload": {
    "points": [
      {
        "id": "uuid",
        "payload": {
          "user_id": "user_123",
          "text": "I love Rust"
        }
      }
    ]
  }
}
```

---

## Deployment Notes

- Designed to run on a single EC2 with Docker + Elastic IP
- No external DB required (Qdrant only)
- Stateless server (safe restarts)

---

