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
    <el-form-item label="deps">
      {{ deps }}
    </el-form-item>
    <el-form-item label="Build Deps">
      {{ buildDeps }}
    </el-form-item>
    <el-form-item label="Pkg Breaks">
      {{ pkgBreaks }}
    </el-form-item>
    <el-form-item label="Pkg Recommend">
      {{ pkgBreaks }}
    </el-form-item>
    <el-form-item label="Pkg Provides">
      {{ provides }}
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
      provides: [],
    }
  },
  props: {
    name: String,
  },
  async mounted() {
    const resp = await fetch(`http://127.0.0.1:2333/package?name=${this.name}`);
    const data = await resp.json();
    this.pkgName = data.name;
    this.version = data.version;
    this.desc = data.desc;
    this.deps = data.deps;
    this.buildDeps = data.build_deps;
    this.pkgBreaks = data.pkgbreak;
    this.pkgRecommend = data.pkgrecom;
    this.provides = data.provides;
  }
}
</script>
