import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';
import viteCompression from 'vite-plugin-compression';

const COMPRESSION_FILE_FILTER = /\.(js|mjs|json|css|svg)$/i;
const __dirname = dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  root: 'app',
  publicDir: '../public',
  plugins: [
    viteCompression({
      filter: COMPRESSION_FILE_FILTER,
      algorithm: 'gzip',
      ext: '.gz',
    }),
    viteCompression({
      filter: COMPRESSION_FILE_FILTER,
      algorithm: 'brotliCompress',
      ext: '.br',
    }),
  ],
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        _layout: resolve(__dirname, 'app', 'index.html'),
      },
    }
  }
});
