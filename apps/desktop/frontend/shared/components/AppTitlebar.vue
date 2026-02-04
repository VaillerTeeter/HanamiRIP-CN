<script setup lang="ts">
import { NButton } from "naive-ui";
import type { PageKey } from "../types/page";

const props = defineProps<{
  activePage: PageKey;
  onMinimize: () => void | Promise<void>;
  onClose: () => void | Promise<void>;
}>();

const emit = defineEmits<{ (e: "switch", page: PageKey): void }>();

const isActive = (page: PageKey) => props.activePage === page;

const switchPage = (page: PageKey) => {
  emit("switch", page);
};
</script>

<template>
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
