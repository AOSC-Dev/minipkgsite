<template>
  <el-form :model="form" label-width="auto" style="max-width: 600px">
    <el-form-item label="Package name">
      {{ pkgName }}
    </el-form-item>
    <el-form-item label="Desc">
      {{ desc }}
    </el-form-item>
    <el-form-item label="Version">
      {{ version }}
    </el-form-item>
    <el-form-item label="Path">
      <div>
        {{ path }}
        <el-link :href="'https://github.com/aosc-dev/aosc-os-abbs/tree/stable/' + path"
          >(Github)</el-link
        >
      </div>
    </el-form-item>
    <el-form-item label="Deps" v-if="deps.length != 0">
      <div class="links">
        <li v-for="d of deps" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Build Deps" v-if="buildDeps.length != 0">
      <div class="links">
        <li v-for="d of buildDeps" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Pkg Breaks" v-if="pkgBreaks.length != 0">
      <div class="break-links">
        <li v-for="d of pkgBreaks" :key="d">
          <div class="breaks">
            <el-link :href="'../package/' + d.name">{{ d.name }}</el-link>
            <span>{{ d.comp }}</span>
            <span class="version">{{ d.version }}</span>
          </div>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Pkg Recommend" v-if="pkgRecommend.length != 0">
      <div class="links">
        <li v-for="d of pkgRecommend" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Pkg Provides" v-if="provides.length != 0">
      <div class="links">
        <li v-for="d of provides" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
  </el-form>
</template>

<script>
export default {
  data() {
    return {
      uri: import.meta.env.VITE_MINIPKGSITE,
      pkgName: '',
      desc: '',
      version: '',
      path: '',
      deps: [],
      buildDeps: [],
      pkgBreaks: [],
      pkgRecommend: [],
      provides: []
    }
  },
  props: {
    name: String
  },
  async mounted() {
    const resp = await fetch(`${this.uri}/package?name=${this.name}`)
    const data = await resp.json()
    this.pkgName = data.name
    this.version = data.version
    this.path = data.path
    this.desc = data.desc
    this.deps = data.deps
    this.buildDeps = data.build_deps
    this.pkgBreaks = data.pkgbreak
    this.pkgRecommend = data.pkgrecom
    this.provides = data.provides
  }
}
</script>

<style>
.links {
  list-style: none;
  display: flex;
  flex-flow: column;
  white-space: normal;
  word-break: keep-all;
  max-width: 300px;
}

.break-links {
  list-style: none;
  display: flex;
  flex-flow: column;
  white-space: normal;
  word-break: keep-all;
}

.el-link {
  margin-right: 8px;
}

.breaks {
  display: flex;
  flex-flow: row;
  word-break: keep-all;
}

.version {
  margin-left: 6px;
}
</style>
