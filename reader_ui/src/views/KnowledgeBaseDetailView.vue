<template>
  <div class="knowledge-base-detail">
    <!-- Header with breadcrumb -->
    <div class="page-header">
      <el-breadcrumb separator="/">
        <el-breadcrumb-item :to="{ path: '/knowledge-bases' }">
          知识库
        </el-breadcrumb-item>
        <el-breadcrumb-item>
          {{ currentKnowledgeBase?.name || "加载中..." }}
        </el-breadcrumb-item>
      </el-breadcrumb>
    </div>

    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ currentKnowledgeBase?.name || "知识库详情" }}</span>
        </div>
      </template>

      <div class="content">
        <div class="kb-info">
          <h3>知识库信息</h3>
          <div class="info-item">
            <label>名称：</label>
            <span>{{ currentKnowledgeBase?.name || "加载中..." }}</span>
          </div>
          <div class="info-item">
            <label>描述：</label>
            <span>{{ currentKnowledgeBase?.description || "暂无描述" }}</span>
          </div>
          <div class="info-item">
            <label>创建时间：</label>
            <span>{{ formatDate(currentKnowledgeBase?.created_at) }}</span>
          </div>
          <div class="info-item">
            <label>文档数量：</label>
            <span>{{ currentKnowledgeBase?.document_count || 0 }} 个</span>
          </div>
        </div>

        <div class="actions">
          <h3>操作</h3>
          <div class="action-buttons">
            <el-button
              type="primary"
              size="large"
              @click="
                $router.push(`/knowledge-bases/${knowledgeBaseId}/documents`)
              "
            >
              <el-icon><Document /></el-icon>
              文档管理
            </el-button>
            <el-button
              type="success"
              size="large"
              @click="$router.push(`/knowledge-bases/${knowledgeBaseId}/quiz`)"
            >
              <el-icon><EditPen /></el-icon>
              AI 问答
            </el-button>
            <el-button
              type="warning"
              size="large"
              @click="
                $router.push(`/knowledge-bases/${knowledgeBaseId}/review`)
              "
            >
              <el-icon><View /></el-icon>
              复习模式
            </el-button>
            <el-button
              type="info"
              size="large"
              @click="
                $router.push(`/knowledge-bases/${knowledgeBaseId}/history`)
              "
            >
              <el-icon><Clock /></el-icon>
              学习历史
            </el-button>
          </div>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script>
import { mapState, mapGetters, mapActions } from "vuex";
import { Document, EditPen, View, Clock } from "@element-plus/icons-vue";
import { ElMessage } from "element-plus";

export default {
  name: "KnowledgeBaseDetailView",
  components: {
    Document,
    EditPen,
    View,
    Clock,
  },
  computed: {
    ...mapState(["knowledgeBases"]),
    ...mapGetters(["getKnowledgeBaseById"]),

    knowledgeBaseId() {
      return this.$route.params.id;
    },

    currentKnowledgeBase() {
      return this.getKnowledgeBaseById(this.knowledgeBaseId);
    },
  },
  async created() {
    await this.loadData();
  },
  methods: {
    ...mapActions(["fetchKnowledgeBases"]),

    async loadData() {
      try {
        // Load knowledge bases if not already loaded
        if (this.knowledgeBases.length === 0) {
          await this.fetchKnowledgeBases();
        }
      } catch (error) {
        ElMessage.error("加载数据失败");
      }
    },

    formatDate(dateString) {
      if (!dateString) return "";
      const date = new Date(dateString);
      return date.toLocaleString("zh-CN");
    },
  },
};
</script>

<style lang="scss" scoped>
.knowledge-base-detail {
  padding: 20px;
}

.page-header {
  margin-bottom: 20px;
}

.card-header {
  font-size: 20px;
  font-weight: 600;
}

.content {
  padding: 20px;
}

.kb-info {
  margin-bottom: 40px;

  h3 {
    margin-bottom: 20px;
    color: #303133;
    font-size: 18px;
  }
}

.info-item {
  display: flex;
  margin-bottom: 12px;
  align-items: center;

  label {
    font-weight: 500;
    color: #606266;
    width: 100px;
    flex-shrink: 0;
  }

  span {
    color: #303133;
  }
}

.actions {
  h3 {
    margin-bottom: 20px;
    color: #303133;
    font-size: 18px;
  }
}

.action-buttons {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;

  .el-button {
    height: 60px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 16px;

    .el-icon {
      font-size: 20px;
    }
  }
}

// Responsive design
@media (max-width: 768px) {
  .knowledge-base-detail {
    padding: 10px;
  }

  .content {
    padding: 10px;
  }

  .action-buttons {
    grid-template-columns: 1fr;
    gap: 12px;

    .el-button {
      height: 50px;
      font-size: 14px;

      .el-icon {
        font-size: 18px;
      }
    }
  }

  .info-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;

    label {
      width: auto;
    }
  }
}
</style>
