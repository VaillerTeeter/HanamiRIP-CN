<script setup lang="ts">
/** 文件：AsyncState.vue | 用途：统一渲染异步四态（loading/empty/error/ready）容器 | 关键对象：Props, props, emit, handleRetry */
// 本行目的：引入异步阶段类型定义。
import type { AsyncPhase } from "../types/async-state";

/** 类型：Props | 用途：定义异步状态组件输入参数 | 字段：phase 与各状态文案配置 */
// 本行目的：声明组件 props 类型。
interface Props {
    // 本行目的：声明当前异步阶段。
    phase: AsyncPhase;
    // 本行目的：声明加载态文案（可选）。
    loadingText?: string;
    // 本行目的：声明空态文案（可选）。
    emptyText?: string;
    // 本行目的：声明错误标题（可选）。
    errorTitle?: string;
    // 本行目的：声明错误描述文案（可选）。
    errorText?: string;
    // 本行目的：声明是否显示重试按钮（可选）。
    showRetry?: boolean;
}

// 变量：props | 含义：带默认值的组件输入参数对象 | 类型：Props（withDefaults 后） | 作用域：AsyncState 组件内
/** 函数：withDefaults | 输入：defineProps<Props>() 与默认值对象 | 输出：带默认值的 props | 可能失败：无 */
// 本行目的：为可选 props 提供默认文案与行为。
const props = withDefaults(defineProps<Props>(), {
    loadingText: "加载中…",
    emptyText: "暂无数据",
    errorTitle: "加载失败",
    errorText: "请稍后重试",
    showRetry: true,
});

// 变量：emit | 含义：组件事件派发函数 | 类型：由 defineEmits 推断 | 作用域：AsyncState 组件内
/** 函数：defineEmits | 输入：retry 事件 | 输出：emit 函数 | 可能失败：无 */
// 本行目的：声明重试事件。
const emit = defineEmits<{
    (event: "retry"): void;
}>();

// 变量：handleRetry | 含义：触发 retry 事件的处理函数 | 类型：() => void | 作用域：AsyncState 组件内
/** 函数：handleRetry | 输入：无 | 输出：无（触发 retry） | 可能失败：无 */
// 本行目的：封装重试按钮点击逻辑。
const handleRetry = () => {
    emit("retry");
};
</script>

<template>
    <!-- 本行目的：异步状态容器，data-phase 用于样式区分。 -->
    <div class="async-state" :data-phase="props.phase">
        <!-- 本行目的：loading 阶段展示加载动画与文案。 -->
        <template v-if="props.phase === 'loading'">
            <a-space direction="vertical" size="16" align="center">
                <a-spin :size="32" />
                <a-typography-text type="secondary">{{ props.loadingText }}</a-typography-text>
            </a-space>
        </template>
        <!-- 本行目的：empty 阶段展示空状态文案。 -->
        <template v-else-if="props.phase === 'empty'">
            <a-empty :description="props.emptyText" />
        </template>
        <!-- 本行目的：error 阶段展示错误结果并可重试。 -->
        <template v-else-if="props.phase === 'error'">
            <a-result status="error" :title="props.errorTitle" :subtitle="props.errorText">
                <template #extra>
                    <!-- 本行目的：根据配置决定是否显示重试按钮。 -->
                    <a-button v-if="props.showRetry" type="primary" @click="handleRetry"> 重试 </a-button>
                </template>
            </a-result>
        </template>
        <!-- 本行目的：ready/idle 等其他阶段透传默认插槽内容。 -->
        <template v-else>
            <slot />
        </template>
    </div>
</template>

<style scoped>
/* 本行目的：异步状态容器采用居中布局并设置最小高度。 */
.async-state {
    /* 本行目的：启用弹性布局。 */
    display: flex;
    /* 本行目的：垂直方向居中。 */
    align-items: center;
    /* 本行目的：水平方向居中。 */
    justify-content: center;
    /* 本行目的：保证状态区域最小可视高度。 */
    min-height: 160px;
    /* 本行目的：设置容器内边距。 */
    padding: 16px;
}
</style>
