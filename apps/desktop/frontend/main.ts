/** 文件：main.ts | 用途：前端应用启动入口，负责创建 Vue 应用并挂载到页面根节点 | 关键对象：createApp、ArcoVue、app */
// 引入 Vue 应用工厂，用于创建根应用实例。
import { createApp } from "vue";
// 引入 Arco 组件库插件，用于全局注册 Arco 组件能力。
import ArcoVue from "@arco-design/web-vue";
// 引入 Arco 全局样式，确保组件显示正常。
import "@arco-design/web-vue/dist/arco.css";
// 引入根组件，作为应用渲染入口视图。
import App from "./App.vue";
// 引入全局样式文件，覆盖应用级布局与页面样式。
import "./style.css";

// 变量：app | 含义：Vue 根应用实例 | 类型：App<Element> | 作用域：main.ts 模块
const app = createApp(App);

// 为应用安装 Arco 插件，使模板可直接使用 Arco 组件。
app.use(ArcoVue);
// 把应用挂载到 index.html 中的 #app 容器，完成前端启动。
app.mount("#app");
