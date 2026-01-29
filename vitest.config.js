import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    // Run test files sequentially (not in parallel)
    // This is needed because LMDB uses a global singleton
    fileParallelism: false,

    // Run tests within a file sequentially
    sequence: {
      concurrent: false,
    },
  },
});
