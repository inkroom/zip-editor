import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import Reader from '../views/Reader.vue'

import List from '../views/List.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: 'home',
      component: HomeView
    },
    {
      path: '/:file',
      name: 'home2',
      component: HomeView
    },
    {
      path: '/reader/:file',
      name: 'reader',
      component: Reader
    },
    {
      path:'/list',
      name:'list',
      component: List
    }
  ]
})

export default router
