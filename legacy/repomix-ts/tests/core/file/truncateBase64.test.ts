import { describe, expect, it } from 'vitest';
import { truncateBase64Content } from '../../../src/core/file/truncateBase64.js';

describe('truncateBase64Content', () => {
  it('should truncate data URI base64 strings', () => {
    const input =
      'background: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==);';
    const result = truncateBase64Content(input);
    expect(result).toBe('background: url(data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB...);');
  });

  it('should handle data URIs with charset parameter', () => {
    const input =
      'src="data:image/svg+xml;charset=utf-8;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxMDAiIGhlaWdodD0iMTAwIj48cmVjdCB3aWR0aD0iMTAwIiBoZWlnaHQ9IjEwMCIgZmlsbD0iIzAwMCIvPjwvc3ZnPg=="';
    const result = truncateBase64Content(input);
    expect(result).toBe('src="data:image/svg+xml;charset=utf-8;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53..."');
  });

  it('should truncate standalone base64 strings', () => {
    const base64String =
      'VGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4gVGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4gVGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4=';
    const input = `const data = "${base64String}";`;
    const result = truncateBase64Content(input);
    expect(result).toBe('const data = "VGhlIHF1aWNrIGJyb3duIGZveCBqdW1w...";');
  });

  it('should preserve short base64 strings', () => {
    const input = 'const shortData = "SGVsbG8gV29ybGQ=";';
    const result = truncateBase64Content(input);
    expect(result).toBe(input);
  });

  it('should not truncate non-base64 strings', () => {
    const input =
      'const longString = "ABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZABCDEFGHIJKLMNOPQRSTUVWXYZ";';
    const result = truncateBase64Content(input);
    expect(result).toBe(input);
  });

  it('should handle multiple base64 occurrences in same content', () => {
    const input = `
      const img1 = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==";
      const img2 = "data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD/2wBDAAIBAQIBAQICAgICAgICAwUDAwMDAwYEBAMFBwYHBwcGBwcICQsJCAgKCAcHCg0KCgsMDAwMBwkODw0MDgsMDAz/wAALCAABAAEBAREA/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/2gAIAQEAAD8AKp//2Q==";
      const data = "VGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4gVGhlIHF1aWNrIGJyb3duIGZveCBqdW1wcyBvdmVyIHRoZSBsYXp5IGRvZy4=";
    `;
    const result = truncateBase64Content(input);
    expect(result).toContain('data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB...');
    expect(result).toContain('data:image/jpeg;base64,/9j/4AAQSkZJRgABAQEAYABgAAD/2wBD...');
    expect(result).toContain('VGhlIHF1aWNrIGJyb3duIGZveCBqdW1w...');
  });

  it('should handle base64 with whitespace around it', () => {
    const base64String =
      'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==';
    const input = `const data = \`\n  ${base64String}\n\`;`;
    const result = truncateBase64Content(input);
    expect(result).toContain('iVBORw0KGgoAAAANSUhEUgAAAAEAAAAB...');
  });

  it('should handle base64 strings with padding', () => {
    const input =
      'const paddedData = "VGhpcyBpcyBhIHRlc3Qgc3RyaW5nIHRoYXQgaXMgbG9uZyBlbm91Z2ggdG8gYmUgdHJ1bmNhdGVkIGJ5IHRoZSBmdW5jdGlvbi4gSXQgc2hvdWxkIGJlIHRydW5jYXRlZCBhZnRlciAzMiBjaGFyYWN0ZXJzLg==";';
    const result = truncateBase64Content(input);
    expect(result).toBe('const paddedData = "VGhpcyBpcyBhIHRlc3Qgc3RyaW5nIHRo...";');
  });
});
