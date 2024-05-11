<template>
  <el-autocomplete
    v-model="state"
    :fetch-suggestions="querySearchAsync"
    placeholder="Please input"
    @select="handleSelect"
    style="width: 600px"
    size="large"
    :suffix-icon="Search"
  />
</template>

<script setup>
import router from '@/router'
import { Search } from '@element-plus/icons-vue'
import { onMounted, ref } from 'vue'

const state = ref('')
const links = ref([])

const loadAll = async () => {
  const uri = import.meta.env.VITE_MINIPKGSITE
  const resp = await fetch(`${uri}/all`)
  const data = await resp.json()
  // only proceed once second promise is resolved
  return data.map((v) => {
    return {
      value: v
    }
  })
}

let timeout
const querySearchAsync = (queryString, cb) => {
  const results = queryString ? links.value.filter(createFilter(queryString)) : links.value

  clearTimeout(timeout)
  timeout = setTimeout(() => {
    cb(results)
  }, 3000 * Math.random())
}
const createFilter = (queryString) => {
  return (restaurant) => {
    return restaurant.value.indexOf(queryString.toLowerCase()) === 0
  }
}

const handleSelect = (item) => {
  router.push(`/package/${item.value}`)
}

onMounted(async () => {
  links.value = await loadAll()
})
</script>
