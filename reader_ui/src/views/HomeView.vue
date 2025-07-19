<template>
  <div class="home">
    <el-card class="welcome-card">
      <template #header>
        <div class="card-header">
          <span>欢迎使用知识积累系统</span>
        </div>
      </template>

      <div class="welcome-content">
        <el-row :gutter="20">
          <el-col :span="8">
            <el-card shadow="hover" class="feature-card">
              <el-icon class="feature-icon"><Collection /></el-icon>
              <h3>知识库管理</h3>
              <p>创建和管理多个知识库，按主题组织学习材料</p>
              <el-button type="primary" @click="goToKnowledgeBases">
                开始使用
              </el-button>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="feature-card">
              <el-icon class="feature-icon"><ChatDotRound /></el-icon>
              <h3>AI智能问答</h3>
              <p>基于知识库内容生成问题，测试知识掌握程度</p>
              <el-button type="primary" @click="goToKnowledgeBases">
                立即体验
              </el-button>
            </el-card>
          </el-col>

          <el-col :span="8">
            <el-card shadow="hover" class="feature-card">
              <el-icon class="feature-icon"><Refresh /></el-icon>
              <h3>学习复习</h3>
              <p>回顾历史问答，巩固学习效果</p>
              <el-button type="primary" @click="goToKnowledgeBases">
                开始复习
              </el-button>
            </el-card>
          </el-col>
        </el-row>

        <div class="stats-section">
          <el-row :gutter="20">
            <el-col :span="6">
              <el-statistic title="知识库数量" :value="knowledgeBasesCount" />
            </el-col>
            <el-col :span="6">
              <el-statistic title="文档总数" :value="documentsCount" />
            </el-col>
            <el-col :span="6">
              <el-statistic title="问答次数" :value="quizCount" />
            </el-col>
            <el-col :span="6">
              <el-statistic title="复习次数" :value="reviewCount" />
            </el-col>
          </el-row>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script>
import { computed, onMounted } from "vue";
import { useStore } from "vuex";
import { useRouter } from "vue-router";
import { Collection, ChatDotRound, Refresh } from "@element-plus/icons-vue";

export default {
  name: "HomeView",
  components: {
    Collection,
    ChatDotRound,
    Refresh,
  },
  setup() {
    const store = useStore();
    const router = useRouter();

    // Computed properties for stats
    const knowledgeBasesCount = computed(
      () => store.state.knowledgeBases.length
    );
    const documentsCount = computed(() => store.state.documents.length);
    const quizCount = computed(() => 0); // TODO: Implement quiz count
    const reviewCount = computed(() => 0); // TODO: Implement review count

    // Methods
    const goToKnowledgeBases = () => {
      router.push("/knowledge-bases");
    };

    // Load initial data
    onMounted(async () => {
      try {
        await store.dispatch("fetchKnowledgeBases");
      } catch (error) {
        console.error("Failed to load knowledge bases:", error);
      }
    });

    return {
      knowledgeBasesCount,
      documentsCount,
      quizCount,
      reviewCount,
      goToKnowledgeBases,
    };
  },
};
</script>

<style lang="scss" scoped>
.home {
  padding: 20px;
}

.welcome-card {
  max-width: 1200px;
  margin: 0 auto;
}

.card-header {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.welcome-content {
  .feature-card {
    text-align: center;
    height: 280px;
    display: flex;
    flex-direction: column;
    justify-content: center;

    .feature-icon {
      font-size: 48px;
      color: #409eff;
      margin-bottom: 16px;
    }

    h3 {
      margin: 16px 0;
      color: #303133;
      font-size: 18px;
    }

    p {
      color: #606266;
      margin-bottom: 20px;
      line-height: 1.6;
    }
  }

  .stats-section {
    margin-top: 40px;
    padding: 20px;
    background-color: #f5f7fa;
    border-radius: 8px;
  }
}

@media (max-width: 768px) {
  .welcome-content {
    .el-col {
      margin-bottom: 20px;
    }

    .feature-card {
      height: auto;
      padding: 20px;
    }
  }
}
</style>
