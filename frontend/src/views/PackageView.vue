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
    <el-form-item label="Deps">
      <div class="links">
        <li v-for="d of deps" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Build Deps">
      <div class="links">
        <li v-for="d of buildDeps" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Pkg Breaks">
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
    <el-form-item label="Pkg Recommend">
      <div class="links">
        <li v-for="d of pkgRecommend" :key="d">
          <el-link :href="'../package/' + d">{{ d }}</el-link>
        </li>
      </div>
    </el-form-item>
    <el-form-item label="Pkg Provides">
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
      pkgName: '',
      desc: '',
      version: '',
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
    const resp = await fetch(`http://127.0.0.1:2333/package?name=${this.name}`)
    const data = await resp.json()
    this.pkgName = data.name
    this.version = data.version
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
