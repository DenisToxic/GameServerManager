import { createRouter, createWebHistory } from 'vue-router';
import { useAuthStore } from '../stores/authStore';
import AdminLayout from '../layouts/AdminLayout.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
      meta: { requiresGuest: true }
    },
    {
      path: '/',
      component: AdminLayout,
      meta: { requiresAuth: true },
      children: [
        {
          path: '',
          name: 'dashboard',
          component: () => import('../views/DashboardView.vue'),
        },
        {
          path: 'servers',
          name: 'servers',
          component: () => import('../views/DashboardView.vue'),
        },
        {
          path: 'servers/create',
          name: 'createServer',
          component: () => import('../views/CreateServerView.vue'),
        },
        {
          path: 'servers/:id',
          name: 'serverDetail',
          component: () => import('../views/ServerDetailView.vue'),
        }
      ]
    }
  ]
});

router.beforeEach((to, _from, next) => {
  const authStore = useAuthStore();
  const isAuthenticated = authStore.isAuthenticated;

  if (to.meta.requiresAuth && !isAuthenticated) {
    next('/login');
  } else if (to.meta.requiresGuest && isAuthenticated) {
    next('/');
  } else {
    next();
  }
});

export default router;
