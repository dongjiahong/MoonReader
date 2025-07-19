<template>
  <div class="settings">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>系统设置</span>
        </div>
      </template>

      <div class="content">
        <el-tabs v-model="activeTab" type="border-card">
          <el-tab-pane label="AI配置" name="ai-config">
            <div class="ai-config-content">
              <!-- AI Provider Selection -->
              <el-card class="config-section">
                <template #header>
                  <div class="section-header">
                    <el-icon>
                      <Setting />
                    </el-icon>
                    <span>AI服务提供商</span>
                  </div>
                </template>

                <el-form
                  :model="configForm"
                  :rules="configRules"
                  ref="configFormRef"
                  label-width="120px"
                >
                  <el-form-item label="服务提供商" prop="provider">
                    <el-radio-group
                      v-model="configForm.provider"
                      @change="onProviderChange"
                    >
                      <el-radio value="deepseek">DeepSeek</el-radio>
                      <el-radio value="openai">OpenAI</el-radio>
                      <el-radio value="local">本地AI接口</el-radio>
                    </el-radio-group>
                  </el-form-item>

                  <!-- DeepSeek Configuration -->
                  <template v-if="configForm.provider === 'deepseek'">
                    <el-form-item label="API密钥" prop="api_key">
                      <el-input
                        v-model="configForm.api_key"
                        type="password"
                        placeholder="请输入DeepSeek API密钥"
                        show-password
                        clearable
                      />
                      <div class="form-help">
                        <el-text type="info" size="small">
                          获取API密钥请访问
                          <el-link
                            href="https://platform.deepseek.com"
                            target="_blank"
                            type="primary"
                          >
                            DeepSeek平台
                          </el-link>
                        </el-text>
                      </div>
                    </el-form-item>

                    <el-form-item label="模型名称" prop="model_name">
                      <el-select
                        v-model="configForm.model_name"
                        placeholder="选择模型"
                      >
                        <el-option
                          label="deepseek-chat"
                          value="deepseek-chat"
                        />
                        <el-option
                          label="deepseek-coder"
                          value="deepseek-coder"
                        />
                      </el-select>
                    </el-form-item>
                  </template>

                  <!-- OpenAI Configuration -->
                  <template v-if="configForm.provider === 'openai'">
                    <el-form-item label="API密钥" prop="api_key">
                      <el-input
                        v-model="configForm.api_key"
                        type="password"
                        placeholder="请输入OpenAI API密钥"
                        show-password
                        clearable
                      />
                      <div class="form-help">
                        <el-text type="info" size="small">
                          获取API密钥请访问
                          <el-link
                            href="https://platform.openai.com"
                            target="_blank"
                            type="primary"
                          >
                            OpenAI平台
                          </el-link>
                        </el-text>
                      </div>
                    </el-form-item>

                    <el-form-item label="API地址" prop="api_url">
                      <el-input
                        v-model="configForm.api_url"
                        placeholder="https://api.openai.com/v1"
                        clearable
                      />
                      <div class="form-help">
                        <el-text type="info" size="small">
                          默认使用官方API地址，如需使用代理请修改
                        </el-text>
                      </div>
                    </el-form-item>

                    <el-form-item label="模型名称" prop="model_name">
                      <el-select
                        v-model="configForm.model_name"
                        placeholder="选择模型"
                      >
                        <el-option
                          label="gpt-3.5-turbo"
                          value="gpt-3.5-turbo"
                        />
                        <el-option label="gpt-4" value="gpt-4" />
                        <el-option label="gpt-4-turbo" value="gpt-4-turbo" />
                      </el-select>
                    </el-form-item>
                  </template>

                  <!-- Local AI Configuration -->
                  <template v-if="configForm.provider === 'local'">
                    <el-form-item label="API地址" prop="api_url">
                      <el-input
                        v-model="configForm.api_url"
                        placeholder="http://localhost:11434/api/generate"
                        clearable
                      />
                      <div class="form-help">
                        <el-text type="info" size="small">
                          请输入本地AI服务的API地址，如Ollama等
                        </el-text>
                      </div>
                    </el-form-item>

                    <el-form-item label="模型名称" prop="model_name">
                      <el-input
                        v-model="configForm.model_name"
                        placeholder="llama2, qwen等"
                        clearable
                      />
                      <div class="form-help">
                        <el-text type="info" size="small">
                          请输入本地AI服务支持的模型名称
                        </el-text>
                      </div>
                    </el-form-item>
                  </template>
                </el-form>
              </el-card>

              <!-- Advanced Settings -->
              <el-card class="config-section">
                <template #header>
                  <div class="section-header">
                    <el-icon>
                      <Tools />
                    </el-icon>
                    <span>高级设置</span>
                  </div>
                </template>

                <el-form :model="configForm" label-width="120px">
                  <el-form-item label="最大令牌数">
                    <el-slider
                      v-model="configForm.max_tokens"
                      :min="100"
                      :max="4000"
                      :step="100"
                      show-input
                      :show-input-controls="false"
                    />
                    <div class="form-help">
                      <el-text type="info" size="small">
                        控制AI响应的最大长度，较大的值会产生更详细的回答
                      </el-text>
                    </div>
                  </el-form-item>

                  <el-form-item label="温度参数">
                    <el-slider
                      v-model="configForm.temperature"
                      :min="0"
                      :max="2"
                      :step="0.1"
                      show-input
                      :show-input-controls="false"
                    />
                    <div class="form-help">
                      <el-text type="info" size="small">
                        控制AI回答的创造性，0为最保守，2为最有创造性
                      </el-text>
                    </div>
                  </el-form-item>
                </el-form>
              </el-card>

              <!-- Connection Test -->
              <el-card class="config-section">
                <template #header>
                  <div class="section-header">
                    <el-icon>
                      <Connection />
                    </el-icon>
                    <span>连接测试</span>
                  </div>
                </template>

                <div class="test-section">
                  <el-alert
                    v-if="testResult"
                    :title="testResult.success ? '连接成功' : '连接失败'"
                    :type="testResult.success ? 'success' : 'error'"
                    :description="testResult.message"
                    show-icon
                    :closable="false"
                    class="test-result"
                  />

                  <div class="test-actions">
                    <el-button
                      type="primary"
                      :loading="testing"
                      :disabled="!isConfigValid"
                      @click="testConnection"
                    >
                      <el-icon>
                        <Connection />
                      </el-icon>
                      测试连接
                    </el-button>

                    <el-text v-if="!isConfigValid" type="warning" size="small">
                      请先完成必要的配置项
                    </el-text>
                  </div>
                </div>
              </el-card>

              <!-- Action Buttons -->
              <div class="action-buttons">
                <el-button
                  type="primary"
                  size="large"
                  :loading="saving"
                  @click="saveConfig"
                >
                  <el-icon>
                    <Check />
                  </el-icon>
                  保存配置
                </el-button>

                <el-button size="large" @click="resetConfig">
                  <el-icon>
                    <RefreshLeft />
                  </el-icon>
                  重置配置
                </el-button>
              </div>
            </div>
          </el-tab-pane>

          <el-tab-pane label="系统设置" name="system">
            <div class="tab-content">
              <el-empty description="系统设置功能开发中">
                <el-button type="primary" disabled>敬请期待</el-button>
              </el-empty>
            </div>
          </el-tab-pane>
        </el-tabs>
      </div>
    </el-card>
  </div>
