<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/authStore';

const router = useRouter();
const authStore = useAuthStore();

const email = ref('admin@example.com');
const password = ref('password');

const handleSubmit = async () => {
  try {
    await authStore.login(email.value, password.value);
    router.push('/');
  } catch (error) {
    // Error is handled in store
  }
};
</script>

<template>
  <div class="min-h-screen w-full flex items-center justify-center bg-background p-4">
    <div class="w-full max-w-md">
      <div class="bg-surface rounded-xl shadow-2xl border border-surface-hover p-8 overflow-hidden relative">
        <!-- Decorative gradient -->
        <div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-primary to-primary-hover"></div>
        
        <div class="mb-8 text-center">
          <h1 class="text-2xl font-bold text-white mb-2">Welcome Back</h1>
          <p class="text-gray-400 text-sm">Sign in to manage your servers</p>
        </div>

        <form @submit.prevent="handleSubmit" class="space-y-6">
          <div v-if="authStore.error" class="bg-danger/10 text-danger text-sm p-3 rounded-md border border-danger/20">
            {{ authStore.error }}
          </div>
          
          <div class="space-y-2">
            <label class="text-sm font-medium text-gray-300">Email Address</label>
            <input 
              v-model="email"
              type="email" 
              class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
              placeholder="admin@example.com"
              required
            />
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium text-gray-300">Password</label>
            <input 
              v-model="password"
              type="password" 
              class="w-full bg-background border border-surface-hover rounded-md px-4 py-2 text-gray-200 focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary transition-colors"
              placeholder="••••••••"
              required
            />
          </div>

          <button 
            type="submit" 
            :disabled="authStore.loading"
            class="w-full bg-primary hover:bg-primary-hover text-white font-medium py-2 px-4 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center min-h-[40px]"
          >
            <span v-if="authStore.loading" class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
            <span v-else>Sign In</span>
          </button>
        </form>
      </div>
    </div>
  </div>
</template>
