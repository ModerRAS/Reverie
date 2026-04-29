import { test as base, Page, Locator } from '@playwright/test';

export class TestContext {
  constructor(public page: Page) {}

  async login(username: string = 'admin', password: string = 'admin') {
    await this.page.goto('/login');
    await this.page.fill('[data-testid="username-input"]', username);
    await this.page.fill('[data-testid="password-input"]', password);
    await this.page.click('[data-testid="login-button"]');
    await this.page.waitForURL('**/home', { timeout: 10000 });
  }

  async navigateTo(pageName: string) {
    await this.page.click(`[data-testid="nav-item-${pageName}"]`);
  }

  async waitForPage(pageName: string) {
    await this.page.waitForSelector(`[data-testid="page-${pageName}"]`, { timeout: 10000 });
  }
}

export const test = base.extend<{ context: TestContext }>({
  context: async ({ page }, use) => {
    const ctx = new TestContext(page);
    await use(ctx);
  },
});
