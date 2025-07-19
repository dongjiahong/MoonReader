<template>
  <div id="app">
    <AppLayout />
  </div>
</template>

<script>
import AppLayout from "./components/Layout.vue";

export default {
  name: "App",
  components: {
    AppLayout,
  },
};
</script>

<style lang="scss">
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

#app {
  font-family: "Helvetica Neue", Helvetica, "PingFang SC", "Hiragino Sans GB",
    "Microsoft YaHei", "微软雅黑", Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  color: #2c3e50;
  height: 100vh;
  overflow-x: hidden;
}

body {
  margin: 0;
  padding: 0;
  overflow-x: hidden;
}

// Global responsive breakpoints
$mobile: 768px;
$tablet: 1024px;
$desktop: 1200px;

// Global styles for Element Plus customization
.el-menu-item {
  &:hover {
    background-color: #ecf5ff !important;
  }
}

.el-menu--horizontal .el-menu-item {
  border-bottom: none !important;
}

// Enhanced loading overlay with better UX
.el-loading-mask {
  background-color: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(2px);

  .el-loading-spinner {
    .el-loading-text {
      color: #409eff;
      font-size: 14px;
      margin-top: 10px;
    }
  }
}

// Global loading states
.loading-container {
  padding: 20px;
  text-align: center;

  .el-skeleton {
    margin-bottom: 16px;
  }
}

// Enhanced error display
.error-alert {
  margin-bottom: 20px;
  border-radius: 8px;

  .el-alert__content {
    .el-alert__title {
      font-weight: 600;
    }

    .el-alert__description {
      margin-top: 8px;
      line-height: 1.5;
    }
  }
}

// Global confirmation dialog styles
.el-message-box {
  border-radius: 12px;

  .el-message-box__header {
    padding: 20px 20px 10px;

    .el-message-box__title {
      font-weight: 600;
      font-size: 18px;
    }
  }

  .el-message-box__content {
    padding: 10px 20px 20px;

    .el-message-box__message {
      line-height: 1.6;

      p {
        margin: 8px 0;
      }

      .warning-text {
        color: #e6a23c;
        display: flex;
        align-items: center;
        gap: 8px;
        font-size: 14px;
        margin-top: 12px;
        padding: 8px;
        background-color: #fdf6ec;
        border-radius: 4px;
      }
    }
  }

  .el-message-box__btns {
    padding: 10px 20px 20px;

    .el-button {
      min-width: 80px;
    }
  }
}

// Global responsive utilities
.mobile-only {
  display: none;

  @media (max-width: $mobile) {
    display: block;
  }
}

.desktop-only {
  display: block;

  @media (max-width: $mobile) {
    display: none;
  }
}

// Enhanced button styles for better mobile interaction
.el-button {
  transition: all 0.2s ease;

  &:active {
    transform: scale(0.98);
  }

  @media (max-width: $mobile) {
    min-height: 44px; // Better touch target
    padding: 12px 20px;

    &.el-button--small {
      min-height: 36px;
      padding: 8px 16px;
    }

    &.el-button--mini {
      min-height: 32px;
      padding: 6px 12px;
    }
  }
}

// Enhanced form styles for mobile
.el-form {
  @media (max-width: $mobile) {
    .el-form-item {
      margin-bottom: 20px;

      .el-form-item__label {
        font-weight: 600;
        margin-bottom: 8px;
      }

      .el-input,
      .el-textarea,
      .el-select {
        .el-input__inner,
        .el-textarea__inner {
          font-size: 16px; // Prevent zoom on iOS
          padding: 12px;
        }
      }
    }
  }
}

// Enhanced table responsiveness
.el-table {
  @media (max-width: $mobile) {
    font-size: 14px;

    .el-table__header-wrapper,
    .el-table__body-wrapper {
      .cell {
        padding: 8px 4px;
        word-break: break-word;
      }
    }

    .el-table__row {
      .el-table__cell {
        &:first-child {
          position: sticky;
          left: 0;
          background-color: #fff;
          z-index: 1;
        }
      }
    }
  }
}

// Enhanced card styles
.el-card {
  border-radius: 12px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
  transition: box-shadow 0.3s ease;

  &:hover {
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.12);
  }

  .el-card__header {
    border-bottom: 1px solid #f0f2f5;
    padding: 20px;

    @media (max-width: $mobile) {
      padding: 16px;
    }
  }

  .el-card__body {
    padding: 20px;

    @media (max-width: $mobile) {
      padding: 16px;
    }
  }
}

// Global animation classes
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from {
  transform: translateY(20px);
  opacity: 0;
}

.slide-up-leave-to {
  transform: translateY(-20px);
  opacity: 0;
}

// Enhanced message styles
.el-message {
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);

  @media (max-width: $mobile) {
    left: 16px !important;
    right: 16px !important;
    width: auto !important;
    max-width: none !important;
  }
}

// Enhanced notification styles
.el-notification {
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);

  @media (max-width: $mobile) {
    right: 16px !important;
    width: calc(100vw - 32px) !important;
    max-width: 400px;
  }
}
</style>
