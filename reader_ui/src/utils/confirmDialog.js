import { ElMessageBox } from "element-plus";
import { h } from "vue";

/**
 * 显示确认对话框
 * @param {Object} options - 对话框选项
 * @param {string} options.title - 标题
 * @param {string} options.message - 消息内容
 * @param {string} options.type - 类型 (warning, error, info, success)
 * @param {string} options.confirmButtonText - 确认按钮文本
 * @param {string} options.cancelButtonText - 取消按钮文本
 * @param {string} options.confirmButtonType - 确认按钮类型
 * @param {boolean} options.dangerouslyUseHTMLString - 是否使用HTML字符串
 * @param {string} options.warningText - 警告文本
 * @returns {Promise} 确认结果
 */
export const showConfirmDialog = async (options = {}) => {
  const {
    title = "确认操作",
    message = "确定要执行此操作吗？",
    type = "warning",
    confirmButtonText = "确定",
    cancelButtonText = "取消",
    confirmButtonType = "primary",
    dangerouslyUseHTMLString = false,
    warningText = null,
  } = options;

  try {
    // 构建消息内容
    let messageContent = message;

    if (warningText) {
      if (dangerouslyUseHTMLString) {
        messageContent = `
          <p>${message}</p>
          <div class="warning-text">
            <i class="el-icon-warning"></i>
            ${warningText}
          </div>
        `;
      } else {
        // 使用 VNode 渲染
        messageContent = h("div", [
          h("p", { style: { margin: "8px 0" } }, message),
          h(
            "div",
            {
              class: "warning-text",
              style: {
                color: "#e6a23c",
                display: "flex",
                alignItems: "center",
                gap: "8px",
                fontSize: "14px",
                marginTop: "12px",
                padding: "8px",
                backgroundColor: "#fdf6ec",
                borderRadius: "4px",
              },
            },
            [h("i", { class: "el-icon-warning" }), warningText]
          ),
        ]);
      }
    }

    const result = await ElMessageBox.confirm(messageContent, title, {
      confirmButtonText,
      cancelButtonText,
      type,
      confirmButtonType,
      dangerouslyUseHTMLString,
      customClass: "custom-confirm-dialog",
      beforeClose: (action, instance, done) => {
        if (action === "confirm") {
          instance.confirmButtonLoading = true;
          instance.confirmButtonText = "处理中...";
          setTimeout(() => {
            done();
          }, 300);
        } else {
          done();
        }
      },
    });

    return result === "confirm";
  } catch (error) {
    // 用户取消操作
    return false;
  }
};

/**
 * 显示删除确认对话框
 * @param {string} itemName - 要删除的项目名称
 * @param {string} itemType - 项目类型 (如: 知识库, 文档)
 * @param {Object} options - 额外选项
 * @returns {Promise<boolean>} 确认结果
 */
export const showDeleteConfirmDialog = async (
  itemName,
  itemType = "项目",
  options = {}
) => {
  return await showConfirmDialog({
    title: "确认删除",
    message: `确定要删除${itemType} "${itemName}" 吗？`,
    warningText: "此操作无法恢复！",
    confirmButtonText: "确认删除",
    confirmButtonType: "danger",
    type: "warning",
    ...options,
  });
};

/**
 * 显示批量操作确认对话框
 * @param {number} count - 操作数量
 * @param {string} action - 操作类型
 * @param {Object} options - 额外选项
 * @returns {Promise<boolean>} 确认结果
 */
export const showBatchConfirmDialog = async (
  count,
  action = "操作",
  options = {}
) => {
  return await showConfirmDialog({
    title: `确认批量${action}`,
    message: `确定要${action} ${count} 个项目吗？`,
    warningText: count > 10 ? "数量较多，请谨慎操作！" : null,
    confirmButtonText: `确认${action}`,
    type: "warning",
    ...options,
  });
};

/**
 * 显示保存确认对话框
 * @param {string} message - 消息内容
 * @param {Object} options - 额外选项
 * @returns {Promise<boolean>} 确认结果
 */
export const showSaveConfirmDialog = async (
  message = "确定要保存更改吗？",
  options = {}
) => {
  return await showConfirmDialog({
    title: "保存更改",
    message,
    confirmButtonText: "保存",
    confirmButtonType: "primary",
    type: "info",
    ...options,
  });
};

/**
 * 显示离开页面确认对话框
 * @param {Object} options - 额外选项
 * @returns {Promise<boolean>} 确认结果
 */
export const showLeaveConfirmDialog = async (options = {}) => {
  return await showConfirmDialog({
    title: "确认离开",
    message: "您有未保存的更改，确定要离开此页面吗？",
    warningText: "离开后未保存的更改将丢失！",
    confirmButtonText: "确认离开",
    confirmButtonType: "warning",
    type: "warning",
    ...options,
  });
};
