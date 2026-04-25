import axios from 'axios';
import { getValidStoredToken } from './utils/auth';

export interface ApiResponse<T> {
  data: T;
}

const api = axios.create({
  baseURL: '/api',
});

// Interceptor to attach the token to every request
api.interceptors.request.use((config) => {
  const token = getValidStoredToken();
  if (token && config.headers) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
}, (error) => {
  return Promise.reject(error);
});

api.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');

      if (window.location.pathname !== '/login') {
        window.location.href = '/login';
      }
    }

    return Promise.reject(error);
  },
);

export default api;
