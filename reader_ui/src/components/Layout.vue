<template>
  <el-container class="app-container">
    <!-- Header -->
    <el-header class="app-header">
      <div class="header-content">
        <div class="header-left">
          <h1 class="app-title">知识积累系统</h1>
          <!-- Mobile menu toggle -->
          <el-button
            class="mobile-menu-toggle mobile-only"
            link
            @click="toggleMobileMenu"
          >
            <el-icon size="20"><Menu /></el-icon>
          </el-button>
        </div>

        <!-- Desktop menu -->
        <el-menu
          mode="horizontal"
          :default-active="activeMenu"
          class="header-menu desktop-only"
          @select="handleMenuSelect"
        >
          <el-menu-item index="/">
            <el-icon><House /></el-icon>
            <span>首页</span>
          </el-menu-item>
          <el-menu-item index="/knowledge-bases">
            <el-icon><Collection /></el-icon>
            <span>知识库</span>
          </el-menu-item>
          <el-menu-item index="/settings">
            <el-icon><Setting /></el-icon>
            <span>设置</span>
          </el-menu-item>
        </el-menu>
      </div>

      <!-- Mobile menu drawer -->
      <el-drawer
        v-model="showMobileMenu"
        direction="ltr"
        size="280px"
        :with-header="false"
        class="mobile-menu-drawer"
      >
        <div class="mobile-menu-content">
          <div class="mobile-menu-header">
            <h2>菜单</h2>
            <el-button link @click="showMobileMenu = false">
              <el-icon><Close /></el-icon>
            </el-button>
          </div>

          <el-menu
            :default-active="activeMenu"
            class="mobile-menu"
            @select="handleMobileMenuSelect"
          >
            <el-menu-item index="/">
              <el-icon><House /></el-icon>
              <span>首页</span>
            </el-menu-item>
            <el-menu-item index="/knowledge-bases">
              <el-icon><Collection /></el-icon>
              <span>知识库</span>
            </el-menu-item>
            <el-menu-item index="/settings">
              <el-icon><Setting /></el-icon>
              <span>设置</span>
            </el-menu-item>
          </el-menu>
        </div>
      </el-drawer>
    </el-header>

    <!-- Main Content -->
    <el-container>
      <!-- Sidebar (conditional) -->
      <el-aside v-if="showSidebar" width="250px" class="app-sidebar">
        <el-menu
          :default-active="activeSubmenu"
          class="sidebar-menu"
          @select="handleSubmenuSelect"
        >
          <template v-if="currentKnowledgeBase">
            <div class="knowledge-base-info">
              <h3>{{ currentKnowledgeBase.name }}</h3>
              <p>{{ currentKnowledgeBase.description }}</p>
            </div>
            <el-menu-item
              :index="`/knowledge-bases/${currentKnowledgeBase.id}`"
            >
              <el-icon><InfoFilled /></el-icon>
              <span>概览</span>
            </el-menu-item>
            <el-menu-item
              :index="`/knowledge-bases/${currentKnowledgeBase.id}/documents`"
            >
              <el-icon><Document /></el-icon>
              <span>文档管理</span>
            </el-menu-item>
            <el-menu-item
              :index="`/knowledge-bases/${currentKnowledgeBase.id}/quiz`"
            >
              <el-icon><ChatDotRound /></el-icon>
              <span>AI问答</span>
            </el-menu-item>
            <el-menu-item
              :index="`/knowledge-bases/${currentKnowledgeBase.id}/review`"
            >
              <el-icon><Refresh /></el-icon>
              <span>复习模块</span>
            </el-menu-item>
            <el-menu-item
              :index="`/knowledge-bases/${currentKnowledgeBase.id}/history`"
            >
              <el-icon><Clock /></el-icon>
              <span>历史记录</span>
            </el-menu-item>
          </template>
        </el-menu>
      </el-aside>

      <!-- Main Content Area -->
      <el-main class="app-main">
        <!-- Loading Overlay -->
        <div v-if="loading" v-loading="loading" class="loading-overlay"></div>

        <!-- Error Alert -->
        <el-alert
          v-if="error"
          :title="error"
          type="error"
          show-icon
          :closable="true"
          @close="clearError"
          class="error-alert"
        />

        <!-- Router View -->
        <router-view />
      </el-main>
    </el-container>
  </el-container>
