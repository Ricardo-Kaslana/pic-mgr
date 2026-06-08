import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../views/WorkspaceHome.vue'),
    },
    {
      path: '/workspace/:id',
      name: 'browser',
      component: () => import('../views/BrowserView.vue'),
      props: true,
    },
    {
      path: '/dedup/:id',
      name: 'dedup',
      component: () => import('../views/DedupView.vue'),
      props: true,
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('../views/SettingsPage.vue'),
    },
  ],
})

export default router
