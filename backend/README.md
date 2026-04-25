# 🎮 GameServerManager – Backend (Rust)

High-performance **Rust-based orchestration backend** for GameServerManager, responsible for managing the full lifecycle of dedicated game servers running in Docker containers.

This backend acts as the **central control plane** between the frontend UI, runtime daemon, and containerized game servers.

---

## ⚙️ What this backend does (real implementation scope)

The Rust backend is responsible for:

* 🐳 Creating and managing Docker-based game server containers
* 🎮 Installing and updating game servers via SteamCMD
* 📡 Providing REST API + WebSocket real-time communication
* 🔐 Authentication + role-based access control
* 📊 Tracking server state, logs, and lifecycle events
* ⚡ Streaming live console output (server PTY/logs)
* 🔄 Coordinating execution through a daemon layer

---

## 🧱 System Architecture

```text
Frontend (Vue)
      ↓ REST / WebSocket
Rust Backend (this service)
      ↓
Core Orchestration Layer
      ↓
Docker Engine API
      ↓
Game Server Containers (SteamCMD-based)
      ↓
Daemon (runtime execution agent)
```

### Key idea:

The backend does **NOT run game servers directly**
It orchestrates them via Docker + daemon control.

---

## 🧠 Internal Architecture (based on code structure)

Your backend is structured as a **feature-based Rust service**, split into:

### 📡 API Layer

* HTTP REST endpoints
* WebSocket upgrade handlers
* Request validation & routing

---

### 🔐 Auth Layer

* Token-based authentication
* Role-based access control (RBAC)
* Server-scoped permissions

---

### 🎮 Server Manager Core

* Server lifecycle state machine:

  * create
  * start
  * stop
  * restart
* Server registry / tracking
* Runtime state handling

---

### 📦 SteamCMD Installer

* Automated game installation pipeline
* AppID-based installs
* Server definition handling
* Update workflow for existing servers

---

### 🐳 Docker Layer

* Container creation & deletion
* Volume mapping (persistent data)
* Environment setup per server
* Isolation per instance

---

### 📡 Event / Realtime System

* WebSocket event streaming
* Log forwarding
* Server state updates
* Live console output (PTY stream)

---

### ⚙️ Daemon Communication

* Backend → daemon command dispatch
* Execution delegation (OS-level operations)
* Runtime monitoring feedback loop

---

## 🚀 Getting Started

### Requirements

* Rust (latest stable)
* Docker Engine running
* SteamCMD (optional or containerized depending on setup)
* Linux recommended for production

---

### Build

```bash
cargo build --release
```

---

### Run

```bash
cargo run --release
```

Default API:

```text
http://localhost:5000
```

---

## 📡 API Overview

### Server Management

* `POST /api/servers/create`
* `POST /api/servers/start`
* `POST /api/servers/stop`
* `POST /api/servers/restart`
* `GET  /api/servers/:id`

---

### Realtime WebSocket

* `/ws/events`
* `/ws/logs/:server_id`
* `/ws/console/:server_id`

---

## 🔐 Security Model

Because this system executes infrastructure-level operations:

* All game servers run inside isolated Docker containers
* No direct host filesystem access for game instances
* Authentication required for all API calls
* RBAC restricts server-level operations
* Daemon layer isolates execution privileges

---

## 🧠 Design Principles

### ⚡ Async-first Rust architecture

Backend is designed for:

* concurrent server management
* non-blocking I/O
* event-driven communication

---

### 🐳 Container-first isolation

Every server:

* runs in its own Docker container
* has isolated runtime environment
* uses persistent volume mappings

---

### 📡 Real-time control system

No polling-based UI:

* WebSocket streaming
* instant state propagation
* live log forwarding

---

### 🔧 Extensible game definitions

New games are added via:

* configuration / install definitions
* not hardcoded logic changes

---

## 📁 Project Structure (backend)

```text
backend/
├── api/          # REST + WebSocket endpoints
├── auth/         # authentication + RBAC
├── core/         # orchestration engine
├── docker/       # Docker abstraction layer
├── steamcmd/     # installation system
├── daemon/       # execution communication layer
├── events/       # event system (logs, updates)
├── models/       # shared data structures
└── config/       # runtime configuration
```

---

## 📌 Notes

* This is a **self-hosted orchestration system**, not a SaaS panel
* Designed for single-node deployment (multi-node possible future extension)
* Actively developed; APIs may change
* Focus is reliability of server lifecycle control

---
