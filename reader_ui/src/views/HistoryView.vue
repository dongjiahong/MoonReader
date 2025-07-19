<template>
  <div class="history">
    <!-- Header with breadcrumb -->
    <div class="page-header">
      <el-breadcrumb separator="/">
        <el-breadcrumb-item :to="{ path: '/knowledge-bases' }">
          知识库
        </el-breadcrumb-item>
        <el-breadcrumb-item>
          {{ currentKnowledgeBase?.name || "加载中..." }}
        </el-breadcrumb-item>
        <el-breadcrumb-item>历史记录</el-breadcrumb-item>
      </el-breadcrumb>
    </div>

    <el-card>
      <template #header>
        <div class="card-header">
          <span>复习历史记录</span>
          <div class="header-actions">
            <el-button type="primary" @click="goToReview">
              <el-icon><Play /></el-icon>
              开始复习
            </el-button>
          </div>
        </div>
      </template>

      <!-- Statistics Overview -->
      <div v-if="historyStats" class="stats-overview">
        <el-row :gutter="20">
          <el-col :xs="24" :sm="6">
            <el-statistic
              title="总复习次数"
              :value="historyStats.totalSessions"
              class="stat-item"
            >
              <template #suffix>
                <el-icon><Document /></el-icon>
              </template>
            </el-statistic>
          </el-col>
          <el-col :xs="24" :sm="6">
            <el-statistic
              title="平均得分"
              :value="historyStats.averageScore"
              suffix="%"
              class="stat-item"
            >
              <template #suffix>
                <span>%</span>
                <el-icon><TrophyBase /></el-icon>
              </template>
            </el-statistic>
          </el-col>
          <el-col :xs="24" :sm="6">
            <el-statistic
              title="最高得分"
              :value="historyStats.highestScore"
              suffix="%"
              class="stat-item"
            >
              <template #suffix>
                <span>%</span>
                <el-icon><Medal /></el-icon>
              </template>
            </el-statistic>
          </el-col>
          <el-col :xs="24" :sm="6">
            <el-statistic
              title="总学习时长"
              :value="historyStats.totalDuration"
              suffix="分钟"
              class="stat-item"
            >
              <template #suffix>
                <span>分钟</span>
                <el-icon><Clock /></el-icon>
              </template>
            </el-statistic>
          </el-col>
        </el-row>
      </div>

      <!-- Filter and Search -->
      <div class="filter-section">
        <el-row :gutter="16">
          <el-col :xs="24" :sm="8">
            <el-date-picker
              v-model="dateRange"
              type="daterange"
              range-separator="至"
              start-placeholder="开始日期"
              end-placeholder="结束日期"
              format="YYYY-MM-DD"
              value-format="YYYY-MM-DD"
              @change="handleDateRangeChange"
              class="date-picker"
            />
          </el-col>
          <el-col :xs="24" :sm="8">
            <el-select
              v-model="scoreFilter"
              placeholder="按得分筛选"
              clearable
              @change="handleScoreFilterChange"
              class="score-filter"
            >
              <el-option label="优秀 (90-100分)" value="excellent" />
              <el-option label="良好 (70-89分)" value="good" />
              <el-option label="需改进 (0-69分)" value="poor" />
            </el-select>
          </el-col>
          <el-col :xs="24" :sm="8">
            <el-button
              type="default"
              @click="resetFilters"
              class="reset-button"
            >
              <el-icon><RefreshLeft /></el-icon>
              重置筛选
            </el-button>
          </el-col>
        </el-row>
      </div>

      <!-- Loading state -->
      <div v-if="isLoading" class="loading-container">
        <el-skeleton :rows="5" animated />
      </div>

      <!-- Empty state -->
      <div v-else-if="filteredHistory.length === 0" class="empty-state">
        <el-empty description="暂无复习记录">
          <el-button type="primary" @click="goToReview">
            开始第一次复习
          </el-button>
        </el-empty>
      </div>

      <!-- History List -->
      <div v-else class="history-list">
        <el-timeline>
          <el-timeline-item
            v-for="session in paginatedHistory"
            :key="session.id"
            :timestamp="formatDate(session.created_at)"
            placement="top"
            :type="getTimelineType(session.average_score)"
            :icon="getTimelineIcon(session.average_score)"
            size="large"
          >
            <el-card class="history-item" shadow="hover">
              <div class="session-header">
                <div class="session-info">
                  <h4 class="session-title">复习会话 #{{ session.id }}</h4>
                  <div class="session-meta">
                    <el-tag
                      :type="getScoreTagType(session.average_score)"
                      size="small"
                    >
                      平均得分: {{ session.average_score }}%
                    </el-tag>
                    <el-tag type="info" size="small">
                      {{ session.questions_count }} 题
                    </el-tag>
                    <el-tag type="info" size="small">
                      用时: {{ session.duration }} 分钟
                    </el-tag>
                  </div>
                </div>
                <div class="session-actions">
                  <el-button
                    type="text"
                    size="small"
                    @click="viewSessionDetails(session)"
                  >
                    <el-icon><View /></el-icon>
                    查看详情
                  </el-button>
                </div>
              </div>

              <!-- Session Summary -->
              <div class="session-summary">
                <el-progress
                  :percentage="session.average_score"
                  :status="getProgressStatus(session.average_score)"
                  :stroke-width="8"
                  class="score-progress"
                />

                <div class="summary-stats">
                  <div class="stat">
                    <span class="stat-label">正确率</span>
                    <span class="stat-value"
                      >{{ calculateAccuracy(session) }}%</span
                    >
                  </div>
                  <div class="stat">
                    <span class="stat-label">效率</span>
                    <span class="stat-value">{{
                      calculateEfficiency(session)
                    }}</span>
                  </div>
                </div>
              </div>

              <!-- Performance Insights -->
              <div v-if="session.insights" class="performance-insights">
                <el-divider content-position="left">
                  <span class="insights-label">学习洞察</span>
                </el-divider>
                <ul class="insights-list">
                  <li
                    v-for="(insight, index) in session.insights"
                    :key="index"
                    class="insight-item"
                  >
                    {{ insight }}
                  </li>
                </ul>
              </div>
            </el-card>
          </el-timeline-item>
        </el-timeline>

        <!-- Pagination -->
        <div v-if="totalPages > 1" class="pagination-container">
          <el-pagination
            v-model:current-page="currentPage"
            :page-size="pageSize"
            :total="filteredHistory.length"
            layout="prev, pager, next, jumper"
            @current-change="handlePageChange"
          />
        </div>
      </div>
    </el-card>

    <!-- Session Details Dialog -->
    <el-dialog
      v-model="showDetailsDialog"
      title="复习会话详情"
      width="800px"
      class="details-dialog"
    >
      <div v-if="selectedSession" class="session-details">
        <div class="details-header">
          <h3>复习会话 #{{ selectedSession.id }}</h3>
          <el-tag
            :type="getScoreTagType(selectedSession.average_score)"
            size="large"
          >
            平均得分: {{ selectedSession.average_score }}%
          </el-tag>
        </div>

        <el-descriptions :column="2" border>
          <el-descriptions-item label="开始时间">
            {{ formatDateTime(selectedSession.created_at) }}
          </el-descriptions-item>
          <el-descriptions-item label="用时">
            {{ selectedSession.duration }} 分钟
          </el-descriptions-item>
          <el-descriptions-item label="题目数量">
            {{ selectedSession.questions_count }} 题
          </el-descriptions-item>
          <el-descriptions-item label="平均得分">
            {{ selectedSession.average_score }}%
          </el-descriptions-item>
        </el-descriptions>

        <!-- Question Details -->
        <div v-if="selectedSession.questions" class="questions-details">
          <h4>题目详情</h4>
          <el-collapse v-model="activeQuestions">
            <el-collapse-item
              v-for="(question, index) in selectedSession.questions"
              :key="question.id"
              :title="`第 ${index + 1} 题 - 得分: ${question.score}%`"
              :name="question.id"
            >
              <div class="question-detail">
                <div class="question-text">
                  <strong>问题:</strong> {{ question.question_text }}
                </div>
                <div class="user-answer">
                  <strong>您的回答:</strong> {{ question.user_answer }}
                </div>
                <div class="ai-feedback">
                  <strong>AI评价:</strong> {{ question.ai_feedback }}
                </div>
              </div>
            </el-collapse-item>
          </el-collapse>
        </div>
      </div>

      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showDetailsDialog = false">关闭</el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { computed, ref, onMounted, watch } from "vue";
