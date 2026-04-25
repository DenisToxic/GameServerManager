<script setup lang="ts">
import { useUiStore } from '../stores/uiStore';
import { useAuthStore } from '../stores/authStore';
import { useRouter } from 'vue-router';
import { LayoutDashboard, Server, LogOut, Menu } from 'lucide-vue-next';

const uiStore = useUiStore();
const authStore = useAuthStore();
const router = useRouter();

const handleLogout = () => {
  authStore.logout();
  router.push('/login');
};
</script>

<template>
  <div class="h-screen w-full flex bg-background text-gray-200 overflow-hidden">
    <!-- Sidebar -->
    <aside 
      :class="[
        'bg-surface border-r border-surface-hover flex flex-col transition-all duration-300 z-20',
        uiStore.sidebarOpen ? 'w-64' : 'w-16'
      ]"
    >
      <div class="h-16 flex items-center justify-center border-b border-surface-hover">
        <div v-if="uiStore.sidebarOpen" class="font-bold text-xl tracking-wider text-primary">PANEL</div>
        <div v-else class="font-bold text-xl text-primary">P</div>
      </div>
      
      <nav class="flex-1 py-4 flex flex-col gap-2 px-2 overflow-y-auto">
        <router-link to="/" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-surface-hover text-sm font-medium transition-colors" active-class="bg-primary/10 text-primary">
          <LayoutDashboard class="w-5 h-5 flex-shrink-0" />
          <span v-if="uiStore.sidebarOpen">Dashboard</span>
        </router-link>
        <router-link to="/servers" class="flex items-center gap-3 px-3 py-2 rounded-md hover:bg-surface-hover text-sm font-medium transition-colors" active-class="bg-primary/10 text-primary">
          <Server class="w-5 h-5 flex-shrink-0" />
          <span v-if="uiStore.sidebarOpen">Servers</span>
        </router-link>
      </nav>

      <div class="p-2 border-t border-surface-hover">
        <button 
          @click="handleLogout" 
          class="w-full flex items-center gap-3 px-3 py-2 rounded-md hover:bg-danger/20 hover:text-danger text-sm font-medium transition-colors"
        >
          <LogOut class="w-5 h-5 flex-shrink-0" />
          <span v-if="uiStore.sidebarOpen">Logout</span>
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Topbar -->
      <header class="h-16 bg-surface border-b border-surface-hover flex items-center px-4 justify-between z-10">
        <button @click="uiStore.toggleSidebar" class="p-2 rounded-md hover:bg-surface-hover text-gray-400 hover:text-gray-100 transition-colors">
          <Menu class="w-5 h-5" />
        </button>

        <div class="flex items-center gap-4">
          <div class="w-8 h-8 rounded-full bg-primary/20 flex items-center justify-center text-primary font-bold text-sm">
            A
          </div>
        </div>
      </header>

      <!-- Page Content -->
      <main class="flex-1 overflow-y-auto p-6">
        <div class="max-w-6xl mx-auto h-full">
          <router-view />
        </div>
      </main>
    </div>
  </div>
</template>
