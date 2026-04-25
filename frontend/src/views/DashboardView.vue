<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';
import { useServerStore } from '../stores/serverStore';
import { Cpu, Activity, Plus } from 'lucide-vue-next';
import { useRouter } from 'vue-router';

const store = useServerStore();
const router = useRouter();

onMounted(() => {
  store.fetchServers();
  store.startPolling();
});

onUnmounted(() => {
  store.stopPolling();
});

const getStatusColor = (status: string) => {
  switch (status) {
    case 'running': return 'bg-success';
    case 'stopped': return 'bg-gray-500';
    case 'starting':
    case 'stopping': return 'bg-warning animate-pulse';
    default: return 'bg-gray-500';
  }
};
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-3xl font-bold text-white">Servers</h1>
      <button 
        @click="router.push('/servers/create')" 
        class="bg-primary hover:bg-primary-hover text-white px-4 py-2 rounded-md font-medium transition-colors flex items-center gap-2"
      >
        <Plus class="w-4 h-4" />
        Create Server
      </button>
    </div>

    <div v-if="store.loading && store.servers.length === 0" class="flex justify-center py-12">
      <div class="w-8 h-8 border-4 border-surface border-t-primary rounded-full animate-spin"></div>
    </div>
    
    <div v-else-if="store.servers.length === 0" class="bg-surface rounded-xl border border-surface-hover p-12 text-center">
      <Server class="w-12 h-12 text-gray-500 mx-auto mb-4" />
      <h3 class="text-lg font-medium text-white mb-2">No servers found</h3>
      <p class="text-gray-400 mb-6">You don't have any servers yet. Create one to get started.</p>
      <button 
        @click="router.push('/servers/create')" 
        class="bg-primary hover:bg-primary-hover text-white px-4 py-2 rounded-md font-medium transition-colors inline-block"
      >
        Create Server
      </button>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
      <div 
        v-for="server in store.servers" 
        :key="server.id"
        class="bg-surface rounded-xl border border-surface-hover overflow-hidden hover:border-primary/50 transition-colors cursor-pointer group"
        @click="router.push(`/servers/${server.id}`)"
      >
        <div class="p-6 border-b border-surface-hover flex items-start justify-between">
          <div>
            <h3 class="font-semibold text-lg text-white mb-1 group-hover:text-primary transition-colors">{{ server.name }}</h3>
            <div class="flex items-center gap-2 text-sm text-gray-400">
              <span class="rounded-full border border-surface-hover px-2 py-0.5 text-xs font-medium uppercase tracking-[0.2em] text-gray-300">
                {{ server.gameKind }}
              </span>
              <Activity class="w-4 h-4" />
              <span>{{ server.id }}</span>
            </div>
          </div>
          <div class="flex items-center gap-2 text-sm font-medium capitalize">
            <span class="w-2.5 h-2.5 rounded-full" :class="getStatusColor(server.status)"></span>
            <span :class="{'text-success': server.status === 'running', 'text-gray-400': server.status === 'stopped', 'text-warning': server.status === 'starting' || server.status === 'stopping'}">
              {{ server.status }}
            </span>
          </div>
        </div>

        <div class="px-6 py-4 bg-surface-hover/20 grid grid-cols-2 gap-4">
          <div class="flex items-center gap-2 text-sm">
            <Cpu class="w-4 h-4 text-gray-400" />
            <span class="text-gray-300">{{ server.cpuLimit }}%</span>
          </div>
          <div class="flex items-center gap-2 text-sm">
            <div class="w-4 h-4 rounded text-gray-400 font-mono text-xs flex items-center justify-center border border-gray-500">M</div>
            <span class="text-gray-300">{{ server.memoryLimit }} MB</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
