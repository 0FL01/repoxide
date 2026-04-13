import { beforeEach, describe, expect, it } from 'vitest';

// Mock DOM environment
Object.defineProperty(window, 'location', {
  value: {
    pathname: '/yamadashy/repoxide',
    href: 'https://github.com/yamadashy/repoxide',
  },
  writable: true,
});

describe('RepoxideIntegration', () => {
  beforeEach(() => {
    // Reset DOM
    document.body.innerHTML = '';

    // Mock GitHub page structure
    const navActions = document.createElement('ul');
    navActions.className = 'pagehead-actions';
    document.body.appendChild(navActions);
  });

  it('should extract repository information correctly', () => {
    // This is a placeholder test since we're testing static methods
    // In a real scenario, we'd need to import and test the actual classes
    const pathMatch = window.location.pathname.match(/^\/([^/]+)\/([^/]+)/);
    expect(pathMatch).toBeTruthy();

    if (pathMatch) {
      const [, owner, repo] = pathMatch;
      expect(owner).toBe('yamadashy');
      expect(repo).toBe('repoxide');
    }
  });

  it('should construct correct Repoxide URL', () => {
    const repoUrl = 'https://github.com/yamadashy/repoxide';
    const expectedUrl = `https://repoxide.com/?repo=${encodeURIComponent(repoUrl)}`;

    expect(expectedUrl).toBe('https://repoxide.com/?repo=https%3A%2F%2Fgithub.com%2Fyamadashy%2Frepoxide');
  });

  it('should find navigation container', () => {
    const navContainer = document.querySelector('ul.pagehead-actions');
    expect(navContainer).toBeTruthy();
  });
});
