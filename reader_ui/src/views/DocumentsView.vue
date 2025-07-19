<template>
  <div class="documents">
    <!-- Header with breadcrumb -->
    <div class="page-header">
      <el-breadcrumb separator="/">
        <el-breadcrumb-item :to="{ path: '/knowledge-bases' }">
          知识库
        </el-breadcrumb-item>
        <el-breadcrumb-item>
          {{ currentKnowledgeBase?.name || "加载中..." }}
        </el-breadcrumb-item>
        <el-breadcrumb-item>文档管理</el-breadcrumb-item>
      </el-breadcrumb>
    </div>

    <el-card>
      <template #header>
        <div class="card-header">
          <span>文档管理</span>
          <div class="header-actions">
            <el-upload
              ref="uploadRef"
              :action="uploadUrl"
              :headers="uploadHeaders"
              :before-upload="beforeUpload"
              :on-success="handleUploadSuccess"
              :on-error="handleUploadError"
              :on-progress="handleUploadProgress"
              :show-file-list="false"
              :accept="acceptedFileTypes"
              multiple
            >
              <el-button type="primary">
                <el-icon><Upload /></el-icon>
                上传文档
              </el-button>
            </el-upload>
          </div>
        </div>
      </template>

      <!-- Upload Progress -->
      <div v-if="uploadingFiles.length > 0" class="upload-progress">
        <h4>上传进度</h4>
        <div v-for="file in uploadingFiles" :key="file.uid" class="upload-item">
          <div class="upload-info">
            <span class="filename">{{ file.name }}</span>
            <span class="file-size">{{ formatFileSize(file.size) }}</span>
          </div>
          <el-progress
            :percentage="file.progress"
            :status="getProgressStatus(file)"
            :stroke-width="6"
          />
        </div>
      </div>

      <!-- Loading state -->
      <div
        v-if="isLoading && filteredDocuments.length === 0"
        class="loading-container"
      >
        <el-skeleton :rows="3" animated />
      </div>

      <!-- Empty state -->
      <div v-else-if="filteredDocuments.length === 0" class="empty-state">
        <el-empty description="暂无文档">
          <el-upload
            :action="uploadUrl"
            :headers="uploadHeaders"
            :before-upload="beforeUpload"
            :on-success="handleUploadSuccess"
            :on-error="handleUploadError"
            :show-file-list="false"
            :accept="acceptedFileTypes"
            multiple
          >
            <el-button type="primary">上传第一个文档</el-button>
          </el-upload>
        </el-empty>
      </div>

      <!-- Documents list -->
      <div v-else class="documents-list">
        <el-table :data="filteredDocuments" stripe>
          <el-table-column prop="filename" label="文件名" min-width="200">
            <template #default="{ row }">
              <div class="file-info">
                <el-icon class="file-icon">
                  <Document v-if="row.file_type === 'pdf'" />
                  <Reading v-else-if="row.file_type === 'epub'" />
                  <Memo v-else />
                </el-icon>
                <span class="filename">{{ row.filename }}</span>
              </div>
            </template>
          </el-table-column>

          <el-table-column prop="file_type" label="类型" width="80">
            <template #default="{ row }">
              <el-tag :type="getFileTypeTagType(row.file_type)" size="small">
                {{ row.file_type.toUpperCase() }}
              </el-tag>
            </template>
          </el-table-column>

          <el-table-column prop="file_size" label="大小" width="100">
            <template #default="{ row }">
              {{ formatFileSize(row.file_size) }}
            </template>
          </el-table-column>

          <el-table-column prop="upload_date" label="上传时间" width="150">
            <template #default="{ row }">
              {{ formatDate(row.upload_date) }}
            </template>
          </el-table-column>

          <el-table-column
            prop="content_preview"
            label="内容预览"
            min-width="200"
          >
            <template #default="{ row }">
              <div class="content-preview">
                {{ row.content_preview || "暂无预览" }}
              </div>
            </template>
          </el-table-column>

          <el-table-column label="操作" width="120" fixed="right">
            <template #default="{ row }">
              <el-button
                type="danger"
                size="small"
                @click="showDeleteConfirmation(row)"
              >
                <el-icon><Delete /></el-icon>
                删除
              </el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
    </el-card>
  </div>