</template>

<script>
import { computed, watch, ref } from "vue";
import { useStore } from "vuex";
import { useRoute, useRouter } from "vue-router";
import {
  House,
  Collection,
  Setting,
  InfoFilled,
  Document,
  ChatDotRound,
  Refresh,
  Clock,
  Menu,
  Close,
} from "@element-plus/icons-vue";

export default {
  name: "AppLayout",
  components: {
    House,
    Collection,
    Setting,
    InfoFilled,
    Document,
    ChatDotRound,
    Refresh,
    Clock,
    Menu,
    Close,
  },
  setup() {
    const store = useStore();
    const route = useRoute();
    const router = useRouter();

    // Reactive data
    const showMobileMenu = ref(false);

    // Computed properties
    const loading = computed(() => store.getters.isLoading);
    const error = computed(() => store.state.error);
    const currentKnowledgeBase = computed(
      () => store.state.currentKnowledgeBase
    );

    const showSidebar = computed(() => {
      return route.path.includes("/knowledge-bases/") && route.params.id;
    });

    const activeMenu = computed(() => {
      if (route.path === "/") return "/";
      if (route.path.startsWith("/knowledge-bases")) return "/knowledge-bases";
      if (route.path.startsWith("/settings")) return "/settings";
      return "/";
    });

    const activeSubmenu = computed(() => {
      return route.path;
    });

    // Methods
    const handleMenuSelect = (index) => {
      router.push(index);
    };

    const handleMobileMenuSelect = (index) => {
      router.push(index);
      showMobileMenu.value = false;
    };

    const handleSubmenuSelect = (index) => {
      router.push(index);
    };

    const toggleMobileMenu = () => {
      showMobileMenu.value = !showMobileMenu.value;
    };

    const clearError = () => {
      store.commit("CLEAR_ERROR");
    };

    // Watch for route changes to update current knowledge base
    watch(
      () => route.params.id,
      async (newId) => {
        if (newId && route.path.includes("/knowledge-bases/")) {
          const knowledgeBase = store.getters.getKnowledgeBaseById(newId);
          if (knowledgeBase) {
            store.commit("SET_CURRENT_KNOWLEDGE_BASE", knowledgeBase);
          } else {
            // Fetch knowledge bases if not loaded
            try {
              await store.dispatch("fetchKnowledgeBases");
              const kb = store.getters.getKnowledgeBaseById(newId);
              if (kb) {
                store.commit("SET_CURRENT_KNOWLEDGE_BASE", kb);
              }
            } catch (error) {
              console.error("Failed to fetch knowledge base:", error);
            }
          }
        } else {
          store.commit("SET_CURRENT_KNOWLEDGE_BASE", null);
        }
      },
      { immediate: true }
    );

    // Close mobile menu on route change
    watch(
      () => route.path,
      () => {
        showMobileMenu.value = false;
      }
    );

    return {
      loading,
      error,
      currentKnowledgeBase,
      showSidebar,
      activeMenu,
      activeSubmenu,
      showMobileMenu,
      handleMenuSelect,
      handleMobileMenuSelect,
      handleSubmenuSelect,
      toggleMobileMenu,
      clearError,
    };
  },
};
</script>

<style lang="scss" scoped>
$mobile: 768px;
$tablet: 1024px;

.app-container {
  height: 100vh;
  overflow: hidden;
}

.app-header {
  background-color: #fff;
  border-bottom: 1px solid #e4e7ed;
  padding: 0;
  position: relative;
  z-index: 1000;
}

.header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 20px;

  @media (max-width: $mobile) {
    padding: 0 16px;
  }
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.app-title {
  margin: 0;
  color: #303133;
  font-size: 20px;
  font-weight: 600;

  @media (max-width: $mobile) {
    font-size: 18px;
  }
}

