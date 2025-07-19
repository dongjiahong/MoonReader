<template>
  <div class="ai-quiz">
    <!-- Header Card -->
    <el-card class="header-card">
      <template #header>
        <div class="card-header">
          <span>AI问答</span>
          <div class="header-actions">
            <el-button
              type="primary"
              :loading="loading"
              :disabled="!hasDocuments"
              @click="generateNewQuestion"
            >
              <el-icon><Refresh /></el-icon>
              生成新问题
            </el-button>
          </div>
        </div>
      </template>

      <!-- Empty State -->
      <div v-if="!hasDocuments" class="empty-state">
        <el-empty description="此知识库暂无文档">
          <template #image>
            <el-icon size="60"><Document /></el-icon>
          </template>
          <el-button type="primary" @click="goToDocuments">
            添加文档
          </el-button>
        </el-empty>
      </div>

      <!-- AI Config Warning -->
      <el-alert
        v-if="hasDocuments && !isAIConfigured"
        title="AI服务未配置"
        type="warning"
        show-icon
        :closable="false"
        class="config-alert"
      >
        <template #default>
          <p>请先配置AI服务才能使用问答功能</p>
          <el-button type="text" @click="goToSettings"> 前往设置 </el-button>
        </template>
      </el-alert>
    </el-card>

    <!-- Question Card -->
    <el-card
      v-if="currentQuestion && hasDocuments && isAIConfigured"
      class="question-card"
    >
      <template #header>
        <div class="question-header">
          <el-icon><ChatDotRound /></el-icon>
          <span>问题</span>
          <el-tag type="info" size="small">
            {{ formatDate(currentQuestion.generated_at) }}
          </el-tag>
        </div>
      </template>

      <div class="question-content">
        <p class="question-text">{{ currentQuestion.question_text }}</p>

        <!-- Context snippet if available -->
        <div v-if="currentQuestion.context_snippet" class="context-snippet">
          <el-divider content-position="left">
            <span class="context-label">相关内容</span>
          </el-divider>
          <el-text class="context-text" type="info">
            {{ currentQuestion.context_snippet }}
          </el-text>
        </div>
      </div>
    </el-card>

    <!-- Answer Input Card -->
    <el-card
      v-if="currentQuestion && hasDocuments && isAIConfigured"
      class="answer-card"
    >
      <template #header>
        <div class="answer-header">
          <el-icon><Edit /></el-icon>
          <span>您的回答</span>
        </div>
      </template>

      <div class="answer-content">
        <el-input
          v-model="userAnswer"
          type="textarea"
          :rows="6"
          placeholder="请输入您的回答..."
          :disabled="loading || !!currentAnswerResult"
          class="answer-input"
        />

        <div class="answer-actions">
          <el-button
            v-if="!currentAnswerResult"
            type="primary"
            :loading="loading"
            :disabled="!userAnswer.trim()"
            @click="submitAnswer"
          >
            <el-icon><Check /></el-icon>
            提交答案
          </el-button>

          <el-button
            v-if="currentAnswerResult"
            type="success"
            @click="startNewQuestion"
          >
            <el-icon><Plus /></el-icon>
            开始新问题
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- Answer Result Card -->
    <el-card v-if="currentAnswerResult" class="result-card">
      <template #header>
        <div class="result-header">
          <el-icon><TrophyBase /></el-icon>
          <span>评估结果</span>
          <el-tag
            :type="getScoreType(currentAnswerResult.ai_evaluation.score)"
            size="large"
          >
            {{ currentAnswerResult.ai_evaluation.score }}/100
          </el-tag>
        </div>
      </template>

      <div class="result-content">
        <!-- AI Feedback -->
        <div class="feedback-section">
          <h4>AI评价</h4>
          <el-text class="feedback-text">
            {{ currentAnswerResult.ai_evaluation.feedback }}
          </el-text>
        </div>

        <!-- Suggestions -->
        <div
          v-if="
            currentAnswerResult.ai_evaluation.suggestions &&
            currentAnswerResult.ai_evaluation.suggestions.length > 0
          "
          class="suggestions-section"
        >
          <h4>改进建议</h4>
          <ul class="suggestions-list">
            <li
              v-for="(suggestion, index) in currentAnswerResult.ai_evaluation
                .suggestions"
              :key="index"
              class="suggestion-item"
            >
              {{ suggestion }}
            </li>
          </ul>
        </div>

        <!-- User Answer Review -->
        <div class="answer-review">
          <el-divider content-position="left">
            <span>您的回答</span>
          </el-divider>
          <el-text class="user-answer-text">
            {{ currentAnswerResult.user_answer }}
          </el-text>
        </div>
      </div>
    </el-card>

    <!-- Loading Overlay -->
    <div v-if="loading" class="loading-overlay">
      <el-loading-directive />
    </div>
  </div>
</template>

<script>
import { computed, ref, onMounted, watch } from "vue";
import { useStore } from "vuex";
import { useRoute, useRouter } from "vue-router";
import { ElMessage, ElMessageBox } from "element-plus";
import {
  Refresh,
  Document,
  ChatDotRound,
  Edit,
  Check,
  Plus,
  TrophyBase,
} from "@element-plus/icons-vue";

