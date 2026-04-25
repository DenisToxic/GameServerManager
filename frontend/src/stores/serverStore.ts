import { defineStore } from 'pinia';
import api, { type ApiResponse } from '../api';

export interface Server {
  id: string;
  userId: string;
  nodeId: number;
  gameKind: 'minecraft' | 'rust' | 'hytale';
  serverSettings: Record<string, unknown>;
  name: string;
  status: 'running' | 'stopped' | 'starting' | 'stopping' | 'error';
  dockerContainerId: string | null;
  allocatedPort: number;
  memoryLimit: number;
  cpuLimit: number;
  createdAt: string;
}

interface ServerApi {
  id: string;
  user_id: string;
  node_id: number;
  game_kind: Server['gameKind'];
  server_settings: Record<string, unknown>;
  name: string;
  status: Server['status'];
  docker_container_id: string | null;
  allocated_port: number;
  memory_limit_mb: number;
  cpu_limit_percent: number;
  created_at: string;
}

interface ServerLogLineApi {
  timestamp: string;
  line: string;
}

interface ServerLogsApi {
  server_id: string;
  lines: ServerLogLineApi[];
}

export interface ServerFileEntry {
  name: string;
  path: string;
  is_directory: boolean;
  size_bytes: number;
}

interface ServerFileListApi {
  server_id: string;
  path: string;
  entries: ServerFileEntry[];
}

interface ServerFileContentApi {
  server_id: string;
  path: string;
  content: string;
}

interface CreateServerPayload {
  name: string;
  node_id: number;
  game_kind: Server['gameKind'];
  server_settings: Record<string, unknown>;
  allocated_port: number;
  memory_limit_mb: number;
  cpu_limit_percent: number;
}

function mapServer(server: ServerApi): Server {
  return {
    id: server.id,
    userId: server.user_id,
    nodeId: server.node_id,
    gameKind: server.game_kind,
    serverSettings: server.server_settings,
    name: server.name,
    status: server.status,
    dockerContainerId: server.docker_container_id,
    allocatedPort: server.allocated_port,
    memoryLimit: server.memory_limit_mb,
    cpuLimit: server.cpu_limit_percent,
    createdAt: server.created_at,
  };
}

export const useServerStore = defineStore('server', {
  state: () => ({
    servers: [] as Server[],
    loading: false,
    error: null as string | null,
    pollInterval: null as ReturnType<typeof setInterval> | null,
  }),
  actions: {
    async fetchServers() {
      this.loading = true;
      this.error = null;
      try {
        const response = await api.get<ApiResponse<ServerApi[]>>('/servers');
        this.servers = response.data.data.map(mapServer);
      } catch (err: any) {
        this.error = err.response?.data?.error || err.message || 'Failed to load servers';
      } finally {
        this.loading = false;
      }
    },
    async fetchServer(id: string) {
      this.error = null;
      try {
        const response = await api.get<ApiResponse<ServerApi>>(`/servers/${id}`);
        const server = mapServer(response.data.data);
        const index = this.servers.findIndex((entry) => entry.id === id);
        if (index !== -1) {
          this.servers[index] = server;
        } else {
          this.servers.push(server);
        }
        return server;
      } catch (err: any) {
        this.error = err.response?.data?.error || err.message || 'Failed to load server';
        throw err;
      }
    },
    async fetchServerLogs(id: string) {
      const response = await api.get<ApiResponse<ServerLogsApi>>(`/servers/${id}/logs`);
      return response.data.data.lines;
    },
    async listServerFiles(id: string, path = '') {
      const response = await api.get<ApiResponse<ServerFileListApi>>(`/servers/${id}/files`, {
        params: path ? { path } : {},
      });
      return response.data.data;
    },
    async readServerFile(id: string, path: string) {
      const response = await api.get<ApiResponse<ServerFileContentApi>>(`/servers/${id}/file`, {
        params: { path },
      });
      return response.data.data;
    },
    async writeServerFile(id: string, path: string, content: string) {
      await api.put(`/servers/${id}/file`, { path, content });
    },
    async uploadServerFile(id: string, path: string, contentBase64: string) {
      await api.put(`/servers/${id}/file`, { path, content: '', content_base64: contentBase64 });
    },
    async createServerDirectory(id: string, path: string) {
      await api.post(`/servers/${id}/directories`, { path });
    },
    async createServer(payload: CreateServerPayload) {
      await api.post<ApiResponse<ServerApi>>('/servers', payload);
      await this.fetchServers();
    },
    async deleteServer(id: string) {
      await api.delete(`/servers/${id}`);
      this.servers = this.servers.filter((server) => server.id !== id);
    },
    async startServer(id: string) {
      const server = this.servers.find((entry) => entry.id === id);
      if (server) server.status = 'starting';
      await api.post(`/servers/${id}/start`);
      await this.fetchServer(id);
    },
    async stopServer(id: string) {
      const server = this.servers.find((entry) => entry.id === id);
      if (server) server.status = 'stopping';
      await api.post(`/servers/${id}/stop`);
      await this.fetchServer(id);
    },
    async restartServer(id: string) {
      const server = this.servers.find((entry) => entry.id === id);
      if (server) server.status = 'starting';
      await api.post(`/servers/${id}/restart`);
      await this.fetchServer(id);
    },
    startPolling() {
      if (this.pollInterval) return;
      this.pollInterval = setInterval(() => {
        this.fetchServers();
      }, 5000);
    },
    stopPolling() {
      if (this.pollInterval) {
        clearInterval(this.pollInterval);
        this.pollInterval = null;
      }
    },
  },
});
