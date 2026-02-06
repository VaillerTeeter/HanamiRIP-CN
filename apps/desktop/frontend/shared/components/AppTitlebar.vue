<script setup lang="ts">
/**
 * 应用标题栏：
 * - 展示应用标题与导航按钮
 * - 提供最小化/关闭按钮
 */
import { NButton } from "naive-ui";
import type { PageKey } from "../types/page";

/**
 * activePage：当前页面
 * onMinimize/onClose：窗口控制函数
 */
const props = defineProps<{
  activePage: PageKey;
  onMinimize: () => void | Promise<void>;
  onClose: () => void | Promise<void>;
}>();

/**
 * emit：切换页面事件。
 */
const emit = defineEmits<{ (e: "switch", page: PageKey): void }>();

// 当前按钮是否处于激活态。
const isActive = (page: PageKey) => props.activePage === page;

// 切换页面并通知父组件。
const switchPage = (page: PageKey) => {
  emit("switch", page);
};
</script>

<template>
  <!-- 自定义标题栏（支持拖拽区域） -->
  <header class="app-titlebar" data-tauri-drag-region>
    <div class="titlebar-drag-region" data-tauri-drag-region>
      <div class="titlebar-left" data-tauri-drag-region>
        <span class="app-title">HanamiRIP</span>
        <span class="app-subtitle">番剧助手</span>
      </div>
      <div class="titlebar-nav" data-tauri-drag-region>
        <NButton
          secondary
          :type="isActive('query') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('query')"
        >
          季度查询
        </NButton>
        <NButton
          secondary
          :type="isActive('watching') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('watching')"
        >
          正在追番
        </NButton>
        <NButton
          secondary
          :type="isActive('backlog') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('backlog')"
        >
          补番计划
        </NButton>
        <NButton
          secondary
          :type="isActive('finished') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('finished')"
        >
          已完番剧
        </NButton>
        <NButton
          secondary
          :type="isActive('search') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('search')"
        >
          搜索资源
        </NButton>
        <NButton
          secondary
          :type="isActive('download') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('download')"
        >
          下载
        </NButton>
        <NButton
          secondary
          :type="isActive('tracks') ? 'primary' : 'default'"
          :data-tauri-drag-region="false"
          @click="switchPage('tracks')"
        >
          轨道工坊
        </NButton>
      </div>
    </div>
    <!-- 右侧窗口控制按钮 -->
    <div class="titlebar-actions" aria-label="window actions" data-tauri-drag-region="false">
      <button
        class="titlebar-control"
        type="button"
        aria-label="最小化"
        data-tauri-drag-region="false"
        @click="props.onMinimize"
      >
        <span class="titlebar-icon">−</span>
      </button>
      <button
        class="titlebar-control titlebar-close"
        type="button"
        aria-label="关闭"
        data-tauri-drag-region="false"
        @click="props.onClose"
      >
        <span class="titlebar-icon">×</span>
      </button>
    </div>
  </header>
</template>
