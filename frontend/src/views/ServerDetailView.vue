<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useServerStore } from '../stores/serverStore';
import { ArrowLeft, Play, Square, RefreshCw, Terminal, Settings as SettingsIcon, Activity, Trash2, FolderOpen, Save, FilePlus, FolderPlus } from 'lucide-vue-next';

const route = useRoute();
const router = useRouter();
const store = useServerStore();

const activeTab = ref('overview'); // overview, console, files, settings

const serverId = route.params.id as string;

// Console logs state
const logs = ref<string[]>([]);
const logResetAt = ref<string | null>(null);
let logInterval: ReturnType<typeof setInterval> | null = null;
const currentDirectory = ref('');
const fileEntries = ref<Array<{ name: string; path: string; is_directory: boolean; size_bytes: number }>>([]);
const selectedFilePath = ref('');
const fileContent = ref('');
const fileLoading = ref(false);
const fileSaving = ref(false);
const fileError = ref<string | null>(null);
const isDragActive = ref(false);
const uploadInProgress = ref(false);

async function refreshServerState() {
  await store.fetchServer(serverId);

  try {
    const lines = await store.fetchServerLogs(serverId);
    logs.value = lines
      .filter((entry) => !logResetAt.value || entry.timestamp >= logResetAt.value)
      .map((entry) => entry.line);
  } catch (error: any) {
    if (error?.response?.status !== 404) {
      console.error(error);
    }
  }
}

async function loadDirectory(path = currentDirectory.value) {
  fileLoading.value = true;
  fileError.value = null;

  try {
    const response = await store.listServerFiles(serverId, path);
    currentDirectory.value = response.path;
    fileEntries.value = response.entries;
  } catch (error: any) {
    fileError.value = error.response?.data?.error || 'Failed to load files';
  } finally {
    fileLoading.value = false;
  }
}

async function openFile(path: string) {
  fileLoading.value = true;
  fileError.value = null;

  try {
    const response = await store.readServerFile(serverId, path);
    selectedFilePath.value = response.path;
    fileContent.value = response.content;
  } catch (error: any) {
    fileError.value = error.response?.data?.error || 'Failed to open file';
  } finally {
    fileLoading.value = false;
  }
}

async function saveFile() {
  if (!selectedFilePath.value) return;

  fileSaving.value = true;
  fileError.value = null;
  try {
    await store.writeServerFile(serverId, selectedFilePath.value, fileContent.value);
    await loadDirectory(currentDirectory.value);
  } catch (error: any) {
    fileError.value = error.response?.data?.error || 'Failed to save file';
  } finally {
    fileSaving.value = false;
  }
}

async function createFile() {
  const name = window.prompt('New file path', currentDirectory.value ? `${currentDirectory.value}/new-file.txt` : 'new-file.txt');
  if (!name) return;

  fileError.value = null;
  try {
    await store.writeServerFile(serverId, name, '');
    await loadDirectory(currentDirectory.value);
    await openFile(name);
    activeTab.value = 'files';
  } catch (error: any) {
    fileError.value = error.response?.data?.error || 'Failed to create file';
  }
}

async function createDirectory() {
  const name = window.prompt('New folder path', currentDirectory.value ? `${currentDirectory.value}/new-folder` : 'new-folder');
  if (!name) return;

  fileError.value = null;
  try {
    await store.createServerDirectory(serverId, name);
    await loadDirectory(currentDirectory.value);
  } catch (error: any) {
    fileError.value = error.response?.data?.error || 'Failed to create folder';
  }
}

function joinExplorerPath(basePath: string, name: string) {
  return basePath ? `${basePath}/${name}` : name;
}

async function uploadFiles(files: FileList | File[]) {
  const queuedFiles = Array.from(files);
  if (queuedFiles.length === 0) return;

  uploadInProgress.value = true;
  fileError.value = null;

  try {
    for (const file of queuedFiles) {
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      let binary = '';
      for (const byte of bytes) {
        binary += String.fromCharCode(byte);
      }

      await store.uploadServerFile(serverId, joinExplorerPath(currentDirectory.value, file.name), btoa(binary));
    }

    await loadDirectory(currentDirectory.value);
  } catch (error: any) {
    fileError.value = error.response?.data?.error || 'Failed to upload files';
  } finally {
    uploadInProgress.value = false;
    isDragActive.value = false;
  }
}

function onFileInputChange(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files) {
    void uploadFiles(target.files);
  }
  target.value = '';
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
  isDragActive.value = true;
}

function onDragLeave(event: DragEvent) {
  event.preventDefault();
  const relatedTarget = event.relatedTarget as Node | null;
  if (!relatedTarget || !(event.currentTarget as HTMLElement).contains(relatedTarget)) {
    isDragActive.value = false;
  }
}

