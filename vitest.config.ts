import { defineConfig } from 'vitest/config';
import path from 'path';

export default defineConfig({
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
    alias: {
      '$lib': path.resolve(__dirname, './src/lib')
    }
  }
});
