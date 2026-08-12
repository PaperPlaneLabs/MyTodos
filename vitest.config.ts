import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import path from 'path';

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'happy-dom',
    globals: true,
    include: ['src/**/*.{test,spec}.{js,ts}'],
    coverage: {
      reporter: ['text', 'json', 'html'],
      include: ['src/lib/**/*.{js,ts,svelte}'],
      exclude: [
        'src/lib/**/*.{test,spec}.{js,ts}',
        'src/lib/test-utils/**',
        '**/*.d.ts',
        '**/*.config.{js,ts}'
      ]
    }
  },
  resolve: {
    conditions: ['browser'],
    alias: {
      '$lib': path.resolve(__dirname, './src/lib')
    }
  }
});
