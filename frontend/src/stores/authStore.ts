import { defineStore } from 'pinia';
import api, { type ApiResponse } from '../api';
import { getValidStoredToken } from '../utils/auth';

interface User {
  id: string;
  email: string;
}

interface AuthPayload {
  access_token: string;
  token_type: string;
  expires_in_seconds: number;
  user: User;
}

export const useAuthStore = defineStore('auth', {
  state: () => ({
    token: getValidStoredToken(),
    user: null as User | null,
    loading: false,
    error: null as string | null,
  }),
  getters: {
    isAuthenticated: (state) => !!state.token,
  },
  actions: {
    async login(email: string, password: string) {
      this.loading = true;
      this.error = null;
      try {
        const response = await api.post<ApiResponse<AuthPayload>>('/auth/login', { email, password });
        const auth = response.data.data;

        this.token = auth.access_token;
        this.user = auth.user;

        localStorage.setItem('token', this.token);
      } catch (err: any) {
        this.error = err.response?.data?.error || 'Login failed';
        throw err;
      } finally {
        this.loading = false;
      }
    },
    logout() {
      this.token = null;
      this.user = null;
      localStorage.removeItem('token');
      // Typically router push to login happens in a component or router interceptor
    }
  }
});
