<script setup lang="ts">
/** 文件：AliasModal.vue | 用途：为搜索条目选择别名映射的确认弹窗 | 关键对象：props, emit */
// 本行目的：引入国际化函数与语言键类型，用于多语言文案渲染。
import { t, type LocaleKey } from "../../../shared/i18n/messages";

// 变量：props | 含义：别名弹窗输入参数集合 | 类型：defineProps 返回对象 | 作用域：组件内
// 本行目的：声明组件 props 类型约束。
const props = defineProps<{
    // 本行目的：当前语言键。
    locale: LocaleKey;
    // 本行目的：控制弹窗是否可见。
    show: boolean;
    // 本行目的：待处理的追踪名称。
    pendingTrackedName: string;
    // 本行目的：别名候选是否仍在加载。
    aliasLoading: boolean;
    // 本行目的：候选别名列表。
    aliasOptions: string[];
    // 本行目的：当前已选别名列表。
    aliasSelected: string[];
    // 本行目的：取消操作回调。
    onCancel: () => void;
    // 本行目的：确认操作回调。
    onConfirm: () => void;
    // 本行目的：结束 props 类型定义。
}>();

// 变量：emit | 含义：向父组件发送可见性与选中项更新事件 | 类型：defineEmits 返回函数 | 作用域：组件内
/** 函数：emit | 输入：事件名与事件负载 | 输出：无（触发事件） | 可能失败：无（事件机制） */
// 本行目的：声明组件可触发事件及其参数类型。
const emit = defineEmits<{
    // 本行目的：定义可见性双向绑定更新事件。
    (e: "update:show", value: boolean): void;
    // 本行目的：定义别名选择结果更新事件。
    (e: "update:aliasSelected", value: string[]): void;
}>();
</script>

<template>
    <!-- 本行目的：别名选择弹窗主体。 -->
    <a-modal
        :visible="props.show"
        :title="t('search.alias.title', props.locale)"
        :width="520"
        :footer="false"
        @update:visible="(value: boolean) => emit('update:show', value)"
    >
        <!-- 本行目的：弹窗内容容器。 -->
        <div class="alias-modal-body">
            <!-- 本行目的：显示别名绑定说明标题。 -->
            <p class="alias-title">
                {{ t("search.alias.subtitle.prefix", props.locale) }}
                {{ props.pendingTrackedName || t("search.alias.subtitle.fallback", props.locale) }}
                {{ t("search.alias.subtitle.suffix", props.locale) }}
            </p>
            <!-- 本行目的：候选加载中时展示提示文案。 -->
            <p v-if="props.aliasLoading" class="alias-hint">
                {{ t("search.alias.loading", props.locale) }}
            </p>
            <!-- 本行目的：候选加载完成后展示选择区。 -->
            <template v-else>
                <!-- 本行目的：别名多选组，受控于 props.aliasSelected。 -->
                <a-checkbox-group
                    :model-value="props.aliasSelected"
                    @update:modelValue="(value: Array<string | number>) => emit('update:aliasSelected', value as string[])"
                >
                    <!-- 本行目的：候选项列表容器。 -->
                    <div class="alias-list">
                        <!-- 本行目的：遍历候选别名并渲染复选框。 -->
                        <a-checkbox v-for="name in props.aliasOptions" :key="name" :value="name" class="alias-item">
                            {{ name }}
                        </a-checkbox>
                    </div>
                </a-checkbox-group>
            </template>
            <!-- 本行目的：底部操作按钮区。 -->
            <div class="alias-actions">
                <!-- 本行目的：取消按钮，执行取消回调。 -->
                <a-button size="small" @click="props.onCancel">
                    {{ t("search.alias.cancel", props.locale) }}
                </a-button>
                <!-- 本行目的：确认按钮，加载中或未选择时禁用。 -->
                <a-button size="small" type="primary" :disabled="props.aliasLoading || !props.aliasSelected.length" @click="props.onConfirm">
                    {{ t("search.alias.confirm", props.locale) }}
                </a-button>
            </div>
        </div>
    </a-modal>
</template>
