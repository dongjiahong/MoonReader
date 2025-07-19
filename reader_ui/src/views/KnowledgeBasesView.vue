<template>
  <div class="knowledge-bases">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>知识库管理</span>
          <el-button type="primary" @click="showCreateDialog = true">
            <el-icon><Plus /></el-icon>
            创建知识库
          </el-button>
        </div>
      </template>

      <!-- Loading state -->
      <div v-if="isLoading" class="loading-container">
        <el-skeleton :rows="3" animated />
      </div>

      <!-- Empty state -->
      <div v-else-if="knowledgeBases.length === 0" class="empty-state">
        <el-empty description="暂无知识库">
          <el-button type="primary" @click="showCreateDialog = true">
            创建第一个知识库
          </el-button>
        </el-empty>
      </div>

      <!-- Knowledge bases list -->
      <div v-else class="knowledge-bases-list">
        <el-row :gutter="20">
          <el-col
            v-for="kb in knowledgeBases"
            :key="kb.id"
            :xs="24"
            :sm="12"
            :md="8"
            :lg="6"
            class="kb-col"
          >
            <el-card class="kb-card" shadow="hover">
              <div class="kb-header">
                <h3 class="kb-title">{{ kb.name }}</h3>
                <el-dropdown @command="handleKbAction">
                  <el-button type="text" size="small">
                    <el-icon><MoreFilled /></el-icon>
                  </el-button>
                  <template #dropdown>
                    <el-dropdown-menu>
                      <el-dropdown-item :command="{ action: 'edit', kb }">
                        <el-icon><Edit /></el-icon>
                        编辑
                      </el-dropdown-item>
                      <el-dropdown-item
                        :command="{ action: 'delete', kb }"
                        divided
                      >
                        <el-icon><Delete /></el-icon>
                        删除
                      </el-dropdown-item>
                    </el-dropdown-menu>
                  </template>
                </el-dropdown>
              </div>

              <p class="kb-description">
                {{ kb.description || "暂无描述" }}
              </p>

              <div class="kb-stats">
                <el-tag size="small" type="info">
                  {{ kb.document_count || 0 }} 个文档
                </el-tag>
                <span class="kb-date">
                  {{ formatDate(kb.created_at) }}
                </span>
              </div>

              <div class="kb-actions">
                <el-button
                  type="primary"
                  size="small"
                  @click="$router.push(`/knowledge-bases/${kb.id}`)"
                >
                  进入知识库
                </el-button>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </div>
    </el-card>

    <!-- Create/Edit Knowledge Base Dialog -->
    <el-dialog
      v-model="showCreateDialog"
      :title="editingKb ? '编辑知识库' : '创建知识库'"
      width="500px"
      @close="resetForm"
    >
      <el-form
        ref="kbFormRef"
        :model="kbForm"
        :rules="kbFormRules"
        label-width="80px"
      >
        <el-form-item label="名称" prop="name">
          <el-input
            v-model="kbForm.name"
            placeholder="请输入知识库名称"
            maxlength="50"
            show-word-limit
          />
        </el-form-item>
        <el-form-item label="描述" prop="description">
          <el-input
            v-model="kbForm.description"
            type="textarea"
            :rows="3"
            placeholder="请输入知识库描述（可选）"
            maxlength="200"
            show-word-limit
          />
        </el-form-item>
      </el-form>

      <template #footer>
        <span class="dialog-footer">
          <el-button @click="showCreateDialog = false">取消</el-button>
          <el-button type="primary" :loading="isLoading" @click="handleSubmit">
            {{ editingKb ? "更新" : "创建" }}
          </el-button>
        </span>
      </template>
    </el-dialog>
  </div>
</template>

<script>
import { mapState, mapGetters, mapActions } from "vuex";
import { Plus, Edit, Delete, MoreFilled } from "@element-plus/icons-vue";
import { ElMessage, ElNotification } from "element-plus";
import { showDeleteConfirmDialog } from "@/utils/confirmDialog";

