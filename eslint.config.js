import js from "@eslint/js";
import vue from "eslint-plugin-vue";
import tseslint from "typescript-eslint";
import globals from "globals";

/**
 * 前端 lint 配置（flat config，ESM）。
 * 原则：与项目现状对齐——正确性检查为主，不引入会大面积改动的风格规则；
 * 未使用变量已由 vue-tsc 的 noUnusedLocals/noUnusedParameters 兜底，这里放行。
 */
export default tseslint.config(
  {
    ignores: [
      "dist/**",
      "node_modules/**",
      "public/**",
      "src-tauri/**",
      "website/**",
      "*.config.*",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...vue.configs["flat/recommended"],
  {
    files: ["**/*.ts", "**/*.vue"],
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        parser: tseslint.parser,
        extraFileExtensions: [".vue"],
        sourceType: "module",
      },
    },
    rules: {
      "no-unused-vars": "off",
      "@typescript-eslint/no-unused-vars": "off",
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-empty-object-type": "off",
      // 项目组件多为单名单词（GameCard / App / Icon…），且用 TS 类型化 props
      "vue/multi-word-component-names": "off",
      "vue/require-default-prop": "off",
      "vue/require-explicit-emits": "off",
      "vue/attributes-order": "off",
      "vue/component-tags-order": "off",
      "vue/max-attributes-per-line": "off",
      "vue/html-self-closing": "off",
      "vue/singleline-html-element-content-newline": "off",
      "vue/multiline-html-element-content-newline": "off",
      "vue/no-v-html": "off",
      "vue/no-mutating-props": "off",
      "vue/valid-v-for": "off", // 交给 vue-tsc 校验（v-for 带 index 时存在误报）
      // 保持项目既有排版习惯，不强制重排模板
      "vue/block-order": "off",
      "vue/html-indent": "off",
      "vue/html-closing-bracket-newline": "off",
    },
  },
);