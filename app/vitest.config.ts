import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import path from "path";

const isVitest = process.env.VITEST === "true";

export default defineConfig({
  plugins: [svelte()],
  resolve: {
    alias: {
      $lib: path.resolve(__dirname, "src/lib"),
    },
    ...(isVitest && {
      conditions: ["browser"],
    }),
  },
  test: {
    globals: true,
    environment: "jsdom",
    setupFiles: "./src/setupTests.ts",
    include: ["src/**/*.{test,spec}.{js,ts}"],
    environmentOptions: {
      jsdom: {
        resources: "usable",
      },
    },
  },
});
