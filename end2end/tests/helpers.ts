import { expect, Page } from "@playwright/test";

/**
 * Leptos SSR HTML is interactive only after WASM hydration.
 * `networkidle` waits for the WASM/JS bundle to finish loading.
 */
export async function gotoHydrated(page: Page, path: string): Promise<void> {
  await page.goto(path, { waitUntil: "networkidle" });
  await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
}

export async function loginAsAdmin(page: Parameters<typeof gotoHydrated>[0]) {
  await gotoHydrated(page, "/login");
  await page
    .getByTestId("login-member-select")
    .selectOption({ label: "Admin" });
  await page.getByTestId("login-submit").click();
  await expect(page.getByTestId("session-name")).toHaveText("Admin");
}

export function isArraySorted(arr: string[]): boolean {
  for (let i = 0; i < arr.length - 1; i++) {
    if (arr[i].localeCompare(arr[i + 1]) > 0) {
      return false; // Found an element that is greater than the next one
    }
  }
  return true; // All elements are in order
}