</template>

<script>
import { mapState, mapGetters, mapActions } from "vuex";
import {
  Upload,
  Document,
  Reading,
  Memo,
  Delete,
} from "@element-plus/icons-vue";
import { ElMessage, ElNotification } from "element-plus";
import { showDeleteConfirmDialog } from "@/utils/confirmDialog";

export default {
  name: "DocumentsView",
  components: {
    Upload,
    Document,
    Reading,
    Memo,
    Delete,
  },
  data() {
    return {
      showDeleteDialog: false,
      deletingDocument: null,
      uploadingFiles: [],
      acceptedFileTypes: ".pdf,.epub,.txt",
    };
  },
  computed: {
    ...mapState(["documents", "knowledgeBases"]),
    ...mapGetters([
      "isModuleLoading",
      "getModuleError",
      "getKnowledgeBaseById",
      "getDocumentsByKnowledgeBase",
    ]),

    isLoading() {
      return this.isModuleLoading("documents");
    },

    error() {
      return this.getModuleError("documents");
    },

    knowledgeBaseId() {
      return this.$route.params.id;
    },

    currentKnowledgeBase() {
      return this.getKnowledgeBaseById(this.knowledgeBaseId);
    },

    uploadUrl() {
      return `${
        process.env.VUE_APP_API_URL || "http://localhost:3000/api"
      }/knowledge-bases/${this.knowledgeBaseId}/documents`;
    },

    uploadHeaders() {
      return {
        // Add any authentication headers if needed
      };
    },

    filteredDocuments() {
      // Since fetchDocuments loads documents for a specific knowledge base,
      // we can directly return all documents from the store
      console.log("Documents in store:", this.documents);
      console.log("Knowledge base ID:", this.knowledgeBaseId);
      return this.documents || [];
    },
  },
  async created() {
    await this.loadData();
  },
  methods: {
    ...mapActions(["fetchKnowledgeBases", "fetchDocuments", "deleteDocument"]),

    async loadData() {
      try {
        // Load knowledge bases if not already loaded
        if (this.knowledgeBases.length === 0) {
          await this.fetchKnowledgeBases();
        }

        // Load documents for this knowledge base
        console.log(
          "Loading documents for knowledge base:",
          this.knowledgeBaseId
        );
        await this.fetchDocuments(this.knowledgeBaseId);
        console.log("Documents loaded:", this.filteredDocuments);
      } catch (error) {
        console.error("Load data error:", error);
        ElMessage.error("加载数据失败");
      }
    },

    beforeUpload(file) {
      // Validate file type
      const fileExtension = file.name.split(".").pop().toLowerCase();
      const allowedExtensions = ["pdf", "epub", "txt"];

      if (!allowedExtensions.includes(fileExtension)) {
        ElMessage.error("只支持 PDF、EPUB 和 TXT 格式的文件");
        return false;
      }

      // Validate file size (50MB limit)
      const maxSize = 50 * 1024 * 1024; // 50MB
      if (file.size > maxSize) {
        ElMessage.error("文件大小不能超过 50MB");
        return false;
      }

      // Add to uploading files list
      this.uploadingFiles.push({
        uid: file.uid,
        name: file.name,
        size: file.size,
        progress: 0,
        status: "", // Empty string for normal progress
      });

      return true;
    },

    handleUploadProgress(event, file) {
      const uploadingFile = this.uploadingFiles.find((f) => f.uid === file.uid);
      if (uploadingFile) {
        uploadingFile.progress = Math.round(event.percent);
      }
    },

    async handleUploadSuccess(response, file) {
      const uploadingFile = this.uploadingFiles.find((f) => f.uid === file.uid);
      if (uploadingFile) {
        uploadingFile.status = "success";
        uploadingFile.progress = 100;
      }

      ElMessage.success(`文档 ${file.name} 上传成功`);

      // Remove from uploading list after a delay
      setTimeout(() => {
        this.uploadingFiles = this.uploadingFiles.filter(
          (f) => f.uid !== file.uid
        );
      }, 2000);

      // Refresh documents list
      await this.fetchDocuments(this.knowledgeBaseId);
    },

    handleUploadError(error, file) {
      const uploadingFile = this.uploadingFiles.find((f) => f.uid === file.uid);
      if (uploadingFile) {
        uploadingFile.status = "exception";
      }

      ElMessage.error(`文档 ${file.name} 上传失败`);

      // Remove from uploading list after a delay
      setTimeout(() => {
        this.uploadingFiles = this.uploadingFiles.filter(
          (f) => f.uid !== file.uid
        );
      }, 3000);
    },

    async showDeleteConfirmation(document) {
      const confirmed = await showDeleteConfirmDialog(
        document.filename,
        "文档",
        {
          warningText: "此操作无法恢复！",
        }
      );

      if (confirmed) {
        await this.handleDelete(document);
      }
    },

    async handleDelete(document) {
      try {
        await this.deleteDocument({
          id: document.id,
          filename: document.filename,
        });
        ElNotification.success({
          title: "删除成功",
          message: `文档 "${document.filename}" 已删除`,
          duration: 3000,
        });
      } catch (error) {
        console.error("Delete document failed:", error);
      }
    },

    getFileTypeTagType(fileType) {
      switch (fileType) {
        case "pdf":
          return "danger";
        case "epub":
          return "warning";
        case "txt":
          return "info";
        default:
          return "";
      }
    },

    formatFileSize(bytes) {
      if (!bytes) return "0 B";
      const k = 1024;
      const sizes = ["B", "KB", "MB", "GB"];
      const i = Math.floor(Math.log(bytes) / Math.log(k));
      return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
    },

    formatDate(dateString) {
      if (!dateString) return "";
      const date = new Date(dateString);
      return date.toLocaleString("zh-CN");
    },

    getProgressStatus(file) {
      // Return valid Element Plus Progress status values
      if (file.status === "success") {
        return "success";
      } else if (file.status === "exception") {
        return "exception";
      } else {
        return ""; // Default status for uploading
      }
    },
  },
  watch: {
    error(newError) {
      if (newError) {
        ElMessage.error(newError);
      }
    },

    "$route.params.id": {
      handler(newId) {
        if (newId) {
          this.loadData();
        }
      },
      immediate: true,
    },
  },
};
</script>