</template>

<script>
import { ref, computed, onMounted, reactive } from "vue";
import { useStore } from "vuex";
import { ElMessage, ElMessageBox, ElNotification } from "element-plus";
import {
  Setting,
  Tools,
  Connection,
  Check,
  RefreshLeft,
} from "@element-plus/icons-vue";

export default {
  name: "SettingsView",
  components: {
    Setting,
    Tools,
    Connection,
    Check,
    RefreshLeft,
  },
  setup() {
    const store = useStore();
    const activeTab = ref("ai-config");
    const configFormRef = ref(null);
    const testResult = ref(null);

    // Computed properties for loading states
    const saving = computed(() => store.getters.isModuleLoading("aiConfig"));
    const testing = computed(() => store.getters.isModuleLoading("aiConfig"));

    // Form data
    const configForm = reactive({
      provider: "deepseek",
      api_key: "",
      api_url: "",
      model_name: "",
      max_tokens: 1000,
      temperature: 0.7,
    });

    // Form validation rules
    const configRules = {
      provider: [
        { required: true, message: "请选择AI服务提供商", trigger: "change" },
      ],
      api_key: [
        {
          validator: (rule, value, callback) => {
            if (configForm.provider !== "local" && !value) {
              callback(new Error("请输入API密钥"));
            } else {
              callback();
            }
          },
          trigger: "blur",
        },
      ],
      api_url: [
        {
          validator: (rule, value, callback) => {
            if (configForm.provider === "local" && !value) {
              callback(new Error("请输入API地址"));
            } else {
              callback();
            }
          },
          trigger: "blur",
        },
      ],
      model_name: [
        { required: true, message: "请输入或选择模型名称", trigger: "blur" },
      ],
    };

    // Computed properties
    const isConfigValid = computed(() => {
      if (configForm.provider === "local") {
        return !!(configForm.api_url && configForm.model_name);
      }
      return !!(configForm.api_key && configForm.model_name);
    });

    // Methods
    const onProviderChange = (provider) => {
      // Reset form when provider changes
      configForm.api_key = "";
      configForm.api_url = "";
      configForm.model_name = "";
      testResult.value = null;

      // Set default values based on provider
      if (provider === "deepseek") {
        configForm.model_name = "deepseek-chat";
      } else if (provider === "openai") {
        configForm.api_url = "https://api.openai.com/v1";
        configForm.model_name = "gpt-3.5-turbo";
      } else if (provider === "local") {
        configForm.api_url = "http://localhost:11434/api/generate";
      }
    };

    const loadConfig = async () => {
      try {
        await store.dispatch("fetchAIConfig");
        const config = store.state.aiConfig;

        // Update form with loaded config
        Object.assign(configForm, {
          provider: config.provider || "deepseek",
          api_key: config.api_key || "",
          api_url: config.api_url || "",
          model_name: config.model_name || "",
          max_tokens: config.max_tokens || 1000,
          temperature: config.temperature || 0.7,
        });

        // Set default values if needed
        if (!configForm.model_name) {
          onProviderChange(configForm.provider);
        }
      } catch (error) {
        console.error("Failed to load AI config:", error);
        ElMessage.warning("加载AI配置失败，使用默认配置");
      }
    };

    const saveConfig = async () => {
      try {
        // Validate form
        const valid = await configFormRef.value?.validate();
        if (!valid) {
          return;
        }

        testResult.value = null;

        await store.dispatch("saveAIConfig", { ...configForm });
        ElNotification.success({
          title: "保存成功",
          message: "AI配置已保存",
          duration: 3000,
        });
      } catch (error) {
        console.error("Save config error:", error);
        ElMessage.error("保存配置失败，请重试");
      }
    };

    const testConnection = async () => {
      try {
        testResult.value = null;

        const result = await store.dispatch("testAIConnection", {
          ...configForm,
        });

        testResult.value = {
          success: true,
          message: result.message || "AI服务连接正常，可以正常使用问答功能",
        };

        ElMessage.success("连接测试成功");
      } catch (error) {
        console.error("Test connection error:", error);

        testResult.value = {
          success: false,
          message:
            error.response?.data?.details ||
            error.message ||
            "连接测试失败，请检查配置",
        };

        ElMessage.error("连接测试失败");
      }
    };

    const resetConfig = async () => {
      try {
        const result = await ElMessageBox.confirm(
          "重置配置将清除所有AI配置信息，是否继续？",
          "确认重置",
          {
            confirmButtonText: "确定",
            cancelButtonText: "取消",
            type: "warning",
          }
        );

        if (result === "confirm") {
          // Reset to default values
          Object.assign(configForm, {
            provider: "deepseek",
            api_key: "",
            api_url: "",
            model_name: "deepseek-chat",
            max_tokens: 1000,
            temperature: 0.7,
          });

          testResult.value = null;
          ElMessage.success("配置已重置");
        }
      } catch (error) {
        // User cancelled
      }
    };

    // Lifecycle hooks
    onMounted(() => {
      loadConfig();
    });

    return {
      activeTab,
      configForm,
      configRules,
      configFormRef,
      saving,
      testing,
      testResult,
      isConfigValid,
      onProviderChange,
      saveConfig,
      testConnection,
      resetConfig,
    };
  },
};
</script>

