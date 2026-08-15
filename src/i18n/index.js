import { reactive } from "vue";
import { load } from "js-yaml";

// 编译期打包：把 src/locales 下所有 yml 作为原始字符串内联进 bundle
const modules = import.meta.glob("../locales/*.yml", {
  eager: true,
  query: "?raw",
  import: "default",
});

const messages = {};
for (const [path, raw] of Object.entries(modules)) {
  const locale = path.match(/([^/]+)\.yml$/)?.[1];
  if (locale) messages[locale] = load(raw) ?? {};
}

// 跟随系统语言：中文系统 → zh-cn，其余 → en
function detectLocale() {
  const lang = (navigator.language || "en").toLowerCase();
  return lang.startsWith("zh") ? "zh-cn" : "en";
}

const state = reactive({ locale: detectLocale() });

// 简单 {name} 插值
function interpolate(template, params) {
  return template.replace(/\{(\w+)\}/g, (_, k) =>
    k in params ? String(params[k]) : `{${k}}`
  );
}

export function t(key, params = {}) {
  const dict = messages[state.locale] ?? messages["zh-cn"] ?? {};
  return interpolate(dict[key] ?? key, params);
}

export function useI18n() {
  return { t, locale: state.locale };
}
