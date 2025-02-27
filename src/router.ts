import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteLocationNormalized } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'TaskManager',
    component: () => import('./pages/TaskManager.vue'),
  },
  {
    path: '/config',
    name: 'ConfigManager',
    component: () => import('./pages/ConfigManager.vue'),
  },
  {
    path: '/config/new',
    name: 'CreateConfig',
    component: () => import('./pages/ConfigDetail.vue'),
    props: {
      isCreating: true,
    }
  },
  {
    path: '/config/:name',
    name: 'EditConfig',
    component: () => import('./pages/ConfigDetail.vue'),
    props: (route: RouteLocationNormalized) => ({
      isCreating: false,
      configName: route.params.name
    })
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

export default router 