export default {
  name: "AIQuizView",
  components: {
    Refresh,
    Document,
    ChatDotRound,
    Edit,
    Check,
    Plus,
    TrophyBase,
  },
  setup() {
    const store = useStore();
    const route = useRoute();
    const router = useRouter();

    // Reactive data
    const userAnswer = ref("");
    const currentAnswerResult = ref(null);

    // Computed properties
    const loading = computed(() => store.getters.isLoading);
    const currentQuestion = computed(() => store.state.currentQuestion);
    const currentKnowledgeBase = computed(
      () => store.state.currentKnowledgeBase
    );
    const documents = computed(() =>
      store.getters.getDocumentsByKnowledgeBase(route.params.id)
    );
    const aiConfig = computed(() => store.state.aiConfig);

    const hasDocuments = computed(() => documents.value.length > 0);
    const isAIConfigured = computed(() => {
      const config = aiConfig.value;
      if (config.provider === "local") {
        return !!config.api_url;
      }
      return !!(config.api_key && config.provider);
    });

    // Methods
    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleString("zh-CN");
    };

    const getScoreType = (score) => {
      if (score >= 90) return "success";
      if (score >= 70) return "warning";
      return "danger";
    };

    const generateNewQuestion = async () => {
      try {
        currentAnswerResult.value = null;
        userAnswer.value = "";

        await store.dispatch("generateQuestion", route.params.id);
        ElMessage.success("问题生成成功");
      } catch (error) {
        console.error("Generate question error:", error);
        ElMessage.error("生成问题失败，请检查AI配置");
      }
    };

    const submitAnswer = async () => {
      if (!userAnswer.value.trim()) {
        ElMessage.warning("请输入您的回答");
        return;
      }

      try {
        const result = await store.dispatch("submitAnswer", {
          questionId: currentQuestion.value.id,
          answer: userAnswer.value.trim(),
        });

        currentAnswerResult.value = result;
        ElMessage.success("答案提交成功");
      } catch (error) {
        console.error("Submit answer error:", error);
        ElMessage.error("提交答案失败，请重试");
      }
    };

    const startNewQuestion = async () => {
      try {
        const result = await ElMessageBox.confirm(
          "开始新问题将清除当前的问答记录，是否继续？",
          "确认",
          {
            confirmButtonText: "确定",
            cancelButtonText: "取消",
            type: "warning",
          }
        );

        if (result === "confirm") {
          await generateNewQuestion();
        }
      } catch (error) {
        // User cancelled
      }
    };

    const goToDocuments = () => {
      router.push(`/knowledge-bases/${route.params.id}/documents`);
    };

    const goToSettings = () => {
      router.push("/settings");
    };

    // Lifecycle hooks
    onMounted(async () => {
      try {
        // Fetch documents for this knowledge base
        await store.dispatch("fetchDocuments", route.params.id);

        // Fetch AI config
        await store.dispatch("fetchAIConfig");
      } catch (error) {
        console.error("Failed to load initial data:", error);
      }
    });

    // Watch for route changes
    watch(
      () => route.params.id,
      async (newId) => {
        if (newId) {
          try {
            await store.dispatch("fetchDocuments", newId);
            // Clear current question when switching knowledge bases
            store.commit("SET_CURRENT_QUESTION", null);
            currentAnswerResult.value = null;
            userAnswer.value = "";
          } catch (error) {
            console.error("Failed to fetch documents:", error);
          }
        }
      }
    );

    return {
      loading,
      currentQuestion,
      currentKnowledgeBase,
      documents,
      hasDocuments,
      isAIConfigured,
      userAnswer,
      currentAnswerResult,
      formatDate,
      getScoreType,
      generateNewQuestion,
      submitAnswer,
      startNewQuestion,
      goToDocuments,
      goToSettings,
    };
  },
};
</script>

<style lang="scss" scoped>
.ai-quiz {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.header-card {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 20px;
  font-weight: 600;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.config-alert {
  margin-bottom: 20px;
}

.question-card,
.answer-card,
.result-card {
  margin-bottom: 20px;
}

.question-header,
.answer-header,
.result-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 16px;
  font-weight: 600;
}

.question-content {
  .question-text {
    font-size: 16px;
    line-height: 1.6;
    margin-bottom: 20px;
    color: #303133;
  }

  .context-snippet {
    margin-top: 20px;

    .context-label {
      font-size: 14px;
      color: #909399;
    }

    .context-text {
      display: block;
      padding: 12px;
      background-color: #f5f7fa;
      border-radius: 4px;
      font-size: 14px;
      line-height: 1.5;
    }
  }
}

.answer-content {
  .answer-input {
    margin-bottom: 20px;
  }

  .answer-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
  }
}

.result-content {
  .feedback-section,
  .suggestions-section {
    margin-bottom: 24px;

    h4 {
      margin: 0 0 12px 0;
      color: #303133;
      font-size: 16px;
      font-weight: 600;
    }

    .feedback-text {
      display: block;
      line-height: 1.6;
      color: #606266;
    }
  }

  .suggestions-list {
    margin: 0;
    padding-left: 20px;

    .suggestion-item {
      margin-bottom: 8px;
      line-height: 1.5;
      color: #606266;
    }
  }

  .answer-review {
    .user-answer-text {
      display: block;
      padding: 12px;
      background-color: #f5f7fa;
      border-radius: 4px;
      line-height: 1.6;
      color: #303133;
    }
  }
}

.loading-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(255, 255, 255, 0.8);
  z-index: 1000;
}

// Responsive design
@media (max-width: 768px) {
  .ai-quiz {
    padding: 10px;
  }

  .card-header {
    flex-direction: column;
    gap: 10px;
    align-items: flex-start;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-end;
  }

  .question-content .question-text {
    font-size: 14px;
  }

  .answer-actions {
    justify-content: center;
  }

  .result-header {
    flex-wrap: wrap;
    gap: 10px;
  }
}

// Animation
.question-card,
.answer-card,
.result-card {
  transition: all 0.3s ease;
}

.question-card:hover,
.answer-card:hover,
.result-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
</style>
