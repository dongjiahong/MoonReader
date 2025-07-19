<template>
  <div class="knowledge-base-detail">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>知识库详情</span>
        </div>
      </template>

      <div class="content">
        <p>知识库详情页面 - 待实现</p>
        <p>知识库ID: {{ $route.params.id }}</p>
      </div>
    </el-card>
  </div>
</template>

<script>
export default {
  name: "KnowledgeBaseDetailView",
};
</script>

<style lang="scss" scoped>
.knowledge-base-detail {
  padding: 20px;
}

.card-header {
  font-size: 20px;
  font-weight: 600;
}

.content {
  text-align: center;
  padding: 40px;
}
</style>
