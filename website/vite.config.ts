import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// 落地站独立于 Tauri 工程：端口 1422（与根 1420 / HMR 1421 错开）
export default defineConfig({
  plugins: [vue()],
  server: {
    port: 1422,
    strictPort: true,
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
