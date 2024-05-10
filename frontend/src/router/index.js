import PackageView from '@/views/PackageView.vue'
import SearchView from '@/views/SearchView.vue'
import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/',
      name: "main",
      component: SearchView,
    },
    {
      path: '/package/:name',
      name: 'package',
      component: PackageView,
      props: (route) => ({ ...route.query, ...route.params })
    }
  ]
})

export default router
