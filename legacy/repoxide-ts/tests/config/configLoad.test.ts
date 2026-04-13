import type { Stats } from 'node:fs';
import * as fs from 'node:fs/promises';
import path from 'node:path';
import process from 'node:process';
import { beforeEach, describe, expect, test, vi } from 'vitest';
import { loadFileConfig, mergeConfigs } from '../../src/config/configLoad.js';
import { defaultConfig, type RepoxideConfigCli, type RepoxideConfigFile } from '../../src/config/configSchema.js';
import { getGlobalDirectory } from '../../src/config/globalDirectory.js';
import { RepoxideConfigValidationError } from '../../src/shared/errorHandle.js';
import { logger } from '../../src/shared/logger.js';

vi.mock('node:fs/promises');
vi.mock('../../src/shared/logger', () => ({
  logger: {
    trace: vi.fn(),
    note: vi.fn(),
    log: vi.fn(),
  },
}));
vi.mock('../../src/config/globalDirectory', () => ({
  getGlobalDirectory: vi.fn(),
}));

describe('configLoad', () => {
  beforeEach(() => {
    vi.resetAllMocks();
    process.env = {};
  });

  describe('loadFileConfig', () => {
    test('should load and parse a valid local config file', async () => {
      const mockConfig = {
        output: { filePath: 'test-output.txt' },
        ignore: { useDefaultPatterns: true },
      };
      vi.mocked(fs.readFile).mockResolvedValue(JSON.stringify(mockConfig));
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      const result = await loadFileConfig(process.cwd(), 'test-config.json');
      expect(result).toEqual(mockConfig);
    });

    test('should throw RepoxideConfigValidationError for invalid config', async () => {
      const invalidConfig = {
        output: { filePath: 123, style: 'invalid' }, // Invalid filePath type and invalid style
        ignore: { useDefaultPatterns: 'not a boolean' }, // Invalid type
      };
      vi.mocked(fs.readFile).mockResolvedValue(JSON.stringify(invalidConfig));
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      await expect(loadFileConfig(process.cwd(), 'test-config.json')).rejects.toThrow(RepoxideConfigValidationError);
    });

    test('should load global config when local config is not found', async () => {
      const mockGlobalConfig = {
        output: { filePath: 'global-output.txt' },
        ignore: { useDefaultPatterns: false },
      };
      vi.mocked(getGlobalDirectory).mockReturnValue('/global/repoxide');
      vi.mocked(fs.stat)
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.ts
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.mts
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.cts
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.js
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.mjs
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.cjs
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.json5
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.jsonc
        .mockRejectedValueOnce(new Error('File not found')) // Local repoxide.config.json
        .mockRejectedValueOnce(new Error('File not found')) // Global repoxide.config.ts
        .mockRejectedValueOnce(new Error('File not found')) // Global repoxide.config.mts
        .mockRejectedValueOnce(new Error('File not found')) // Global repoxide.config.cts
        .mockRejectedValueOnce(new Error('File not found')) // Global repoxide.config.js
        .mockRejectedValueOnce(new Error('File not found')) // Global repoxide.config.mjs
        .mockRejectedValueOnce(new Error('File not found')) // Global repoxide.config.cjs
        .mockResolvedValueOnce({ isFile: () => true } as Stats); // Global repoxide.config.json5
      vi.mocked(fs.readFile).mockResolvedValue(JSON.stringify(mockGlobalConfig));

      const result = await loadFileConfig(process.cwd(), null);
      expect(result).toEqual(mockGlobalConfig);
      expect(fs.readFile).toHaveBeenCalledWith(path.join('/global/repoxide', 'repoxide.config.json5'), 'utf-8');
    });

    test('should return an empty object if no config file is found', async () => {
      const loggerSpy = vi.spyOn(logger, 'log').mockImplementation(vi.fn());
      vi.mocked(getGlobalDirectory).mockReturnValue('/global/repoxide');
      vi.mocked(fs.stat).mockRejectedValue(new Error('File not found'));

      const result = await loadFileConfig(process.cwd(), null);
      expect(result).toEqual({});

      expect(loggerSpy).toHaveBeenCalledWith(expect.stringContaining('No custom config found'));
      expect(loggerSpy).toHaveBeenCalledWith(expect.stringContaining('repoxide.config.json5'));
      expect(loggerSpy).toHaveBeenCalledWith(expect.stringContaining('repoxide.config.jsonc'));
      expect(loggerSpy).toHaveBeenCalledWith(expect.stringContaining('repoxide.config.json'));
    });

    test('should throw an error for invalid JSON', async () => {
      vi.mocked(fs.readFile).mockResolvedValue('invalid json');
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      await expect(loadFileConfig(process.cwd(), 'test-config.json')).rejects.toThrow('Invalid syntax');
    });

    test('should parse config file with comments', async () => {
      const configWithComments = `{
        // Output configuration
        "output": {
          "filePath": "test-output.txt"
        },
        /* Ignore configuration */
        "ignore": {
          "useGitignore": true // Use .gitignore file
        }
      }`;

      vi.mocked(fs.readFile).mockResolvedValue(configWithComments);
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      const result = await loadFileConfig(process.cwd(), 'test-config.json');
      expect(result).toEqual({
        output: { filePath: 'test-output.txt' },
        ignore: { useGitignore: true },
      });
    });

    test('should parse config file with JSON5 features', async () => {
      const configWithJSON5Features = `{
        // Output configuration
        output: {
          filePath: 'test-output.txt',
          style: 'plain',
        },
        /* Ignore configuration */
        ignore: {
          useGitignore: true, // Use .gitignore file
          customPatterns: [
            '*.log',
            '*.tmp',
            '*.temp', // Trailing comma
          ],
        },
      }`;

      vi.mocked(fs.readFile).mockResolvedValue(configWithJSON5Features);
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      const result = await loadFileConfig(process.cwd(), 'test-config.json');
      expect(result).toEqual({
        output: { filePath: 'test-output.txt', style: 'plain' },
        ignore: {
          useGitignore: true,
          customPatterns: ['*.log', '*.tmp', '*.temp'],
        },
      });
    });

    test('should load .jsonc config file with priority order', async () => {
      const mockConfig = {
        output: { filePath: 'jsonc-output.txt' },
        ignore: { useDefaultPatterns: true },
      };
      vi.mocked(fs.stat)
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.ts
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.mts
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.cts
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.js
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.mjs
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.cjs
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.json5
        .mockResolvedValueOnce({ isFile: () => true } as Stats); // repoxide.config.jsonc
      vi.mocked(fs.readFile).mockResolvedValue(JSON.stringify(mockConfig));

      const result = await loadFileConfig(process.cwd(), null);
      expect(result).toEqual(mockConfig);
      expect(fs.readFile).toHaveBeenCalledWith(path.resolve(process.cwd(), 'repoxide.config.jsonc'), 'utf-8');
    });

    test('should prioritize .json5 over .jsonc and .json', async () => {
      const mockConfig = {
        output: { filePath: 'json5-output.txt' },
        ignore: { useDefaultPatterns: true },
      };
      vi.mocked(fs.stat)
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.ts
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.mts
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.cts
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.js
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.mjs
        .mockRejectedValueOnce(new Error('File not found')) // repoxide.config.cjs
        .mockResolvedValueOnce({ isFile: () => true } as Stats); // repoxide.config.json5 exists
      vi.mocked(fs.readFile).mockResolvedValue(JSON.stringify(mockConfig));

      const result = await loadFileConfig(process.cwd(), null);
      expect(result).toEqual(mockConfig);
      expect(fs.readFile).toHaveBeenCalledWith(path.resolve(process.cwd(), 'repoxide.config.json5'), 'utf-8');
      // Should not check for .jsonc or .json since .json5 was found
      expect(fs.stat).toHaveBeenCalledTimes(7);
    });

    test('should throw RepoxideError when specific config file does not exist', async () => {
      const nonExistentConfigPath = 'non-existent-config.json';
      vi.mocked(fs.stat).mockRejectedValue(new Error('File not found'));

      await expect(loadFileConfig(process.cwd(), nonExistentConfigPath)).rejects.toThrow(
        `Config file not found at ${nonExistentConfigPath}`,
      );
    });

    test('should throw RepoxideError for unsupported config file format', async () => {
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      await expect(loadFileConfig(process.cwd(), 'test-config.yaml')).rejects.toThrow('Unsupported config file format');
    });

    test('should throw RepoxideError for config file with unsupported extension', async () => {
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);

      await expect(loadFileConfig(process.cwd(), 'test-config.toml')).rejects.toThrow('Unsupported config file format');
    });

    test('should handle general errors when loading config', async () => {
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);
      vi.mocked(fs.readFile).mockRejectedValue(new Error('Permission denied'));

      await expect(loadFileConfig(process.cwd(), 'test-config.json')).rejects.toThrow('Error loading config');
    });

    test('should handle non-Error objects when loading config', async () => {
      vi.mocked(fs.stat).mockResolvedValue({ isFile: () => true } as Stats);
      vi.mocked(fs.readFile).mockRejectedValue('String error');

      await expect(loadFileConfig(process.cwd(), 'test-config.json')).rejects.toThrow('Error loading config');
    });
  });

  describe('mergeConfigs', () => {
    test('should correctly merge configs', () => {
      const fileConfig: RepoxideConfigFile = {
        output: { filePath: 'file-output.txt' },
        ignore: { useDefaultPatterns: true, customPatterns: ['file-ignore'] },
      };
      const cliConfig: RepoxideConfigCli = {
        output: { filePath: 'cli-output.txt' },
        ignore: { customPatterns: ['cli-ignore'] },
      };

      const result = mergeConfigs(process.cwd(), fileConfig, cliConfig);

      expect(result.output.filePath).toBe('cli-output.txt');
      expect(result.ignore.useDefaultPatterns).toBe(true);
      expect(result.ignore.customPatterns).toContain('file-ignore');
      expect(result.ignore.customPatterns).toContain('cli-ignore');
    });

    test('should throw RepoxideConfigValidationError for invalid merged config', () => {
      const fileConfig: RepoxideConfigFile = {
        output: { filePath: 'file-output.txt', style: 'plain' },
      };
      const cliConfig: RepoxideConfigCli = {
        // @ts-expect-error
        output: { style: 'invalid' }, // Invalid style
      };

      expect(() => mergeConfigs(process.cwd(), fileConfig, cliConfig)).toThrow(RepoxideConfigValidationError);
    });

    test('should merge nested git config correctly', () => {
      const fileConfig: RepoxideConfigFile = {
        output: { git: { sortByChanges: false } },
      };
      const cliConfig: RepoxideConfigCli = {
        output: { git: { includeDiffs: true } },
      };
      const merged = mergeConfigs(process.cwd(), fileConfig, cliConfig);

      // Both configs should be applied
      expect(merged.output.git.sortByChanges).toBe(false);
      expect(merged.output.git.includeDiffs).toBe(true);
      // Defaults should still be present
      expect(merged.output.git.sortByChangesMaxCommits).toBe(100);
    });

    test('should not mutate defaultConfig', () => {
      const originalFilePath = defaultConfig.output.filePath;
      const fileConfig: RepoxideConfigFile = {
        output: { style: 'markdown' },
      };

      mergeConfigs(process.cwd(), fileConfig, {});

      // defaultConfig should remain unchanged
      expect(defaultConfig.output.filePath).toBe(originalFilePath);
    });

    test('should merge tokenCount config correctly', () => {
      const fileConfig: RepoxideConfigFile = {
        tokenCount: { encoding: 'cl100k_base' },
      };
      const merged = mergeConfigs(process.cwd(), fileConfig, {});

      expect(merged.tokenCount.encoding).toBe('cl100k_base');
    });

    test('should map default filename to style when only style is provided via CLI', () => {
      const merged = mergeConfigs(process.cwd(), {}, { output: { style: 'markdown' } });
      expect(merged.output.filePath).toBe('repoxide-output.md');
      expect(merged.output.style).toBe('markdown');
    });

    test('should keep explicit CLI output filePath even when style is provided', () => {
      const merged = mergeConfigs(process.cwd(), {}, { output: { style: 'markdown', filePath: 'custom-output.any' } });
      expect(merged.output.filePath).toBe('custom-output.any');
      expect(merged.output.style).toBe('markdown');
    });

    test('should keep explicit file config filePath even when style is provided via CLI', () => {
      const merged = mergeConfigs(
        process.cwd(),
        { output: { filePath: 'from-file.txt' } },
        { output: { style: 'markdown' } },
      );
      expect(merged.output.filePath).toBe('from-file.txt');
      expect(merged.output.style).toBe('markdown');
    });

    test('should map default filename when style provided in file config and no filePath anywhere', () => {
      const merged = mergeConfigs(process.cwd(), { output: { style: 'plain' } }, {});
      expect(merged.output.filePath).toBe('repoxide-output.txt');
      expect(merged.output.style).toBe('plain');
    });

    test('should merge skillGenerate boolean from CLI config', () => {
      const merged = mergeConfigs(process.cwd(), {}, { skillGenerate: true });
      expect(merged.skillGenerate).toBe(true);
    });

    test('should merge skillGenerate string from CLI config', () => {
      const merged = mergeConfigs(process.cwd(), {}, { skillGenerate: 'my-custom-skill' });
      expect(merged.skillGenerate).toBe('my-custom-skill');
    });

    test('should not include skillGenerate in merged config when undefined', () => {
      const merged = mergeConfigs(process.cwd(), {}, {});
      expect(merged.skillGenerate).toBeUndefined();
    });

    test('should not allow skillGenerate from file config (CLI-only option)', () => {
      // File config should not have skillGenerate - it's CLI-only
      // This test verifies that even if somehow passed, file config doesn't affect it
      const merged = mergeConfigs(process.cwd(), {}, { skillGenerate: 'from-cli' });
      expect(merged.skillGenerate).toBe('from-cli');
    });
  });
});
