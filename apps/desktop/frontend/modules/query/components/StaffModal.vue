<script setup lang="ts">
/** 文件：StaffModal.vue | 用途：展示条目制作人员分组信息的弹窗组件 | 关键对象：props, emit */
// 本行目的：引入人员分组类型，约束 staff 展示数据结构。
import type { StaffGroup } from "../../tracking/types/anime";
// 本行目的：引入番剧条目类型，用于 selected 的类型约束。
import type { MonthAnime } from "../../tracking/types/anime";
// 本行目的：引入国际化函数与语言键类型，用于多语言文案渲染。
import { t, type LocaleKey } from "../../../shared/i18n/messages";

// 变量：props | 含义：弹窗输入参数集合，控制显示状态与数据内容 | 类型：组件 props 对象 | 作用域：组件内
// 本行目的：声明组件 props 的类型结构。
const props = defineProps<{
    // 本行目的：控制弹窗是否显示。
    show: boolean;
    // 本行目的：当前语言环境键。
    locale: LocaleKey;
    // 本行目的：当前选中的番剧条目，可为空。
    selected: MonthAnime | null;
    // 本行目的：当前正在加载 staff 的条目 ID，可为空。
    staffLoadingId: number | null;
    // 本行目的：staff 请求失败时的错误信息。
    staffError: string;
    // 本行目的：当前条目的人员分组列表。
    selectedStaff: StaffGroup[];
    // 本行目的：打开外部链接的方法（可同步或异步）。
    openExternalLink: (url?: string | null) => void | Promise<void>;
    // 本行目的：结束 props 类型定义。
}>();

// 变量：emit | 含义：组件事件发送器，用于通知父组件更新显示状态 | 类型：defineEmits 返回函数 | 作用域：组件内
/** 函数：emit | 输入：事件名与事件负载 | 输出：无（触发事件） | 可能失败：无（事件机制） */
// 本行目的：声明组件可发出的事件类型。
const emit = defineEmits<{ (e: "update:show", value: boolean): void }>();
</script>

<template>
    <!-- 本行目的：人员信息弹窗主体，显示与隐藏由 props.show 控制。 -->
    <a-modal
        :visible="props.show"
        :title="t('staff.title', props.locale)"
        :width="720"
        :body-style="{ maxHeight: '70vh', overflow: 'auto' }"
        :footer="false"
        @update:visible="(value: boolean) => emit('update:show', value)"
    >
        <!-- 本行目的：弹窗内容容器。 -->
        <div class="staff-modal-body">
            <!-- 本行目的：展示当前条目名称，优先中文名，兜底原名。 -->
            <a-typography-text class="staff-modal-title">
                {{ t("staff.current", props.locale) }}：{{ props.selected?.nameCn || props.selected?.name || "" }}
            </a-typography-text>
            <!-- 本行目的：当当前条目 staff 仍在加载时展示加载提示。 -->
            <a-typography-text v-if="props.staffLoadingId === props.selected?.id" type="secondary">
                {{ t("staff.loading", props.locale) }}
            </a-typography-text>
            <!-- 本行目的：加载失败时展示错误提示。 -->
            <a-typography-text v-else-if="props.staffError" type="danger">
                {{ props.staffError }}
            </a-typography-text>
            <!-- 本行目的：加载完成且无错误时进入正常展示分支。 -->
            <div v-else>
                <!-- 本行目的：无 staff 数据时展示空状态文案。 -->
                <a-typography-text v-if="!props.selectedStaff.length" type="secondary">
                    {{ t("staff.empty", props.locale) }}
                </a-typography-text>
                <!-- 本行目的：有 staff 数据时展示分组列表。 -->
                <div v-else class="staff-group-list">
                    <!-- 本行目的：按角色分组遍历人员信息。 -->
                    <div v-for="group in props.selectedStaff" :key="group.role" class="staff-group">
                        <!-- 本行目的：显示角色名称（例如导演、脚本等）。 -->
                        <div class="staff-role">{{ group.role }}</div>
                        <!-- 本行目的：当前角色下人员列表容器。 -->
                        <div class="staff-people">
                            <!-- 本行目的：遍历当前角色下每位人员，渲染为可交互链接文本。 -->
                            <span
                                v-for="person in group.people"
                                :key="person.id"
                                class="staff-link staff-link-text"
                                role="link"
                                tabindex="0"
                                style="color: #16a34a !important; -webkit-text-fill-color: #16a34a !important"
                                @click="props.openExternalLink(person.url)"
                                @keydown.enter.prevent="props.openExternalLink(person.url)"
                            >
                                <!-- 本行目的：显示人员名称。 -->
                                {{ person.name }}
                            </span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </a-modal>
</template>
