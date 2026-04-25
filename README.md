# 🎮 GameServerManager

A self-hosted, container-based **game server orchestration platform** for deploying, managing, and monitoring dedicated game servers using a modular multi-service architecture.

Built with:

* 🧠 Rust (Backend orchestration + daemon core)
* 🖥 Vue + TypeScript (Frontend control panel)
* ⚙️ Rust (Daemon execution layer)
* 🐳 Docker (Server isolation & runtime)

---

## ⚙️ What this project is

GameServerManager is a **full infrastructure system** for game hosting, consisting of:

* a central orchestration backend
* a local execution daemon
* a real-time web control panel
* Docker-based isolated game servers

It replaces manual server setup with a **programmable, API-driven hosting system**.

---

## 🧱 System Architecture

```text
                ┌────────────────────┐
                │   Vue Frontend     │
                │ (Control Panel UI) │
                └─────────┬──────────┘
                          │ REST / WebSocket
                          ▼
                ┌────────────────────┐
                │ Rust Backend API   │
                │ (Orchestrator)     │
                └─────────┬──────────┘
                          │ Commands / Events
                          ▼
                ┌────────────────────┐
                │ Rust Daemon        │
                │ (Execution Agent)  │
                └─────────┬──────────┘
                          │
                          ▼
                ┌────────────────────┐
                │ Docker Engine      │
                │ Game Containers    │
                └────────────────────┘
```

### Core idea:

* Backend decides **what should happen**
* Daemon executes **what actually happens**
* Frontend visualizes **everything in real-time**

---

## 🧩 Components

## 🧠 Rust Backend (Orchestration Layer)

The backend is the **central control plane** of the system.

### Responsibilities:

* Game server lifecycle management
* Docker container orchestration
* SteamCMD installation pipeline
* Authentication & RBAC
* WebSocket event streaming
* Server registry & state tracking
* Communication with daemon

### Key concept:

> The backend never directly executes system-level actions — it delegates them to the daemon.

---

## ⚙️ Rust Daemon (Execution Layer)

The daemon runs on the host machine and performs **actual system operations**.

### Responsibilities:

* Execute start/stop/restart commands
* Manage server runtime processes
* Interact with Docker / OS-level processes
* Stream logs back to backend
* Monitor server health
* Handle crash detection & recovery signals

### Key concept:

> The daemon is a lightweight execution agent — not a decision maker.

---

## 🖥 Vue Frontend (Control Panel)

The frontend provides a **real-time management interface**.

### Features:

* Server dashboard
* Create / manage game servers
* Live console (terminal access)
* Real-time logs (WebSocket)
* Server status monitoring
* Authentication system UI

---

## 🐳 Core Execution Model

Each game server runs in:

* an isolated Docker container
* with persistent volumes
* controlled lifecycle via backend → daemon

---

## 📡 Communication Flow

### 1. User action

Frontend sends request:

```
Create / Start / Stop server
```

### 2. Backend processing

* validates request
* checks permissions
* updates server state
* sends command to daemon

### 3. Daemon execution

* runs Docker / process operations
* streams logs & state updates

### 4. Real-time feedback

* backend forwards updates
* frontend updates UI instantly

---

## 🔐 Security Model

* JWT/token-based authentication
* Role-based access control (admin / manager / viewer)
* Server-scoped permissions
* Docker isolation per game server
* No direct host access from game containers

---

## 🧠 Design Principles

### ⚡ Event-driven architecture

* everything is command + event based
* no polling-based UI updates

### 🐳 Container-first isolation

* every server is fully isolated
* reproducible deployment environment

### ⚙️ Separation of concerns

* backend = orchestration
* daemon = execution
* frontend = visualization

### 📡 Real-time by design

* WebSockets for logs + state
* live console streaming
* instant status updates

---

## 🚀 Getting Started

### Requirements

* Docker Engine
* Rust toolchain
* Node.js (frontend)
* Linux recommended for daemon

---

### 1. Clone repository

```bash
git clone https://github.com/DenisToxic/GameServerManager.git
cd GameServerManager
```

---

### 2. Backend (Rust)

```bash
cd backend
cargo build --release
cargo run
```

---

### 3. Daemon (Rust)

```bash
cd daemon
cargo run
```

---

### 4. Frontend (Vue)

```bash
cd frontend
npm install
npm run dev
```

---

## 📊 What makes this system different

Most game panels:

* tightly couple UI + backend + execution
* rely on monolithic architectures
* are hard to extend or self-host

This system:

* splits orchestration and execution cleanly
* uses a daemon-based architecture (infra-style design)
* is fully container-driven
* supports event-based real-time control

---

## 📌 Current Status

This project is:

* actively developed
* functional as a self-hosted system
* designed for single-node deployments (multi-node possible future extension)

---

## 🧠 Summary

GameServerManager is a **distributed game server orchestration system** consisting of:

* Rust backend (control plane)
* Rust daemon (execution layer)
* Vue frontend (UI layer)
* Docker runtime (isolation layer)

It turns game server hosting into an **API-driven infrastructure system instead of manual server management**.
