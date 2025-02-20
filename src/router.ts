import { createRouter, createWebHashHistory } from 'vue-router'
import type { RouteLocationNormalized } from 'vue-router'
import TaskManager from './pages/TaskManager.vue'
import ConfigManager from './pages/ConfigManager.vue'
import ConfigDetail from './pages/ConfigDetail.vue'

const routes = [
  {
    path: '/',
    name: 'TaskManager',
    component: TaskManager,
  },
  {
    path: '/config',
    name: 'ConfigManager',
    component: ConfigManager,
  },
  {
    path: '/config/new',
    name: 'CreateConfig',
    component: ConfigDetail,
    props: {
      isCreating: true,
    }
  },
  {
    path: '/config/:name',
    name: 'EditConfig',
    component: ConfigDetail,
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