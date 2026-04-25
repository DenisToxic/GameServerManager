# ⚙️ GameServerManager – Daemon (Rust)

Lightweight **Rust-based execution agent (daemon)** for GameServerManager responsible for handling **local runtime operations, server execution control, and backend-to-host communication**.

This component acts as the **bridge between the orchestration backend and the actual machine running game servers**.

---

## 🧠 Role of the Daemon

The daemon is a **local execution layer** that runs on the host machine where game servers are deployed.

It is responsible for:

* ▶ Executing server start/stop commands issued by the backend
* 🐳 Interacting with Docker runtime (directly or via backend instructions)
* 📡 Streaming logs and runtime output back to backend
* ⚡ Managing long-running game server processes
* 🔁 Handling restart loops and crash recovery signals
* 📊 Reporting runtime state (online/offline/error)

---

## 🧱 System Position

```text
Frontend (Vue)
      ↓
Rust Backend (Orchestrator)
      ↓
Daemon (this service)
      ↓
Docker Engine / OS Process Layer
      ↓
Game Server Instance
```

### Key idea:

The backend does **not directly execute OS-level actions**.

Instead, it delegates sensitive runtime operations to this daemon for:

* security isolation
* reduced privilege exposure
* local execution control

---

## ⚙️ Core Responsibilities

### 🎮 Server Execution Control

* Start game server processes or containers
* Stop running instances safely
* Restart servers on demand or failure

---

### 📡 Log Streaming

* Capture stdout/stderr from game servers
* Stream logs back to backend in real-time
* Maintain structured log channels per server instance

---

### 🔄 Lifecycle Monitoring

* Detect crashed or stopped servers
* Report status changes to backend
* Optional automatic recovery triggers

---

### 🐳 Docker / Process Integration

Depending on configuration, daemon may:

* Execute Docker CLI commands
* Or interact via backend-provided instructions
* Or manage local process execution directly

---

## 📡 Communication Model

The daemon communicates with the backend using a **network-based control channel** (likely HTTP/WebSocket or similar async transport).

### Typical flow:

1. Backend sends command:

   * `start server X`
   * `stop server Y`

2. Daemon executes locally:

   * spawns process or Docker container

3. Daemon streams updates:

   * logs
   * status changes
   * errors

4. Backend forwards updates to frontend UI

---

## 🧠 Design Principles

### ⚡ 1. Minimal & lightweight runtime agent

* no heavy business logic
* no orchestration decisions
* purely execution-focused

---

### 🔐 2. Privilege isolation

* daemon runs with limited required permissions
* backend handles authentication & authorization
* daemon trusts backend commands but does not decide them

---

### 📡 3. Event-driven execution

* command → execute → stream result
* no polling loops where possible
* reactive lifecycle updates

---

### 🧩 4. Host-level abstraction

* abstracts OS-level execution details
* hides Docker/process complexity from backend

---

## 🚀 Runtime Behavior

At startup, the daemon typically:

1. Initializes communication channel with backend
2. Registers itself as available execution node
3. Waits for incoming commands
4. Spawns async task handlers for:

   * execution
   * logging
   * state updates

---

## 📊 Responsibilities vs Backend

| Feature                | Backend       | Daemon   |
| ---------------------- | ------------- | -------- |
| Authentication         | ✅             | ❌        |
| Decision making        | ✅             | ❌        |
| Server lifecycle logic | ✅             | ❌        |
| Process execution      | ❌             | ✅        |
| Log streaming          | ⚠ aggregation | ✅ source |
| Docker control         | delegated     | optional |

---

## 🧪 Reliability Considerations

Because this service controls live game servers:

* must handle unexpected process termination
* must recover from network disconnects
* must not block on long-running tasks
* must isolate per-server execution state

---

## 📌 Notes

* This daemon is part of a **multi-layer orchestration architecture**
* It is not intended to be used standalone
* Designed for Linux-first deployment environments
* Works in combination with Rust backend + Docker runtime

---

## 🧠 Summary

The daemon is a **minimal Rust execution agent** that:

> turns backend orchestration commands into real system-level actions and streams results back in real time.

It is the final execution layer in the GameServerManager stack.
