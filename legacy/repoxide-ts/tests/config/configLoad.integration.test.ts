import path from 'node:path';
import { describe, expect, test } from 'vitest';
import { loadFileConfig } from '../../src/config/configLoad.js';

describe('configLoad Integration Tests', () => {
  const jsFixturesDir = path.join(process.cwd(), 'tests/fixtures/config-js');
  const tsFixturesDir = path.join(process.cwd(), 'tests/fixtures/config-ts');

  describe('TypeScript Config Files', () => {
    test('should load .ts config with ESM default export', async () => {
      const config = await loadFileConfig(tsFixturesDir, 'repoxide.config.ts');

      expect(config).toEqual({
        output: {
          filePath: 'ts-output.xml',
          style: 'xml',
          removeComments: true,
        },
        ignore: {
          customPatterns: ['**/node_modules/**', '**/dist/**'],
        },
      });
    });

    test('should load .mts config', async () => {
      const config = await loadFileConfig(tsFixturesDir, 'repoxide.config.mts');

      expect(config).toEqual({
        output: {
          filePath: 'mts-output.xml',
          style: 'xml',
        },
        ignore: {
          customPatterns: ['**/test/**'],
        },
      });
    });

    test('should load .cts config', async () => {
      const config = await loadFileConfig(tsFixturesDir, 'repoxide.config.cts');

      expect(config).toEqual({
        output: {
          filePath: 'cts-output.xml',
          style: 'plain',
        },
        ignore: {
          customPatterns: ['**/build/**'],
        },
      });
    });

    test('should handle dynamic values in TypeScript config', async () => {
      // Mock jiti to avoid coverage instability caused by dynamic module loading
      // This ensures deterministic test results while verifying config validation
      // We don't actually load the fixture file to prevent jiti from transforming src/ files
      const config = await loadFileConfig(tsFixturesDir, 'repoxide-dynamic.config.ts', {
        jitiImport: async (fileUrl) => {
          // Verify we're loading the correct file
          expect(fileUrl).toContain('repoxide-dynamic.config.ts');

          // Return mock config simulating dynamic values
          return {
            output: {
              filePath: 'output-test-2024-01-01T00-00-00.xml',
              style: 'xml',
            },
            ignore: {
              customPatterns: ['**/node_modules/**'],
            },
          };
        },
      });

      expect(config.output?.filePath).toBe('output-test-2024-01-01T00-00-00.xml');
      expect(config.output?.style).toBe('xml');
      expect(config.ignore?.customPatterns).toEqual(['**/node_modules/**']);
    });
  });

  describe('JavaScript Config Files', () => {
    test('should load .js config with ESM default export', async () => {
      const config = await loadFileConfig(jsFixturesDir, 'repoxide.config.js');

      expect(config).toEqual({
        output: {
          filePath: 'esm-output.xml',
          style: 'xml',
          removeComments: true,
        },
        ignore: {
          customPatterns: ['**/node_modules/**', '**/dist/**'],
        },
      });
    });

    test('should load .mjs config', async () => {
      const config = await loadFileConfig(jsFixturesDir, 'repoxide.config.mjs');

      expect(config).toEqual({
        output: {
          filePath: 'mjs-output.xml',
          style: 'xml',
        },
        ignore: {
          customPatterns: ['**/test/**'],
        },
      });
    });

    test('should load .cjs config with module.exports', async () => {
      const config = await loadFileConfig(jsFixturesDir, 'repoxide.config.cjs');

      expect(config).toEqual({
        output: {
          filePath: 'cjs-output.xml',
          style: 'plain',
        },
        ignore: {
          customPatterns: ['**/build/**'],
        },
      });
    });

    test('should handle dynamic values in JS config', async () => {
      // Mock jiti to avoid coverage instability caused by dynamic module loading
      // This ensures deterministic test results while verifying config validation
      // We don't actually load the fixture file to prevent jiti from transforming src/ files
      const config = await loadFileConfig(jsFixturesDir, 'repoxide-dynamic.config.js', {
        jitiImport: async (fileUrl) => {
          // Verify we're loading the correct file
          expect(fileUrl).toContain('repoxide-dynamic.config.js');

          // Return mock config simulating dynamic values
          return {
            output: {
              filePath: 'output-2024-01-01T00-00-00.xml',
              style: 'xml',
            },
            ignore: {
              customPatterns: ['**/node_modules/**'],
            },
          };
        },
      });

      expect(config.output?.filePath).toBe('output-2024-01-01T00-00-00.xml');
      expect(config.output?.style).toBe('xml');
      expect(config.ignore?.customPatterns).toEqual(['**/node_modules/**']);
    });
  });
});