<style lang="scss" scoped>
.settings {
  padding: 20px;
  max-width: 1000px;
  margin: 0 auto;
}

.card-header {
  font-size: 20px;
  font-weight: 600;
}

.content {
  padding: 20px;
}

.ai-config-content {
  .config-section {
    margin-bottom: 24px;

    .section-header {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 16px;
      font-weight: 600;
    }
  }

  .form-help {
    margin-top: 8px;
    line-height: 1.4;
  }

  .test-section {
    .test-result {
      margin-bottom: 16px;
    }

    .test-actions {
      display: flex;
      align-items: center;
      gap: 12px;
    }
  }

  .action-buttons {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin-top: 32px;
    padding-top: 24px;
    border-top: 1px solid #e4e7ed;
  }
}

.tab-content {
  text-align: center;
  padding: 60px 20px;
}

// Responsive design
@media (max-width: 768px) {
  .settings {
    padding: 10px;
  }

  .content {
    padding: 10px;
  }

  .ai-config-content {
    .action-buttons {
      flex-direction: column;

      .el-button {
        width: 100%;
      }
    }
  }

  :deep(.el-form-item__label) {
    width: 100px !important;
  }
}

// Form styling
:deep(.el-radio-group) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

:deep(.el-slider) {
  margin-right: 16px;
}

:deep(.el-form-item) {
  margin-bottom: 24px;
}

// Card styling
.config-section {
  transition: all 0.3s ease;
}

.config-section:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}
</style>
