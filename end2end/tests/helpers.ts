import { Page } from "@playwright/test";

/**
 * Leptos SSR HTML is interactive only after WASM hydration.
 * `networkidle` waits for the WASM/JS bundle to finish loading.
 */
export async function gotoHydrated(page: Page, path: string): Promise<void> {
  await page.goto(path, { waitUntil: "networkidle" });
  await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
}