import { useStore } from "vuex";
import { useRoute, useRouter } from "vue-router";
import { ElMessage } from "element-plus";
import {
  Play,
  Document,
  TrophyBase,
  Medal,
  Clock,
  RefreshLeft,
  View,
  SuccessFilled,
  WarningFilled,
  CircleCloseFilled,
} from "@element-plus/icons-vue";

export default {
  name: "HistoryView",
  components: {
    Play,
    Document,
    TrophyBase,
    Medal,
    Clock,
    RefreshLeft,
    View,
  },
  setup() {
    const store = useStore();
    const route = useRoute();
    const router = useRouter();

    // Reactive data
    const dateRange = ref([]);
    const scoreFilter = ref("");
    const currentPage = ref(1);
    const pageSize = ref(10);
    const showDetailsDialog = ref(false);
    const selectedSession = ref(null);
    const activeQuestions = ref([]);

    // Computed properties
    const isLoading = computed(() => store.getters.isModuleLoading("review"));
    const reviewHistory = computed(() => store.state.reviewHistory || []);
    const currentKnowledgeBase = computed(
      () => store.state.currentKnowledgeBase
    );

    const historyStats = computed(() => {
      if (reviewHistory.value.length === 0) return null;

      const sessions = reviewHistory.value;
      const totalSessions = sessions.length;
      const totalScore = sessions.reduce(
        (sum, session) => sum + session.average_score,
        0
      );
      const totalDuration = sessions.reduce(
        (sum, session) => sum + session.duration,
        0
      );
      const highestScore = Math.max(
        ...sessions.map((session) => session.average_score)
      );

      return {
        totalSessions,
        averageScore: Math.round(totalScore / totalSessions),
        highestScore,
        totalDuration,
      };
    });

    const filteredHistory = computed(() => {
      let filtered = [...reviewHistory.value];

      // Date range filter
      if (dateRange.value && dateRange.value.length === 2) {
        const [startDate, endDate] = dateRange.value;
        filtered = filtered.filter((session) => {
          const sessionDate = new Date(session.created_at)
            .toISOString()
            .split("T")[0];
          return sessionDate >= startDate && sessionDate <= endDate;
        });
      }

      // Score filter
      if (scoreFilter.value) {
        filtered = filtered.filter((session) => {
          const score = session.average_score;
          switch (scoreFilter.value) {
            case "excellent":
              return score >= 90;
            case "good":
              return score >= 70 && score < 90;
            case "poor":
              return score < 70;
            default:
              return true;
          }
        });
      }

      // Sort by date (newest first)
      return filtered.sort(
        (a, b) => new Date(b.created_at) - new Date(a.created_at)
      );
    });

    const paginatedHistory = computed(() => {
      const start = (currentPage.value - 1) * pageSize.value;
      const end = start + pageSize.value;
      return filteredHistory.value.slice(start, end);
    });

    const totalPages = computed(() => {
      return Math.ceil(filteredHistory.value.length / pageSize.value);
    });

    // Methods
    const formatDate = (dateString) => {
      return new Date(dateString).toLocaleDateString("zh-CN");
    };

    const formatDateTime = (dateString) => {
      return new Date(dateString).toLocaleString("zh-CN");
    };

    const getScoreTagType = (score) => {
      if (score >= 90) return "success";
      if (score >= 70) return "warning";
      return "danger";
    };

    const getTimelineType = (score) => {
      if (score >= 90) return "success";
      if (score >= 70) return "warning";
      return "danger";
    };

    const getTimelineIcon = (score) => {
      if (score >= 90) return SuccessFilled;
      if (score >= 70) return WarningFilled;
      return CircleCloseFilled;
    };

    const getProgressStatus = (score) => {
      if (score >= 90) return "success";
      if (score >= 70) return "warning";
      return "exception";
    };

    const calculateAccuracy = (session) => {
      // Mock calculation - in real app, this would be based on correct/incorrect answers
      return Math.round(session.average_score * 0.9);
    };

    const calculateEfficiency = (session) => {
      // Mock calculation - in real app, this would be based on time per question
      const timePerQuestion = session.duration / session.questions_count;
      if (timePerQuestion < 2) return "高效";
      if (timePerQuestion < 4) return "正常";
      return "需提升";
    };

    const handleDateRangeChange = () => {
      currentPage.value = 1;
    };

    const handleScoreFilterChange = () => {
      currentPage.value = 1;
    };

    const resetFilters = () => {
      dateRange.value = [];
      scoreFilter.value = "";
      currentPage.value = 1;
    };

    const handlePageChange = (page) => {
      currentPage.value = page;
    };

    const viewSessionDetails = (session) => {
      selectedSession.value = session;
      showDetailsDialog.value = true;
      activeQuestions.value = [];
    };

    const goToReview = () => {
      router.push(`/knowledge-bases/${route.params.id}/review`);
    };

    const loadData = async () => {
      try {
        // Load knowledge bases if not already loaded
        if (store.state.knowledgeBases.length === 0) {
          await store.dispatch("fetchKnowledgeBases");
        }

        // Load review history for this knowledge base
        await store.dispatch("fetchReviewHistory", route.params.id);
      } catch (error) {
        console.error("Load data error:", error);
        ElMessage.error("加载历史记录失败");
      }
    };

    // Lifecycle hooks
    onMounted(() => {
      loadData();
    });

    // Watch for route changes
    watch(
      () => route.params.id,
      (newId) => {
        if (newId) {
          loadData();
        }
      }
    );

    return {
      isLoading,
      reviewHistory,
      currentKnowledgeBase,
      historyStats,
      filteredHistory,
      paginatedHistory,
      totalPages,
      dateRange,
      scoreFilter,
      currentPage,
      pageSize,
      showDetailsDialog,
      selectedSession,
      activeQuestions,
      formatDate,
      formatDateTime,
      getScoreTagType,
      getTimelineType,
      getTimelineIcon,
      getProgressStatus,
      calculateAccuracy,
      calculateEfficiency,
      handleDateRangeChange,
      handleScoreFilterChange,
      resetFilters,
      handlePageChange,
      viewSessionDetails,
      goToReview,
    };
  },
};
</script>