export default {
  name: "KnowledgeBasesView",
  components: {
    Plus,
    Edit,
    Delete,
    MoreFilled,
  },
  data() {
    return {
      showCreateDialog: false,
      showDeleteDialog: false,
      editingKb: null,
      deletingKb: null,
      kbForm: {
        name: "",
        description: "",
      },
      kbFormRules: {
        name: [
          { required: true, message: "请输入知识库名称", trigger: "blur" },
          {
            min: 1,
            max: 50,
            message: "名称长度在 1 到 50 个字符",
            trigger: "blur",
          },
        ],
        description: [
          { max: 200, message: "描述长度不能超过 200 个字符", trigger: "blur" },
        ],
      },
    };
  },
  computed: {
    ...mapState(["knowledgeBases", "successMessage"]),
    ...mapGetters(["isModuleLoading", "getModuleError"]),

    isLoading() {
      return this.isModuleLoading("knowledgeBases");
    },

    error() {
      return this.getModuleError("knowledgeBases");
    },
  },
  async created() {
    await this.loadKnowledgeBases();
  },
  methods: {
    ...mapActions([
      "fetchKnowledgeBases",
      "createKnowledgeBase",
      "updateKnowledgeBase",
      "deleteKnowledgeBase",
    ]),

    async loadKnowledgeBases() {
      try {
        await this.fetchKnowledgeBases();
      } catch (error) {
        ElMessage.error("加载知识库列表失败");
      }
    },

    handleKbAction({ action, kb }) {
      if (action === "edit") {
        this.editKnowledgeBase(kb);
      } else if (action === "delete") {
        this.showDeleteConfirmation(kb);
      }
    },

    editKnowledgeBase(kb) {
      this.editingKb = kb;
      this.kbForm = {
        name: kb.name,
        description: kb.description || "",
      };
      this.showCreateDialog = true;
    },

    async showDeleteConfirmation(kb) {
      const confirmed = await showDeleteConfirmDialog(kb.name, "知识库", {
        warningText: "此操作将删除知识库及其所有文档，且无法恢复！",
      });

      if (confirmed) {
        await this.handleDelete(kb);
      }
    },

    async handleSubmit() {
      try {
        await this.$refs.kbFormRef.validate();

        if (this.editingKb) {
          await this.updateKnowledgeBase({
            id: this.editingKb.id,
            data: this.kbForm,
          });
          ElNotification.success({
            title: "更新成功",
            message: `知识库 "${this.kbForm.name}" 已更新`,
            duration: 3000,
          });
        } else {
          await this.createKnowledgeBase(this.kbForm);
          ElNotification.success({
            title: "创建成功",
            message: `知识库 "${this.kbForm.name}" 已创建`,
            duration: 3000,
          });
        }

        this.showCreateDialog = false;
        this.resetForm();
      } catch (error) {
        // Error is handled by the store and displayed via global error handling
        console.error("Knowledge base operation failed:", error);
      }
    },

    async handleDelete(kb) {
      try {
        await this.deleteKnowledgeBase(kb.id);
        ElNotification.success({
          title: "删除成功",
          message: `知识库 "${kb.name}" 已删除`,
          duration: 3000,
        });
      } catch (error) {
        console.error("Delete knowledge base failed:", error);
      }
    },

    resetForm() {
      this.editingKb = null;
      this.kbForm = {
        name: "",
        description: "",
      };
      if (this.$refs.kbFormRef) {
        this.$refs.kbFormRef.resetFields();
      }
    },

    formatDate(dateString) {
      if (!dateString) return "";
      const date = new Date(dateString);
      return date.toLocaleDateString("zh-CN");
    },
  },
  watch: {
    error(newError) {
      if (newError) {
        ElMessage.error(newError);
      }
    },
  },
};
</script>

<style lang="scss" scoped>
.knowledge-bases {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 20px;
  font-weight: 600;
}

.loading-container {
  padding: 20px;
}

.empty-state {
  padding: 40px;
  text-align: center;
}

.knowledge-bases-list {
  padding: 20px 0;
}

.kb-col {
  margin-bottom: 20px;
}

.kb-card {
  height: 200px;
  display: flex;
  flex-direction: column;

  :deep(.el-card__body) {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 16px;
  }
}

.kb-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.kb-title {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
  color: #303133;
  line-height: 1.4;
  flex: 1;
  margin-right: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.kb-description {
  color: #606266;
  font-size: 14px;
  line-height: 1.5;
  margin: 0 0 12px 0;
  flex: 1;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.kb-stats {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
  font-size: 12px;
}

.kb-date {
  color: #909399;
}

.kb-actions {
  margin-top: auto;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.warning-text {
  color: #e6a23c;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  margin-top: 12px;
}

// Responsive design
@media (max-width: 768px) {
  .knowledge-bases {
    padding: 10px;
  }

  .kb-card {
    height: auto;
    min-height: 180px;
  }

  .card-header {
    flex-direction: column;
    gap: 12px;
    align-items: stretch;

    span {
      text-align: center;
    }
  }
}
</style>
