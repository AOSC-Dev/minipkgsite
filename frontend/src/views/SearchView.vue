<template>
  <el-autocomplete
    v-model="state"
    :fetch-suggestions="querySearchAsync"
    placeholder="Input package name (e.g vim)"
    @select="handleSelect"
    style="width: 600px"
    size="large"
    :suffix-icon="Search"
  />
</template>

<script setup>
import router from '@/router'
import { Search } from '@element-plus/icons-vue'
import { ref } from 'vue'

const state = ref('')

let timeout
const querySearchAsync = async (queryString, cb) => {
  const uri = import.meta.env.VITE_MINIPKGSITE
  const query = queryString.toLowerCase()
  const resp = await fetch(`${uri}/search?name=${query}`)
  const results = await resp.json()

  clearTimeout(timeout)
  timeout = setTimeout(
    () =>
      cb(
        results.map((v) => {
          return {
            value: v
          }
        })
      ),
    500
  )
}

const handleSelect = (item) => {
  router.push(`/package/${item.value}`)
}
</script>