<style lang="scss" scoped>
.history {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.page-header {
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

.stats-overview {
  margin-bottom: 24px;
  padding: 20px;
  background-color: #f8f9fa;
  border-radius: 8px;

  .stat-item {
    text-align: center;

    :deep(.el-statistic__content) {
      display: flex;
      align-items: center;
      justify-content: center;
      gap: 8px;
    }
  }
}

.filter-section {
  margin-bottom: 24px;
  padding: 16px;
  background-color: #fff;
  border: 1px solid #e4e7ed;
  border-radius: 8px;

  .date-picker,
  .score-filter {
    width: 100%;
  }

  .reset-button {
    width: 100%;
  }
}

.loading-container {
  padding: 20px;
  text-align: center;
}

.empty-state {
  padding: 60px 20px;
  text-align: center;
}

.history-list {
  .history-item {
    margin-bottom: 16px;
    transition: all 0.3s ease;

    &:hover {
      box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    }
  }

  .session-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 16px;

    .session-info {
      flex: 1;

      .session-title {
        margin: 0 0 8px 0;
        color: #303133;
        font-size: 16px;
        font-weight: 600;
      }

      .session-meta {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
    }

    .session-actions {
      margin-left: 16px;
    }
  }

  .session-summary {
    margin-bottom: 16px;

    .score-progress {
      margin-bottom: 12px;
    }

    .summary-stats {
      display: flex;
      gap: 24px;

      .stat {
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 4px;

        .stat-label {
          font-size: 12px;
          color: #909399;
        }

        .stat-value {
          font-size: 14px;
          font-weight: 600;
          color: #303133;
        }
      }
    }
  }

  .performance-insights {
    .insights-label {
      font-size: 14px;
      color: #909399;
    }

    .insights-list {
      margin: 0;
      padding-left: 20px;

      .insight-item {
        margin-bottom: 8px;
        line-height: 1.5;
        color: #606266;
        font-size: 14px;
      }
    }
  }
}

.pagination-container {
  display: flex;
  justify-content: center;
  margin-top: 24px;
}

.details-dialog {
  .session-details {
    .details-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 20px;

      h3 {
        margin: 0;
        color: #303133;
      }
    }

    .questions-details {
      margin-top: 24px;

      h4 {
        margin: 0 0 16px 0;
        color: #303133;
        font-size: 16px;
        font-weight: 600;
      }

      .question-detail {
        padding: 16px;
        background-color: #f8f9fa;
        border-radius: 8px;

        > div {
          margin-bottom: 12px;
          line-height: 1.6;

          &:last-child {
            margin-bottom: 0;
          }

          strong {
            color: #303133;
            margin-right: 8px;
          }
        }
      }
    }
  }
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
}

// Responsive design
@media (max-width: 768px) {
  .history {
    padding: 10px;
  }

  .card-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;

    span {
      text-align: center;
    }
  }

  .stats-overview {
    padding: 16px;

    .el-col {
      margin-bottom: 16px;

      &:last-child {
        margin-bottom: 0;
      }
    }
  }

  .filter-section {
    .el-col {
      margin-bottom: 12px;

      &:last-child {
        margin-bottom: 0;
      }
    }
  }

  .session-header {
    flex-direction: column;
    gap: 12px;

    .session-actions {
      margin-left: 0;
      align-self: flex-end;
    }
  }

  .summary-stats {
    justify-content: space-around;
  }

  .details-dialog {
    :deep(.el-dialog) {
      width: 95% !important;
      margin: 0 auto;
    }
  }
}

// Timeline customization
:deep(.el-timeline) {
  .el-timeline-item__timestamp {
    font-weight: 500;
    color: #606266;
  }
}
</style>
