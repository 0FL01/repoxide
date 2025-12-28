import { defineConfig } from 'vitepress';
import { configEnUs } from './config/configEnUs';
import { configRu } from './config/configRu';
import { configShard } from './config/configShard';

export default defineConfig({
  ...configShard,
  locales: {
    root: { label: 'English', ...configEnUs },
    ru: { label: 'Русский', ...configRu },
  },
  vite: {
    server: {
      allowedHosts: true,
      proxy: {
        '/api': {
          target: 'http://server:8080',
          changeOrigin: true,
        },
      },
    },
  },
});
