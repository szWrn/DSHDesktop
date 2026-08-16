<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import flashIcon from "./assets/flash.svg";
import { useI18n } from "./i18n";

// i18n
const { t } = useI18n();

const status = ref("");  // 状态消息
const statusVisible = ref(false);  // 状态可见性(用于淡入淡出动画)

const loading = ref(false);
const running = ref(false);

const dsh_url = ref("");

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

let statusTimer = null;
function showStatus(msg, duration = 2500) {
  status.value = msg;
  statusVisible.value = true;
  clearTimeout(statusTimer);
  statusTimer = setTimeout(() => (statusVisible.value = false), duration);
}

onMounted(() => {
  showStatus(t("status.starting"));
  getDSHUrl();
  startDSHService();
  startHealthPolling();
});

// 获取DSH的URL地址
async function getDSHUrl() {
  dsh_url.value = await invoke("get_dsh_url");
}

// 定期检查 DSH 服务运行状态
function startHealthPolling(interval = 3000) {
  setInterval(async () => {
    try {
      running.value = await invoke("is_dsh_service_running");
    } catch {
      running.value = false;
    }
  }, interval);
}

async function startDSHService() {
  loading.value = true;
  try {
    await invoke("start_dsh_service");
    await waitUntilRunning();
  } catch (error) {
    showStatus(t("status.start_failed", { error }), 4000);
  } finally {
    loading.value = false;
  }
}

// 等待服务启动
async function waitUntilRunning(timeoutMs = 15000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await invoke("is_dsh_service_running")) {
      running.value = true;
      showStatus(t("status.started"));
      return;
    }
    await sleep(1000);
  }
  showStatus(t("status.start_timeout"), 5000);
}

async function stopDSHService() {
  loading.value = true;
  try {
    await invoke("kill_dsh_service");
    running.value = false;
    showStatus(t("status.stopped"));
  } catch (error) {
    showStatus(t("status.stop_failed", { error }), 4000);
  } finally {
    loading.value = false;
  }
}

const appWindow = getCurrentWindow();
const minimize = () => appWindow.minimize();
const toggleMaximize = () => appWindow.toggleMaximize();
const close = () => appWindow.close();
</script>

<template>
  <header id="header" data-tauri-drag-region="deep">
    <span class="title">DSH Desktop</span>

    <div class="header-controls">
      <span class="dot" :class="{ online: running }" :title="t('header.service_status')"></span>
      <button type="button" :disabled="loading" @click="startDSHService">{{ t("header.start") }}</button>
      <button type="button" :disabled="loading" @click="stopDSHService">{{ t("header.stop") }}</button>
      <span class="status" :class="{ 'is-hidden': !statusVisible }">{{ status }}</span>
    </div>

    <div class="win-control-buttons">
      <button type="button" :title="t('header.minimize')" :aria-label="t('header.minimize')" @click="minimize">
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1 5h8" /></svg>
      </button>
      <button type="button" :title="t('header.maximize')" :aria-label="t('header.maximize')" @click="toggleMaximize">
        <svg viewBox="0 0 10 10" aria-hidden="true"><rect x="1" y="1" width="8" height="8" /></svg>
      </button>
      <button type="button" class="close" :title="t('header.close')" :aria-label="t('header.close')" @click="close">
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M1 1l8 8M9 1L1 9" /></svg>
      </button>
    </div>
  </header>

  <main class="container">
    <div v-if="!running" class="launcher">
      <img :src="flashIcon" class="flash-icon" alt="DSH" />
    </div>
    <iframe v-else :src="dsh_url" class="dsh-frame" />
  </main>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;

  --header-bg: #f9fafb;
  --header-fg: #1b1b1c;
  --header-hover: rgba(0, 0, 0, 0.06);
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;

    --header-bg: #1b1b1c;
    --header-fg: #f9fafb;
    --header-hover: rgba(255, 255, 255, 0.12);
  }
}

#header {
  display: flex;
  align-items: center;
  height: 36px;
  background-color: var(--header-bg);
  color: var(--header-fg);
  user-select: none;
}

#header .title {
  padding: 0 12px;
  font-size: 13px;
  font-weight: 500;
  white-space: nowrap;
}

.header-controls {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 8px;
}

.header-controls button {
  font-family: inherit;
  font-size: 12px;
  line-height: 1;
  padding: 5px 12px;
  border: 1px solid rgba(128, 128, 128, 0.35);
  border-radius: 4px;
  background: transparent;
  color: var(--header-fg);
  box-shadow: none;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.header-controls button:hover:not(:disabled) {
  background-color: var(--header-hover);
}

.header-controls button:disabled {
  opacity: 0.4;
  cursor: default;
}

.status {
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 220px;
  opacity: 1;
  transition: opacity 0.5s ease;
}

.status.is-hidden {
  opacity: 0;
}

.dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: #e81123;
  flex-shrink: 0;
  transition: background-color 0.3s ease;
}

.dot.online {
  background-color: #2ecc71;
}

.win-control-buttons {
  display: flex;
  align-self: stretch;
}

.win-control-buttons button {
  width: 46px;
  border: none;
  background: transparent;
  color: var(--header-fg);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.15s ease;
}

.win-control-buttons button:hover {
  background-color: var(--header-hover);
}

.win-control-buttons button.close:hover {
  background-color: #e81123;
  color: #fff;
}

.win-control-buttons svg {
  width: 12px;
  height: 12px;
  fill: none;
  stroke: currentColor;
  stroke-width: 1;
}

.container {
  margin: 0;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.launcher {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
}

.flash-icon {
  width: 200px;
  height: 200px;
}

.dsh-frame {
  flex: 1;
  width: 100%;
  border: none;
}
</style>
