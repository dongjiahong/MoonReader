import { createStore } from "vuex";
import axios from "axios";

// API base URL
const API_BASE_URL = process.env.VUE_APP_API_URL || "http://localhost:3000/api";

// Configure axios
axios.defaults.baseURL = API_BASE_URL;

export default createStore({
  state: {
    // Knowledge bases
    knowledgeBases: [],
    currentKnowledgeBase: null,

    // Documents
    documents: [],

    // AI Quiz
    currentQuestion: null,
    currentAnswer: null,

    // Review
    reviewSession: null,
    reviewHistory: [],

    // AI Configuration
    aiConfig: {
      provider: "deepseek",
      api_key: "",
      api_url: "",
      model_name: "",
      max_tokens: 1000,
      temperature: 0.7,
    },

    // UI State
    loading: false,
    loadingStates: {
      knowledgeBases: false,
      documents: false,
      aiQuiz: false,
      aiConfig: false,
      review: false,
    },
    error: null,
    errors: {
      knowledgeBases: null,
      documents: null,
      aiQuiz: null,
      aiConfig: null,
      review: null,
    },

    // Upload states
    uploadProgress: {},

    // Success messages
    successMessage: null,
  },

  getters: {
    getKnowledgeBaseById: (state) => (id) => {
      return state.knowledgeBases.find((kb) => kb.id === id);
    },

    getDocumentsByKnowledgeBase: (state) => (knowledgeBaseId) => {
      return state.documents.filter(
        (doc) => doc.knowledge_base_id === knowledgeBaseId
      );
    },

    isLoading: (state) => state.loading,
    hasError: (state) => !!state.error,

    // Modular loading states
    isModuleLoading: (state) => (module) =>
      state.loadingStates[module] || false,
    getModuleError: (state) => (module) => state.errors[module],
    hasModuleError: (state) => (module) => !!state.errors[module],

    // Combined loading state
    isAnyLoading: (state) => {
      return (
        state.loading ||
        Object.values(state.loadingStates).some((loading) => loading)
      );
    },

    // Combined error state
    hasAnyError: (state) => {
      return (
        !!state.error || Object.values(state.errors).some((error) => !!error)
      );
    },

    // Upload progress
    getUploadProgress: (state) => (fileId) => state.uploadProgress[fileId] || 0,
    hasActiveUploads: (state) => Object.keys(state.uploadProgress).length > 0,
  },

  mutations: {
    // Loading and error states
    SET_LOADING(state, loading) {
      state.loading = loading;
    },

    SET_MODULE_LOADING(state, { module, loading }) {
      state.loadingStates[module] = loading;
    },

    SET_ERROR(state, error) {
      state.error = error;
    },

    SET_MODULE_ERROR(state, { module, error }) {
      state.errors[module] = error;
    },

    CLEAR_ERROR(state) {
      state.error = null;
    },

    CLEAR_MODULE_ERROR(state, module) {
      state.errors[module] = null;
    },

    CLEAR_ALL_ERRORS(state) {
      state.error = null;
      Object.keys(state.errors).forEach((key) => {
        state.errors[key] = null;
      });
    },

    // Success messages
    SET_SUCCESS_MESSAGE(state, message) {
      state.successMessage = message;
    },

    CLEAR_SUCCESS_MESSAGE(state) {
      state.successMessage = null;
    },

    // Upload progress
    SET_UPLOAD_PROGRESS(state, { fileId, progress }) {
      state.uploadProgress = {
        ...state.uploadProgress,
        [fileId]: progress,
      };
    },

    REMOVE_UPLOAD_PROGRESS(state, fileId) {
      const newProgress = { ...state.uploadProgress };
      delete newProgress[fileId];
      state.uploadProgress = newProgress;
    },

    CLEAR_UPLOAD_PROGRESS(state) {
      state.uploadProgress = {};
    },

    // Knowledge bases
    SET_KNOWLEDGE_BASES(state, knowledgeBases) {
      state.knowledgeBases = knowledgeBases;
    },

    ADD_KNOWLEDGE_BASE(state, knowledgeBase) {
      state.knowledgeBases.push(knowledgeBase);
    },

    UPDATE_KNOWLEDGE_BASE(state, updatedKnowledgeBase) {
      const index = state.knowledgeBases.findIndex(
        (kb) => kb.id === updatedKnowledgeBase.id
      );
      if (index !== -1) {
        state.knowledgeBases.splice(index, 1, updatedKnowledgeBase);
      }
    },

    DELETE_KNOWLEDGE_BASE(state, id) {
      state.knowledgeBases = state.knowledgeBases.filter((kb) => kb.id !== id);
    },

    SET_CURRENT_KNOWLEDGE_BASE(state, knowledgeBase) {
      state.currentKnowledgeBase = knowledgeBase;
    },

    // Documents
    SET_DOCUMENTS(state, documents) {
      state.documents = documents;
    },

    ADD_DOCUMENT(state, document) {
      state.documents.push(document);
    },

    DELETE_DOCUMENT(state, id) {
      state.documents = state.documents.filter((doc) => doc.id !== id);
    },

    // AI Quiz
    SET_CURRENT_QUESTION(state, question) {
      state.currentQuestion = question;
    },

    SET_CURRENT_ANSWER(state, answer) {
      state.currentAnswer = answer;
    },

    // Review
    SET_REVIEW_SESSION(state, session) {
      state.reviewSession = session;
    },

    SET_REVIEW_HISTORY(state, history) {
      state.reviewHistory = history;
    },

    // AI Configuration
    SET_AI_CONFIG(state, config) {
      state.aiConfig = { ...state.aiConfig, ...config };
    },
  },

  actions: {
    // Knowledge bases actions
    async fetchKnowledgeBases({ commit }) {
      commit("SET_MODULE_LOADING", { module: "knowledgeBases", loading: true });
      commit("CLEAR_MODULE_ERROR", "knowledgeBases");
      try {
        const response = await axios.get("/knowledge-bases");
        commit("SET_KNOWLEDGE_BASES", response.data);
        commit("SET_SUCCESS_MESSAGE", "知识库列表加载成功");
      } catch (error) {
        const errorMessage =
          error.response?.data?.error || "获取知识库列表失败";
        commit("SET_MODULE_ERROR", {
          module: "knowledgeBases",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", {
          module: "knowledgeBases",
          loading: false,
        });
      }
    },

    async createKnowledgeBase({ commit }, knowledgeBaseData) {
      commit("SET_MODULE_LOADING", { module: "knowledgeBases", loading: true });
      commit("CLEAR_MODULE_ERROR", "knowledgeBases");
      try {
        const response = await axios.post(
          "/knowledge-bases",
          knowledgeBaseData
        );
        commit("ADD_KNOWLEDGE_BASE", response.data);
        commit(
          "SET_SUCCESS_MESSAGE",
          `知识库 "${response.data.name}" 创建成功`
        );
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "创建知识库失败";
        commit("SET_MODULE_ERROR", {
          module: "knowledgeBases",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", {
          module: "knowledgeBases",
          loading: false,
        });
      }
    },

    async updateKnowledgeBase({ commit }, { id, data }) {
      commit("SET_MODULE_LOADING", { module: "knowledgeBases", loading: true });
      commit("CLEAR_MODULE_ERROR", "knowledgeBases");
      try {
        const response = await axios.put(`/knowledge-bases/${id}`, data);
        commit("UPDATE_KNOWLEDGE_BASE", response.data);
        commit(
          "SET_SUCCESS_MESSAGE",
          `知识库 "${response.data.name}" 更新成功`
        );
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "更新知识库失败";
        commit("SET_MODULE_ERROR", {
          module: "knowledgeBases",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", {
          module: "knowledgeBases",
          loading: false,
        });
      }
    },

    async deleteKnowledgeBase({ commit }, id) {
      commit("SET_MODULE_LOADING", { module: "knowledgeBases", loading: true });
      commit("CLEAR_MODULE_ERROR", "knowledgeBases");
      try {
        await axios.delete(`/knowledge-bases/${id}`);
        commit("DELETE_KNOWLEDGE_BASE", id);
        commit("SET_SUCCESS_MESSAGE", "知识库删除成功");
      } catch (error) {
        const errorMessage = error.response?.data?.error || "删除知识库失败";
        commit("SET_MODULE_ERROR", {
          module: "knowledgeBases",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", {
          module: "knowledgeBases",
          loading: false,
        });
      }
    },

    // Documents actions
    async fetchDocuments({ commit }, knowledgeBaseId) {
      commit("SET_MODULE_LOADING", { module: "documents", loading: true });
      commit("CLEAR_MODULE_ERROR", "documents");
      try {
        const response = await axios.get(
          `/knowledge-bases/${knowledgeBaseId}/documents`
        );
        commit("SET_DOCUMENTS", response.data);
      } catch (error) {
        const errorMessage = error.response?.data?.error || "获取文档列表失败";
        commit("SET_MODULE_ERROR", {
          module: "documents",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "documents", loading: false });
      }
    },

    async uploadDocument(
      { commit },
      { knowledgeBaseId, formData, onProgress }
    ) {
      commit("SET_MODULE_LOADING", { module: "documents", loading: true });
      commit("CLEAR_MODULE_ERROR", "documents");
      try {
        const response = await axios.post(
          `/knowledge-bases/${knowledgeBaseId}/documents`,
          formData,
          {
            headers: {
              "Content-Type": "multipart/form-data",
            },
            onUploadProgress: (progressEvent) => {
              if (onProgress) {
                const progress = Math.round(
                  (progressEvent.loaded * 100) / progressEvent.total
                );
                onProgress(progress);
              }
            },
          }
        );
        commit("ADD_DOCUMENT", response.data);
        commit(
          "SET_SUCCESS_MESSAGE",
          `文档 "${response.data.filename}" 上传成功`
        );
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "文档上传失败";
        commit("SET_MODULE_ERROR", {
          module: "documents",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "documents", loading: false });
      }
    },

    async deleteDocument({ commit }, { id, filename }) {
      commit("SET_MODULE_LOADING", { module: "documents", loading: true });
      commit("CLEAR_MODULE_ERROR", "documents");
      try {
        await axios.delete(`/documents/${id}`);
        commit("DELETE_DOCUMENT", id);
        commit("SET_SUCCESS_MESSAGE", `文档 "${filename}" 删除成功`);
      } catch (error) {
        const errorMessage = error.response?.data?.error || "删除文档失败";
        commit("SET_MODULE_ERROR", {
          module: "documents",
          error: errorMessage,
        });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "documents", loading: false });
      }
    },

    // AI Quiz actions
    async generateQuestion({ commit }, knowledgeBaseId) {
      commit("SET_MODULE_LOADING", { module: "aiQuiz", loading: true });
      commit("CLEAR_MODULE_ERROR", "aiQuiz");
      try {
        const response = await axios.post(
          `/knowledge-bases/${knowledgeBaseId}/generate-question`
        );
        commit("SET_CURRENT_QUESTION", response.data);
        commit("SET_SUCCESS_MESSAGE", "问题生成成功");
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "生成问题失败";
        commit("SET_MODULE_ERROR", { module: "aiQuiz", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "aiQuiz", loading: false });
      }
    },

    async submitAnswer({ commit }, { questionId, answer }) {
      commit("SET_MODULE_LOADING", { module: "aiQuiz", loading: true });
      commit("CLEAR_MODULE_ERROR", "aiQuiz");
      try {
        const response = await axios.post(`/questions/${questionId}/answer`, {
          answer,
        });
        commit("SET_CURRENT_ANSWER", response.data);
        commit("SET_SUCCESS_MESSAGE", "答案提交成功");
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "提交答案失败";
        commit("SET_MODULE_ERROR", { module: "aiQuiz", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "aiQuiz", loading: false });
      }
    },

    // AI Configuration actions
    async fetchAIConfig({ commit }) {
      commit("SET_MODULE_LOADING", { module: "aiConfig", loading: true });
      commit("CLEAR_MODULE_ERROR", "aiConfig");
      try {
        const response = await axios.get("/ai-config");
        commit("SET_AI_CONFIG", response.data);
      } catch (error) {
        const errorMessage = error.response?.data?.error || "获取AI配置失败";
        commit("SET_MODULE_ERROR", { module: "aiConfig", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "aiConfig", loading: false });
      }
    },

    async saveAIConfig({ commit }, config) {
      commit("SET_MODULE_LOADING", { module: "aiConfig", loading: true });
      commit("CLEAR_MODULE_ERROR", "aiConfig");
      try {
        const response = await axios.post("/ai-config", config);
        commit("SET_AI_CONFIG", response.data);
        commit("SET_SUCCESS_MESSAGE", "AI配置保存成功");
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "保存AI配置失败";
        commit("SET_MODULE_ERROR", { module: "aiConfig", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "aiConfig", loading: false });
      }
    },

    async testAIConnection({ commit }, config) {
      commit("SET_MODULE_LOADING", { module: "aiConfig", loading: true });
      commit("CLEAR_MODULE_ERROR", "aiConfig");
      try {
        const response = await axios.post("/ai-config/test", config);
        commit("SET_SUCCESS_MESSAGE", "AI连接测试成功");
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "AI连接测试失败";
        commit("SET_MODULE_ERROR", { module: "aiConfig", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "aiConfig", loading: false });
      }
    },

    // Review actions
    async getRandomReviewQuestion({ commit }, knowledgeBaseId) {
      commit("SET_MODULE_LOADING", { module: "review", loading: true });
      commit("CLEAR_MODULE_ERROR", "review");
      try {
        const response = await axios.get(
          `/knowledge-bases/${knowledgeBaseId}/review/random`
        );
        commit("SET_CURRENT_QUESTION", response.data);
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "获取复习问题失败";
        commit("SET_MODULE_ERROR", { module: "review", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "review", loading: false });
      }
    },

    async fetchReviewHistory({ commit }, knowledgeBaseId) {
      commit("SET_MODULE_LOADING", { module: "review", loading: true });
      commit("CLEAR_MODULE_ERROR", "review");
      try {
        const response = await axios.get(
          `/knowledge-bases/${knowledgeBaseId}/history`
        );
        commit("SET_REVIEW_HISTORY", response.data);
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "获取复习历史失败";
        commit("SET_MODULE_ERROR", { module: "review", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "review", loading: false });
      }
    },

    async saveReviewSession({ commit }, sessionData) {
      commit("SET_MODULE_LOADING", { module: "review", loading: true });
      commit("CLEAR_MODULE_ERROR", "review");
      try {
        const response = await axios.post("/review-sessions", sessionData);
        commit("SET_REVIEW_SESSION", response.data);
        commit("SET_SUCCESS_MESSAGE", "复习会话保存成功");
        return response.data;
      } catch (error) {
        const errorMessage = error.response?.data?.error || "保存复习会话失败";
        commit("SET_MODULE_ERROR", { module: "review", error: errorMessage });
        throw error;
      } finally {
        commit("SET_MODULE_LOADING", { module: "review", loading: false });
      }
    },
  },

  modules: {},
});