function onDrop(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer?.files?.length) {
    void uploadFiles(event.dataTransfer.files);
  } else {
    isDragActive.value = false;
  }
}

async function openDirectory(path: string) {
  selectedFilePath.value = '';
  fileContent.value = '';
  await loadDirectory(path);
}

onMounted(async () => {
  await refreshServerState();
  await loadDirectory('');

  logInterval = setInterval(async () => {
    await refreshServerState();
  }, 2500);
});

onUnmounted(() => {
  if (logInterval) clearInterval(logInterval);
});

const server = computed(() => store.servers.find(s => s.id === serverId));
const showRustStartupHint = computed(() => server.value?.gameKind === 'rust' && server.value?.status === 'running');
const parentDirectory = computed(() => {
  if (!currentDirectory.value) return '';
  const parts = currentDirectory.value.split('/');
  parts.pop();
  return parts.join('/');
});

const getStatusColor = (status: string | undefined) => {
  if (!status) return 'text-gray-500';
  switch (status) {
    case 'running': return 'text-success';
    case 'stopped': return 'text-gray-500';
    case 'starting':
    case 'stopping': return 'text-warning';
    default: return 'text-gray-500';
  }
};

const handleAction = async (action: 'start' | 'stop' | 'restart') => {
  logs.value = [];
  logResetAt.value = new Date().toISOString();
  if (action === 'start') await store.startServer(serverId);
  if (action === 'stop') await store.stopServer(serverId);
  if (action === 'restart') await store.restartServer(serverId);
};

const handleDelete = async () => {
  if (confirm('Are you sure you want to delete this server?')) {
    await store.deleteServer(serverId);
    router.push('/');
  }
};
</script>