<style lang="scss" scoped>
.documents {
  padding: 20px;
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
  gap: 12px;
}

.upload-progress {
  margin-bottom: 20px;
  padding: 16px;
  background-color: #f5f7fa;
  border-radius: 8px;

  h4 {
    margin: 0 0 16px 0;
    color: #303133;
    font-size: 16px;
  }
}

.upload-item {
  margin-bottom: 16px;

  &:last-child {
    margin-bottom: 0;
  }
}

.upload-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.filename {
  font-weight: 500;
  color: #303133;
}

.file-size {
  color: #909399;
  font-size: 12px;
}

.loading-container {
  padding: 20px;
}

.empty-state {
  padding: 40px;
  text-align: center;
}

.documents-list {
  margin-top: 20px;
}

.file-info {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-icon {
  font-size: 16px;
  color: #606266;
}

.content-preview {
  color: #606266;
  font-size: 12px;
  line-height: 1.4;
  max-height: 40px;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
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
  .documents {
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

  .documents-list {
    :deep(.el-table) {
      font-size: 12px;
    }

    :deep(.el-table .cell) {
      padding: 8px;
    }
  }

  .upload-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
}

// Table customization
:deep(.el-table) {
  .el-table__row {
    &:hover {
      background-color: #f5f7fa;
    }
  }
}

:deep(.el-upload) {
  .el-button {
    display: flex;
    align-items: center;
    gap: 6px;
  }
}
</style>
