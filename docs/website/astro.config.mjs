// @ts-check
import { defineConfig } from "astro/config";
import tailwindcss from "@tailwindcss/vite";

// The site is served from https://germolinal.github.io/simple/. The book
// (/book), validation reports (/validation) and API docs (/api) are added
// next to it by `make site`.
export default defineConfig({
  site: "https://germolinal.github.io",
  base: "/simple",
  trailingSlash: "ignore",
  // Keep the whitespace between text and inline elements split across lines
  compressHTML: false,
  vite: {
    plugins: [tailwindcss()],
  },
});
