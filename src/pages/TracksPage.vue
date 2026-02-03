<script setup lang="ts">
import { NButton, NCard, NCheckbox, NProgress, NSelect } from "naive-ui";
import type { UseTracksPageReturn } from "../composables/useTracksPage";

const props = defineProps<{
  tracks: UseTracksPageReturn;
}>();
</script>

<template>
  <div class="app-body download-view">
    <NCard title="轨道工坊" size="small" class="download-card">
      <div class="tracks-mix-bar">
        <NButton type="primary" size="small" :loading="tracks.trackMixLoading.value" @click="tracks.enqueueMixTask">
          添加到混流任务队列
        </NButton>
        <span v-if="tracks.trackMixResult.value" class="tracks-mix-success">{{ tracks.trackMixResult.value }}</span>
        <span v-if="tracks.trackMixError.value" class="tracks-mix-error">{{ tracks.trackMixError.value }}</span>
      </div>
      <div class="tracks-mix-lang">
        <span class="tracks-mix-lang-label">默认语言</span>
        <div class="tracks-mix-lang-item">
          <span>视频</span>
          <NSelect
            v-model:value="tracks.trackLangDefaults.video"
            size="small"
            :options="tracks.trackLanguageOptions"
            filterable
            tag
            placeholder="语言"
          />
        </div>
        <div class="tracks-mix-lang-item">
          <span>音频</span>
          <NSelect
            v-model:value="tracks.trackLangDefaults.audio"
            size="small"
            :options="tracks.trackLanguageOptions"
            filterable
            tag
            placeholder="语言"
          />
        </div>
        <div class="tracks-mix-lang-item">
          <span>字幕</span>
          <NSelect
            v-model:value="tracks.trackLangDefaults.subtitle"
            size="small"
            :options="tracks.trackLanguageOptions"
            filterable
            tag
            placeholder="语言"
          />
        </div>
      </div>
      <div class="tracks-list">
        <div class="tracks-section">
          <div class="tracks-header">
            <span class="tracks-title">视频</span>
            <div class="tracks-actions">
              <NButton size="small" secondary @click="tracks.addTrackFile('video')">添加文件</NButton>
              <NButton
                size="small"
                type="primary"
                :loading="tracks.trackLoading.value.video"
                :disabled="!tracks.trackFiles.value.video.length"
                @click="tracks.detectTracks('video')"
              >
                检测
              </NButton>
            </div>
          </div>
          <div class="tracks-body">
            <div class="tracks-files" v-if="tracks.trackFiles.value.video.length">
              <div v-for="file in tracks.trackFiles.value.video" :key="file.id" class="tracks-file">
                <span class="tracks-file-name">{{ file.name }}</span>
                <span class="tracks-file-size">{{ file.fileSize || '-' }}</span>
                <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
              </div>
            </div>
            <p v-else class="download-empty">尚未添加视频文件。</p>
            <div v-if="tracks.trackErrors.value.video" class="tracks-error">{{ tracks.trackErrors.value.video }}</div>
            <div v-if="tracks.trackLoading.value.video" class="tracks-progress">
              <NProgress type="line" :percentage="tracks.trackProgress.value.video" :show-indicator="true" :height="8" />
            </div>
            <div v-if="tracks.trackInfos.value.video.length" class="tracks-info">
              <div v-for="group in tracks.trackInfos.value.video" :key="group.file.id" class="tracks-info-group">
                <div class="tracks-info-file">{{ group.file.name }}</div>
                <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                  <NCheckbox v-model:checked="info.selected" size="small" />
                  <span class="tracks-info-name">轨道 {{ info.trackId || '-' }}</span>
                  <span class="tracks-info-meta">编码 {{ info.codec || '-' }}</span>
                  <span class="tracks-info-meta">语言 {{ info.lang || '-' }}</span>
                  <span class="tracks-info-meta">语言名称 {{ info.languageName || '-' }}</span>
                  <span class="tracks-info-meta">名称 {{ info.trackName || '-' }}</span>
                  <span class="tracks-info-meta">
                    默认 {{ info.isDefault === true ? '是' : info.isDefault === false ? '否' : '-' }}
                  </span>
                  <span class="tracks-info-meta">
                    强制 {{ info.isForced === true ? '是' : info.isForced === false ? '否' : '-' }}
                  </span>
                  <span class="tracks-info-meta">字符集 {{ info.charset || '-' }}</span>
                  <span class="tracks-info-meta">属性 {{ info.attributes || '-' }}</span>
                  <span class="tracks-info-meta">容器 {{ info.container || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tracks-section">
          <div class="tracks-header">
            <span class="tracks-title">音频</span>
            <div class="tracks-actions">
              <NButton size="small" secondary @click="tracks.addTrackFile('audio')">添加文件</NButton>
              <NButton
                size="small"
                type="primary"
                :loading="tracks.trackLoading.value.audio"
                :disabled="!tracks.trackFiles.value.audio.length"
                @click="tracks.detectTracks('audio')"
              >
                检测
              </NButton>
            </div>
          </div>
          <div class="tracks-body">
            <div class="tracks-files" v-if="tracks.trackFiles.value.audio.length">
              <div v-for="file in tracks.trackFiles.value.audio" :key="file.id" class="tracks-file">
                <span class="tracks-file-name">{{ file.name }}</span>
                <span class="tracks-file-size">{{ file.fileSize || '-' }}</span>
                <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
              </div>
            </div>
            <p v-else class="download-empty">尚未添加音频文件。</p>
            <div v-if="tracks.trackErrors.value.audio" class="tracks-error">{{ tracks.trackErrors.value.audio }}</div>
            <div v-if="tracks.trackLoading.value.audio" class="tracks-progress">
              <NProgress type="line" :percentage="tracks.trackProgress.value.audio" :show-indicator="true" :height="8" />
            </div>
            <div v-if="tracks.trackInfos.value.audio.length" class="tracks-info">
              <div v-for="group in tracks.trackInfos.value.audio" :key="group.file.id" class="tracks-info-group">
                <div class="tracks-info-file">{{ group.file.name }}</div>
                <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                  <NCheckbox v-model:checked="info.selected" size="small" />
                  <span class="tracks-info-name">轨道 {{ info.trackId || '-' }}</span>
                  <span class="tracks-info-meta">编码 {{ info.codec || '-' }}</span>
                  <span class="tracks-info-meta">语言 {{ info.lang || '-' }}</span>
                  <span class="tracks-info-meta">语言名称 {{ info.languageName || '-' }}</span>
                  <span class="tracks-info-meta">名称 {{ info.trackName || '-' }}</span>
                  <span class="tracks-info-meta">
                    默认 {{ info.isDefault === true ? '是' : info.isDefault === false ? '否' : '-' }}
                  </span>
                  <span class="tracks-info-meta">
                    强制 {{ info.isForced === true ? '是' : info.isForced === false ? '否' : '-' }}
                  </span>
                  <span class="tracks-info-meta">字符集 {{ info.charset || '-' }}</span>
                  <span class="tracks-info-meta">属性 {{ info.attributes || '-' }}</span>
                  <span class="tracks-info-meta">容器 {{ info.container || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tracks-section">
          <div class="tracks-header">
            <span class="tracks-title">字幕</span>
            <div class="tracks-actions">
              <NButton size="small" secondary @click="tracks.addTrackFile('subtitle')">添加文件</NButton>
              <NButton
                size="small"
                type="primary"
                :loading="tracks.trackLoading.value.subtitle"
                :disabled="!tracks.trackFiles.value.subtitle.length"
                @click="tracks.detectTracks('subtitle')"
              >
                检测
              </NButton>
            </div>
          </div>
          <div class="tracks-body">
            <div class="tracks-files" v-if="tracks.trackFiles.value.subtitle.length">
              <div v-for="file in tracks.trackFiles.value.subtitle" :key="file.id" class="tracks-file">
                <span class="tracks-file-name">{{ file.name }}</span>
                <span class="tracks-file-size">{{ file.fileSize || '-' }}</span>
                <span class="tracks-file-path" :title="file.path">{{ file.path }}</span>
              </div>
            </div>
            <p v-else class="download-empty">尚未添加字幕文件。</p>
            <div v-if="tracks.trackErrors.value.subtitle" class="tracks-error">{{ tracks.trackErrors.value.subtitle }}</div>
            <div v-if="tracks.trackLoading.value.subtitle" class="tracks-progress">
              <NProgress type="line" :percentage="tracks.trackProgress.value.subtitle" :show-indicator="true" :height="8" />
            </div>
            <div v-if="tracks.trackInfos.value.subtitle.length" class="tracks-info">
              <div v-for="group in tracks.trackInfos.value.subtitle" :key="group.file.id" class="tracks-info-group">
                <div class="tracks-info-file">{{ group.file.name }}</div>
                <div v-for="info in group.tracks" :key="info.trackId" class="tracks-info-row">
                  <NCheckbox v-model:checked="info.selected" size="small" />
                  <span class="tracks-info-name">轨道 {{ info.trackId || '-' }}</span>
                  <span class="tracks-info-meta">编码 {{ info.codec || '-' }}</span>
                  <span class="tracks-info-meta">语言 {{ info.lang || '-' }}</span>
                  <span class="tracks-info-meta">语言名称 {{ info.languageName || '-' }}</span>
                  <span class="tracks-info-meta">名称 {{ info.trackName || '-' }}</span>
                  <span class="tracks-info-meta">
                    默认 {{ info.isDefault === true ? '是' : info.isDefault === false ? '否' : '-' }}
                  </span>
                  <span class="tracks-info-meta">
                    强制 {{ info.isForced === true ? '是' : info.isForced === false ? '否' : '-' }}
                  </span>
                  <span class="tracks-info-meta">字符集 {{ info.charset || '-' }}</span>
                  <span class="tracks-info-meta">属性 {{ info.attributes || '-' }}</span>
                  <span class="tracks-info-meta">容器 {{ info.container || '-' }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div class="tracks-section">
          <div class="tracks-header">
            <span class="tracks-title">混流任务队列</span>
            <div class="tracks-actions">
              <NButton size="small" secondary :disabled="tracks.mixQueueRunning.value" @click="tracks.startMixQueue">
                开始任务队列
              </NButton>
              <NButton
                size="small"
                secondary
                :disabled="tracks.mixQueueRunning.value || !tracks.mixQueue.value.length"
                @click="tracks.clearMixQueue"
              >
                清除所有任务
              </NButton>
            </div>
          </div>
          <div class="tracks-body">
            <p v-if="!tracks.mixQueue.value.length" class="download-empty">暂无混流任务。</p>
            <div v-else class="mix-queue-list">
              <div v-for="item in tracks.mixQueue.value" :key="item.id" class="mix-queue-row" @click.stop="tracks.openMixTaskDetail(item)">
                <span class="mix-queue-id">#{{ item.id }}</span>
                <span class="mix-queue-time">{{ item.createdAt }}</span>
                <span class="mix-queue-output" :title="item.outputPath">{{ item.outputPath }}</span>
                <span class="mix-queue-status" :data-status="item.status">
                  {{
                    item.status === 'queued'
                      ? '排队中'
                      : item.status === 'running'
                        ? '处理中'
                        : item.status === 'success'
                          ? '完成'
                          : '失败'
                  }}
                </span>
                <div v-if="item.message" class="mix-queue-message-row" :title="item.message">
                  {{ item.message }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </NCard>
  </div>
</template>
