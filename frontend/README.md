🎮 GameServerManager – Frontend

Modern web interface for managing game servers, built for speed, clarity, and real-time control.

This frontend is part of GameServerManager, a Docker-based game server orchestration platform that simplifies deployment, monitoring, and lifecycle management of dedicated servers.

✨ Overview

The frontend provides a central control panel for:

Deploying and managing game servers
Viewing real-time server status and logs
Accessing interactive terminals (PTY)
Managing users and permissions
Monitoring server resources

It is designed as a single-page application (SPA) focused on responsiveness, real-time updates, and operational clarity.

🧱 Key Features
🎛 Server Management
Create, start, stop, restart game servers
View live server state and health
Manage server configurations
📟 Real-Time Console
Web-based terminal access (Xterm.js)
Live command execution
Streamed server logs
📊 Monitoring Dashboard
CPU / RAM / disk usage per server
Runtime status indicators
Process lifecycle visibility
👤 Authentication & Access Control
User authentication system
Role-based access control (admin / user)
Permission-scoped server access
⚡ Real-Time Updates
WebSocket-driven UI updates
Live status synchronization without refresh
Instant log streaming
🧩 Architecture

The frontend is designed to be API-first and stateless, communicating entirely with the backend.

React Frontend (Vite)
        ↓ REST API / WebSocket
Backend API Service
        ↓
Docker Engine + Game Runtime (SteamCMD / containers)

The UI does not directly manage servers—it interacts through the backend orchestration layer.

🛠 Tech Stack
React 18
TypeScript
Vite (build system)
Xterm.js (terminal emulation)
WebSocket (real-time updates)
Axios (API communication)
Tailwind / CSS modules (UI styling)
🚀 Getting Started
Prerequisites
Node.js 18+
Backend API running (GameServerManager backend)
Installation
git clone https://github.com/DenisToxic/GameServerManager.git
cd GameServerManager/app
npm install
Development
npm run dev

Frontend will be available at:

http://localhost:5173
Production build
npm run build
npm run preview
🔌 Backend Dependency

This frontend requires a running instance of the GameServerManager backend.

By default, it expects:

http://localhost:5000

API base URL can be configured via environment variables.