.mobile-menu-toggle {
  padding: 8px;

  &:hover {
    background-color: #f5f7fa;
  }
}

.header-menu {
  border-bottom: none;

  .el-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;

    span {
      font-weight: 500;
    }
  }
}

// Mobile menu drawer
.mobile-menu-drawer {
  :deep(.el-drawer__body) {
    padding: 0;
  }
}

.mobile-menu-content {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.mobile-menu-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px;
  border-bottom: 1px solid #e4e7ed;
  background-color: #fff;

  h2 {
    margin: 0;
    font-size: 18px;
    font-weight: 600;
    color: #303133;
  }
}

.mobile-menu {
  flex: 1;
  border-right: none;

  .el-menu-item {
    height: 56px;
    line-height: 56px;
    padding: 0 20px;
    display: flex;
    align-items: center;
    gap: 12px;

    span {
      font-size: 16px;
      font-weight: 500;
    }

    &:hover {
      background-color: #f5f7fa;
    }

    &.is-active {
      background-color: #ecf5ff;
      color: #409eff;
    }
  }
}

.app-sidebar {
  background-color: #f5f7fa;
  border-right: 1px solid #e4e7ed;
  transition: width 0.3s ease;

  @media (max-width: $mobile) {
    position: fixed;
    left: 0;
    top: 60px;
    height: calc(100vh - 60px);
    z-index: 999;
    width: 280px !important;
    transform: translateX(-100%);

    &.mobile-sidebar-open {
      transform: translateX(0);
    }
  }
}

.knowledge-base-info {
  padding: 20px;
  border-bottom: 1px solid #e4e7ed;
  margin-bottom: 10px;
  background-color: #fff;

  h3 {
    margin: 0 0 8px 0;
    color: #303133;
    font-size: 16px;
    font-weight: 600;
    line-height: 1.4;
    word-break: break-word;
  }

  p {
    margin: 0;
    color: #606266;
    font-size: 14px;
    line-height: 1.4;
    word-break: break-word;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  @media (max-width: $mobile) {
    padding: 16px;

    h3 {
      font-size: 15px;
    }

    p {
      font-size: 13px;
    }
  }
}

.sidebar-menu {
  border-right: none;
  background-color: transparent;

  .el-menu-item {
    height: 48px;
    line-height: 48px;
    margin: 2px 8px;
    border-radius: 6px;

    &:hover {
      background-color: #ecf5ff;
    }

    &.is-active {
      background-color: #409eff;
      color: #fff;

      .el-icon {
        color: #fff;
      }
    }

    .el-icon {
      margin-right: 8px;
    }

    span {
      font-weight: 500;
    }
  }
}

.app-main {
  background-color: #f5f7fa;
  position: relative;
  overflow-y: auto;
  height: calc(100vh - 60px);

  @media (max-width: $mobile) {
    padding: 0;
  }
}

.error-alert {
  margin: 16px;
  border-radius: 8px;

  :deep(.el-alert__content) {
    .el-alert__title {
      font-weight: 600;
      font-size: 15px;
    }
  }

  @media (max-width: $mobile) {
    margin: 12px;
    font-size: 14px;
  }
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 1000;
}

// Loading overlay enhancements
:deep(.el-loading-mask) {
  background-color: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);

  .el-loading-spinner {
    .el-loading-text {
      color: #409eff;
      font-size: 14px;
      margin-top: 12px;
      font-weight: 500;
    }

    .circular {
      width: 42px;
      height: 42px;
    }
  }
}

// Mobile sidebar overlay
@media (max-width: $mobile) {
  .sidebar-overlay {
    position: fixed;
    top: 60px;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 998;
    opacity: 0;
    visibility: hidden;
    transition: all 0.3s ease;

    &.active {
      opacity: 1;
      visibility: visible;
    }
  }
}

// Animation classes
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-enter-active,
.slide-leave-active {
  transition: transform 0.3s ease;
}

.slide-enter-from {
  transform: translateX(-100%);
}

.slide-leave-to {
  transform: translateX(-100%);
}
</style>
