import { defineConfig } from "vite";

export default defineConfig({
  build: {
    target: "es2022",
    modulePreload: false,
    rollupOptions: {
      input: { app: "src/main.tsx" },
      output: {
        entryFileNames: "[name].js",
        assetFileNames: "[name][extname]",
      },
    },
  },
});
