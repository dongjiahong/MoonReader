<template>
  <div class="review">
    <!-- Header Card -->
    <el-card class="header-card">
      <template #header>
        <div class="card-header">
          <span>复习模块</span>
          <div class="header-actions">
            <el-button
              type="primary"
              :loading="isLoading"
              :disabled="!hasDocuments"
              @click="startReviewSession"
            >
              <el-icon><VideoPlay /></el-icon>
              开始复习
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
          <p>请先配置AI服务才能使用复习功能</p>
          <el-button link @click="goToSettings"> 前往设置 </el-button>
        </template>
      </el-alert>

      <!-- Review Stats -->
      <div v-if="hasDocuments && isAIConfigured" class="review-stats">
        <el-row :gutter="20">
          <el-col :xs="24" :sm="8">
            <el-statistic
              title="总复习次数"
              :value="reviewStats.totalSessions"
            />
          </el-col>
          <el-col :xs="24" :sm="8">
            <el-statistic
              title="平均得分"
              :value="reviewStats.averageScore"
              suffix="%"
            />
          </el-col>
          <el-col :xs="24" :sm="8">
            <el-statistic
              title="最近复习"
              :value="reviewStats.lastReviewDate"
            />
          </el-col>
        </el-row>
      </div>
    </el-card>

    <!-- Current Question Card -->
    <el-card
      v-if="currentQuestion && hasDocuments && isAIConfigured"
      class="question-card"
    >
      <template #header>
        <div class="question-header">
          <el-icon><QuestionFilled /></el-icon>
          <span>复习问题</span>
          <el-tag type="info" size="small">
            第 {{ currentQuestionIndex + 1 }} / {{ totalQuestions }} 题
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

        <!-- Answer Input -->
        <div class="answer-section">
          <el-input
            v-model="userAnswer"
            type="textarea"
            :rows="4"
            placeholder="请输入您的回答..."
            :disabled="isLoading || showAnswer"
            class="answer-input"
          />

          <div class="answer-actions">
            <el-button
              v-if="!showAnswer"
              type="primary"
              :loading="isLoading"
              :disabled="!userAnswer.trim()"
              @click="submitAnswer"
            >
              <el-icon><Check /></el-icon>
              提交答案
            </el-button>

            <el-button
              v-if="showAnswer"
              type="success"
              @click="nextQuestion"
              :disabled="isLoading"
            >
              <el-icon><ArrowRight /></el-icon>
              下一题
            </el-button>

            <el-button
              v-if="showAnswer && currentQuestionIndex === totalQuestions - 1"
              type="warning"
              @click="finishReview"
              :disabled="isLoading"
            >
              <el-icon><Flag /></el-icon>
              完成复习
            </el-button>
          </div>
        </div>
      </div>
    </el-card>

    <!-- Answer Result Card -->
    <el-card v-if="showAnswer && currentAnswerResult" class="result-card">
      <template #header>
        <div class="result-header">
          <el-icon><TrophyBase /></el-icon>
          <span>答案评估</span>
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

    <!-- Review Summary Card -->
    <el-card v-if="reviewCompleted" class="summary-card">
      <template #header>
        <div class="summary-header">
          <el-icon><Medal /></el-icon>
          <span>复习总结</span>
        </div>
      </template>

      <div class="summary-content">
        <div class="summary-stats">
          <el-row :gutter="20">
            <el-col :xs="24" :sm="8">
              <el-statistic
                title="总题数"
                :value="reviewSummary.totalQuestions"
              />
            </el-col>
            <el-col :xs="24" :sm="8">
              <el-statistic
                title="平均得分"
                :value="reviewSummary.averageScore"
                suffix="%"
              />
            </el-col>
            <el-col :xs="24" :sm="8">
              <el-statistic
                title="用时"
                :value="reviewSummary.duration"
                suffix="分钟"
              />
            </el-col>
          </el-row>
        </div>

        <div class="summary-actions">
          <el-button type="primary" @click="startNewReview">
            <el-icon><Refresh /></el-icon>
            再次复习
          </el-button>
          <el-button @click="viewHistory">
            <el-icon><Clock /></el-icon>
            查看历史
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script>
import { computed, ref, onMounted, watch } from "vue";
import { useStore } from "vuex";
import { useRoute, useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import {
  Document,
  QuestionFilled,
  Check,
  ArrowRight,
  Flag,
  TrophyBase,
  Medal,
  Refresh,
  Clock,
  VideoPlay,
} from "@element-plus/icons-vue";

export default {
  name: "ReviewView",
  components: {
    Document,
    QuestionFilled,
    Check,
    ArrowRight,
    Flag,
    TrophyBase,
    Medal,
    Refresh,
    Clock,
    VideoPlay,
  },
  setup() {
    const store = useStore();
    const route = useRoute();
    const router = useRouter();

    // Reactive data
    const userAnswer = ref("");
    const showAnswer = ref(false);
    const currentAnswerResult = ref(null);
    const currentQuestionIndex = ref(0);
    const totalQuestions = ref(5); // Default review session length
    const reviewCompleted = ref(false);
    const reviewSummary = ref({});
    const reviewStats = ref({
      totalSessions: 0,
      averageScore: 0,
      lastReviewDate: "暂无记录",
    });

    // Computed properties
    const isLoading = computed(() => store.getters.isModuleLoading("review"));
    const currentQuestion = computed(() => store.state.currentQuestion);
    const currentKnowledgeBase = computed(
      () => store.state.currentKnowledgeBase
    );
    const documents = computed(() => store.state.documents || []);
    const aiConfig = computed(() => store.state.aiConfig);

    const hasDocuments = computed(() => {
      console.log("Review - Documents:", documents.value);
      return documents.value.length > 0;
    });
    const isAIConfigured = computed(() => {
      const config = aiConfig.value;
      console.log("Review - AI Config:", config);
      if (config.provider === "local") {
        return !!config.api_url;
      }
      const configured = !!(config.api_key_configured && config.provider);
      console.log("Review - Is configured:", configured);
      return configured;
    });

    // Methods
    const getScoreType = (score) => {
      if (score >= 90) return "success";
      if (score >= 70) return "warning";
      return "danger";
    };

    const startReviewSession = async () => {
      try {
        reviewCompleted.value = false;
        currentQuestionIndex.value = 0;
        await generateNewQuestion();
      } catch (error) {
        console.error("Start review session error:", error);
        ElMessage.error("开始复习失败，请重试");
      }
    };

    const generateNewQuestion = async () => {
      try {
        showAnswer.value = false;
        currentAnswerResult.value = null;
        userAnswer.value = "";

        await store.dispatch("getRandomReviewQuestion", route.params.id);
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
        showAnswer.value = true;
        ElMessage.success("答案提交成功");
      } catch (error) {
        console.error("Submit answer error:", error);
        ElMessage.error("提交答案失败，请重试");
      }
    };

    const nextQuestion = async () => {
      currentQuestionIndex.value++;

      if (currentQuestionIndex.value < totalQuestions.value) {
        await generateNewQuestion();
      } else {
        await finishReview();
      }
    };

    const finishReview = async () => {
      try {
        // Calculate review summary
        reviewSummary.value = {
          totalQuestions: totalQuestions.value,
          averageScore: 85, // This should be calculated from actual scores
          duration: Math.floor(Math.random() * 30) + 10, // Mock duration
        };

        // Save review session
        await store.dispatch("saveReviewSession", {
          knowledge_base_id: route.params.id,
          questions_count: totalQuestions.value,
          average_score: reviewSummary.value.averageScore,
          duration: reviewSummary.value.duration,
        });

        reviewCompleted.value = true;
        ElMessage.success("复习完成！");
      } catch (error) {
        console.error("Finish review error:", error);
        ElMessage.error("保存复习记录失败");
      }
    };

    const startNewReview = () => {
      reviewCompleted.value = false;
      startReviewSession();
    };

    const viewHistory = () => {
      router.push(`/knowledge-bases/${route.params.id}/history`);
    };

    const goToDocuments = () => {
      router.push(`/knowledge-bases/${route.params.id}/documents`);
    };

    const goToSettings = () => {
      router.push("/settings");
    };

    const loadReviewStats = async () => {
      try {
        const history = await store.dispatch(
          "fetchReviewHistory",
          route.params.id
        );
        if (history && history.length > 0) {
          reviewStats.value = {
            totalSessions: history.length,
            averageScore: Math.round(
              history.reduce((sum, session) => sum + session.average_score, 0) /
                history.length
            ),
            lastReviewDate: new Date(history[0].created_at).toLocaleDateString(
              "zh-CN"
            ),
          };
        }
      } catch (error) {
        console.error("Load review stats error:", error);
      }
    };

    // Lifecycle hooks
    onMounted(async () => {
      try {
        // Fetch documents for this knowledge base
        await store.dispatch("fetchDocuments", route.params.id);

        // Fetch AI config
        await store.dispatch("fetchAIConfig");

        // Load review statistics
        await loadReviewStats();
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
            await loadReviewStats();
            // Clear current question when switching knowledge bases
            store.commit("SET_CURRENT_QUESTION", null);
            reviewCompleted.value = false;
            showAnswer.value = false;
            currentAnswerResult.value = null;
            userAnswer.value = "";
          } catch (error) {
            console.error("Failed to fetch documents:", error);
          }
        }
      }
    );

    return {
      isLoading,
      currentQuestion,
      currentKnowledgeBase,
      documents,
      hasDocuments,
      isAIConfigured,
      userAnswer,
      showAnswer,
      currentAnswerResult,
      currentQuestionIndex,
      totalQuestions,
      reviewCompleted,
      reviewSummary,
      reviewStats,
      getScoreType,
      startReviewSession,
      submitAnswer,
      nextQuestion,
      finishReview,
      startNewReview,
      viewHistory,
      goToDocuments,
      goToSettings,
    };
  },
};
</script>

<style lang="scss" scoped>
.review {
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

.review-stats {
  padding: 20px 0;
}

.question-card,
.result-card,
.summary-card {
  margin-bottom: 20px;
}

.question-header,
.result-header,
.summary-header {
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
    margin-bottom: 20px;

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

  .answer-section {
    .answer-input {
      margin-bottom: 20px;
    }

    .answer-actions {
      display: flex;
      justify-content: flex-end;
      gap: 10px;
    }
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

.summary-content {
  .summary-stats {
    margin-bottom: 24px;
  }

  .summary-actions {
    display: flex;
    justify-content: center;
    gap: 16px;
  }
}

// Responsive design
@media (max-width: 768px) {
  .review {
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
    flex-wrap: wrap;
  }

  .summary-actions {
    flex-direction: column;

    .el-button {
      width: 100%;
    }
  }
}

// Animation
.question-card,
.result-card,
.summary-card {
  transition: all 0.3s ease;
}

.question-card:hover,
.result-card:hover,
.summary-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
</style>