<template>
  <div v-if="!server" class="flex justify-center py-12">
    <div class="w-8 h-8 border-4 border-surface border-t-primary rounded-full animate-spin"></div>
  </div>
  
  <div v-else class="h-full flex flex-col min-h-0">
    <!-- Header -->
    <div class="mb-6 flex flex-col md:flex-row md:items-center justify-between gap-4 shrink-0">
      <div class="flex items-center gap-4">
        <button @click="router.push('/')" class="p-2 hover:bg-surface rounded-md transition-colors text-gray-400 hover:text-white">
          <ArrowLeft class="w-5 h-5" />
        </button>
        <div>
          <h1 class="text-3xl font-bold text-white">{{ server.name }}</h1>
          <div class="flex items-center gap-2 mt-1">
            <span class="w-2.5 h-2.5 rounded-full" :class="{'bg-success': server.status === 'running', 'bg-gray-500': server.status === 'stopped', 'bg-warning animate-pulse': server.status === 'starting' || server.status === 'stopping'}"></span>
            <span class="text-sm font-medium capitalize" :class="getStatusColor(server.status)">{{ server.status }}</span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-3 bg-surface p-2 rounded-lg border border-surface-hover">
        <button @click="handleAction('start')" :disabled="server.status !== 'stopped'" class="p-2 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed hover:bg-surface-hover text-success" title="Start">
           <Play class="w-5 h-5" />
        </button>
        <button @click="handleAction('stop')" :disabled="server.status === 'stopped'" class="p-2 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed hover:bg-surface-hover text-danger" title="Stop">
           <Square class="w-5 h-5" />
        </button>
        <button @click="handleAction('restart')" :disabled="server.status === 'stopped'" class="p-2 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed hover:bg-surface-hover text-primary" title="Restart">
           <RefreshCw class="w-5 h-5" />
        </button>
      </div>
    </div>

    <!-- Tabs -->
    <div class="flex border-b border-surface-hover mb-6 shrink-0">
      <button 
        @click="activeTab = 'overview'" 
        class="px-4 py-3 font-medium text-sm transition-colors flex items-center gap-2 border-b-2"
        :class="activeTab === 'overview' ? 'border-primary text-primary' : 'border-transparent text-gray-400 hover:text-gray-200'"
      >
        <Activity class="w-4 h-4" /> Overview
      </button>
      <button 
        @click="activeTab = 'console'" 
        class="px-4 py-3 font-medium text-sm transition-colors flex items-center gap-2 border-b-2"
        :class="activeTab === 'console' ? 'border-primary text-primary' : 'border-transparent text-gray-400 hover:text-gray-200'"
      >
        <Terminal class="w-4 h-4" /> Console
      </button>
      <button 
        @click="activeTab = 'files'" 
        class="px-4 py-3 font-medium text-sm transition-colors flex items-center gap-2 border-b-2"
        :class="activeTab === 'files' ? 'border-primary text-primary' : 'border-transparent text-gray-400 hover:text-gray-200'"
      >
        <FolderOpen class="w-4 h-4" /> Files
      </button>
      <button 
        @click="activeTab = 'settings'" 
        class="px-4 py-3 font-medium text-sm transition-colors flex items-center gap-2 border-b-2"
        :class="activeTab === 'settings' ? 'border-primary text-primary' : 'border-transparent text-gray-400 hover:text-gray-200'"
      >
        <SettingsIcon class="w-4 h-4" /> Settings
      </button>
    </div>

    <!-- Tab Content -->
    <div class="flex-1 min-h-0 overflow-y-auto">
      
      <!-- Overview Tab -->
      <div v-show="activeTab === 'overview'" class="space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="bg-surface rounded-xl border border-surface-hover p-6">
            <h3 class="text-lg font-medium text-white mb-4">Resource Limits</h3>
            <div class="space-y-4">
              <div class="flex justify-between items-center py-2 border-b border-surface-hover">
                <span class="text-gray-400">Memory</span>
                <span class="font-medium text-gray-200">{{ server.memoryLimit }} MB</span>
              </div>
              <div class="flex justify-between items-center py-2 border-b border-surface-hover">
                <span class="text-gray-400">CPU</span>
                <span class="font-medium text-gray-200">{{ server.cpuLimit }}%</span>
              </div>
            </div>
          </div>
          
          <div class="bg-surface rounded-xl border border-surface-hover p-6">
            <h3 class="text-lg font-medium text-white mb-4">Information</h3>
            <div class="space-y-4">
              <div class="flex justify-between items-center py-2 border-b border-surface-hover">
                <span class="text-gray-400">Server ID</span>
                <span class="font-mono text-sm text-gray-300">{{ server.id }}</span>
              </div>
              <div class="flex justify-between items-center py-2 border-b border-surface-hover">
                <span class="text-gray-400">Game Type</span>
                <span class="text-sm font-medium uppercase tracking-[0.2em] text-gray-300">{{ server.gameKind }}</span>
              </div>
              <div class="flex justify-between items-center py-2 border-b border-surface-hover">
                <span class="text-gray-400">Created At</span>
                <span class="text-sm text-gray-300">{{ new Date(server.createdAt).toLocaleString() }}</span>
              </div>
              <div class="flex justify-between items-center py-2 border-b border-surface-hover">
                <span class="text-gray-400">Allocated Port</span>
                <span class="text-sm text-gray-300">{{ server.allocatedPort }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Console Tab -->
      <div v-show="activeTab === 'console'" class="h-full bg-background rounded-xl border border-surface-hover shadow-inner flex flex-col font-mono text-sm min-h-[400px]">
        <div v-if="showRustStartupHint" class="border-b border-surface-hover bg-primary/10 px-4 py-3 font-sans text-sm text-gray-300">
          Rust servers can take several minutes on first boot while Steam downloads and installs the dedicated server files.
        </div>
        <div class="flex-1 p-4 overflow-y-auto" style="display: flex; flex-direction: column-reverse;">
          <div class="flex flex-col gap-1">
            <div v-for="(line, idx) in logs" :key="idx" class="whitespace-pre-wrap flex">
              <span class="text-gray-500 mr-4 select-none">{{ idx.toString().padStart(4, '0') }}</span>
              <span :class="{'text-warning': line.includes('[Warn]'), 'text-primary': line.includes('[System]'), 'text-gray-300': !line.includes('[Warn]') && !line.includes('[System]')}">{{ line }}</span>
            </div>
            <div v-if="server.status === 'stopped'" class="text-gray-500 mt-2 italic">Server is offline. Console detached.</div>
          </div>
        </div>
        <div class="p-3 bg-surface border-t border-surface-hover flex gap-2">
          <input type="text" placeholder="Type a command..." class="flex-1 bg-background border border-surface-hover rounded-md px-3 py-1.5 focus:outline-none focus:border-primary text-gray-200" :disabled="server.status !== 'running'">
          <button class="bg-primary/20 text-primary px-4 py-1.5 rounded-md hover:bg-primary hover:text-white transition-colors" :disabled="server.status !== 'running'">Send</button>
        </div>
      </div>

      <div v-show="activeTab === 'files'" class="h-full min-h-[500px] rounded-xl border border-surface-hover bg-surface overflow-hidden">
        <div class="border-b border-surface-hover px-4 py-3 flex flex-col md:flex-row md:items-center justify-between gap-3">
          <div class="text-sm text-gray-300 font-mono">{{ currentDirectory || '/' }}</div>
          <div class="flex items-center gap-2">
            <label class="px-3 py-2 rounded-md text-sm text-gray-300 hover:text-white hover:bg-surface-hover flex items-center gap-2 cursor-pointer">
              <input type="file" class="hidden" multiple @change="onFileInputChange" />
              Upload
            </label>
            <button
              type="button"
              @click="openDirectory(parentDirectory)"
              :disabled="!currentDirectory"
              class="px-3 py-2 rounded-md text-sm text-gray-300 hover:text-white hover:bg-surface-hover disabled:opacity-50 disabled:cursor-not-allowed"
            >
              Up
            </button>
            <button
              type="button"
              @click="createFile"
              class="px-3 py-2 rounded-md text-sm text-gray-300 hover:text-white hover:bg-surface-hover flex items-center gap-2"
            >
              <FilePlus class="w-4 h-4" /> New File
            </button>
            <button
              type="button"
              @click="createDirectory"
              class="px-3 py-2 rounded-md text-sm text-gray-300 hover:text-white hover:bg-surface-hover flex items-center gap-2"
            >
              <FolderPlus class="w-4 h-4" /> New Folder
            </button>
            <button
              type="button"
              @click="saveFile"
              :disabled="!selectedFilePath || fileSaving"
              class="px-3 py-2 rounded-md text-sm bg-primary/20 text-primary hover:bg-primary hover:text-white disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
            >
              <Save class="w-4 h-4" /> Save
            </button>
          </div>
        </div>

        <div v-if="fileError" class="mx-4 mt-4 rounded-md border border-danger/30 bg-danger/10 p-3 text-sm text-danger">
          {{ fileError }}
        </div>

        <div
          class="grid grid-cols-1 lg:grid-cols-[320px_minmax(0,1fr)] h-[calc(100%-61px)] relative"
          :class="isDragActive ? 'ring-2 ring-primary ring-inset bg-primary/5' : ''"
          @dragover="onDragOver"
          @dragleave="onDragLeave"
          @drop="onDrop"
        >
          <div
            v-if="isDragActive || uploadInProgress"
            class="absolute inset-0 z-10 flex items-center justify-center bg-background/80 backdrop-blur-sm text-center px-6"
          >
            <div>
              <div class="text-lg font-semibold text-white">
                {{ uploadInProgress ? 'Uploading files...' : 'Drop files anywhere in this explorer to upload them' }}
              </div>
              <p class="mt-2 text-sm text-gray-400">
                Files are uploaded into {{ currentDirectory || '/' }}.
              </p>
            </div>
          </div>
          <div class="border-r border-surface-hover overflow-y-auto">
            <div v-if="fileLoading && fileEntries.length === 0" class="p-4 text-sm text-gray-400">Loading files...</div>
            <button
              v-for="entry in fileEntries"
              :key="entry.path"
              type="button"
              @click="entry.is_directory ? openDirectory(entry.path) : openFile(entry.path)"
              class="w-full px-4 py-3 text-left border-b border-surface-hover/60 hover:bg-surface-hover transition-colors"
              :class="selectedFilePath === entry.path ? 'bg-primary/10' : ''"
            >
              <div class="flex items-center justify-between gap-3">
                <div class="min-w-0">
                  <div class="text-sm font-medium text-gray-200 truncate">{{ entry.name }}</div>
                  <div class="text-xs text-gray-500 truncate">{{ entry.path }}</div>
                </div>
                <div class="text-xs text-gray-500 whitespace-nowrap">
                  {{ entry.is_directory ? 'Folder' : `${entry.size_bytes} B` }}
                </div>
              </div>
            </button>
            <div v-if="!fileLoading && fileEntries.length === 0" class="p-4 text-sm text-gray-500">
              This folder is empty.
            </div>
          </div>

          <div class="flex flex-col min-h-0">
            <div class="px-4 py-3 border-b border-surface-hover text-sm text-gray-300 font-mono">
              {{ selectedFilePath || 'Select a text file to view or edit it.' }}
            </div>
            <textarea
              v-model="fileContent"
              class="flex-1 min-h-0 w-full bg-background px-4 py-4 text-sm font-mono text-gray-200 focus:outline-none resize-none"
              :placeholder="selectedFilePath ? '' : 'Choose a file from the explorer to start editing.'"
              :disabled="!selectedFilePath || fileLoading"
            />
          </div>
        </div>
      </div>

      <!-- Settings Tab -->
      <div v-show="activeTab === 'settings'" class="space-y-6">
        <div class="bg-surface rounded-xl border border-danger/30 p-6">
          <div class="flex items-start justify-between">
            <div>
              <h3 class="text-lg font-medium text-danger mb-1">Delete Server</h3>
              <p class="text-gray-400 text-sm">This action cannot be undone. All files and configurations will be permanently removed.</p>
            </div>
            <button @click="handleDelete" class="bg-danger/20 text-danger hover:bg-danger hover:text-white px-4 py-2 rounded-md font-medium transition-colors flex items-center gap-2">
               <Trash2 class="w-4 h-4" /> Delete
            </button>
          </div>
        </div>
      </div>

    </div>
  </div>
</template>
