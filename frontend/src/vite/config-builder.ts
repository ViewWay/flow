import Vue from "@vitejs/plugin-vue";
import VueJsx from "@vitejs/plugin-vue-jsx";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import GzipPlugin from "rollup-plugin-gzip";
import Icons from "unplugin-icons/vite";
import { defineConfig, type Plugin } from "vite";
import { setupLibraryExternal } from "./library-external";

interface Options {
  base: string;
  entryFile: string;
  port: number;
  outDir: string;
  plugins?: Plugin[];
  mode: string;
}

export const sharedPlugins = [
  Vue({
    script: {
      defineModel: true,
    },
  }),
  VueJsx(),
  GzipPlugin() as Plugin,
  Icons({
    compiler: "vue3",
    customCollections: {
      core: {
        logo: () => fs.readFileSync("./src/assets/logo.svg", "utf-8"),
      },
    },
  }),
];

export function createViteConfig(options: Options) {
  const isProduction = options.mode === "production";

  const { base, entryFile, port, outDir, plugins } = options;

  const currentFileDir = path.dirname(fileURLToPath(import.meta.url));
  const rootDir = path.resolve(currentFileDir, "../..");

  return defineConfig({
    base,
    experimental: {
      enableNativePlugin: true,
    },
    plugins: [
      ...sharedPlugins,
      ...setupLibraryExternal(isProduction, base, entryFile),
      ...(plugins || []),
    ],
    resolve: {
      alias: {
        "@": path.resolve(rootDir, "src"),
        "@console": path.resolve(rootDir, "console-src"),
        "@uc": path.resolve(rootDir, "uc-src"),
      },
    },
    server: {
      port,
      fs: {
        strict: isProduction ? true : false,
      },
      proxy: {
        // 代理所有API请求到Flow后端
        "/api": {
          target: "http://localhost:8090",
          changeOrigin: true,
          secure: false,
        },
        // 代理Extension API请求
        "/apis": {
          target: "http://localhost:8090",
          changeOrigin: true,
          secure: false,
        },
        // 代理actuator端点（用于获取全局信息）
        "/actuator": {
          target: "http://localhost:8090",
          changeOrigin: true,
          secure: false,
        },
        // 代理主题静态资源
        "/themes": {
          target: "http://localhost:8090",
          changeOrigin: true,
          secure: false,
        },
      },
    },
    build: {
      outDir: path.resolve(rootDir, outDir),
      emptyOutDir: true,
      chunkSizeWarningLimit: 2048,
      rollupOptions: {
        output: {
          advancedChunks: {
            groups: [
              "es-toolkit",
              "vue-grid-layout",
              "transliteration",
              "vue-draggable-plus",
              "colorjs.io",
              "overlayscrollbars",
              "overlayscrollbars-vue",
              "floating-vue",
              "@he-tree/vue",
              "pretty-bytes",
            ].map((name) => ({
              name: "vendor",
              test: name,
            })),
          },
        },
      },
    },
  });
}
