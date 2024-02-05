import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
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
      path:'/list',
      name:'list',
      component: List
    }
  ]
})

export default router
