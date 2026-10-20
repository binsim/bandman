import {test, expect } from "@playwright/test";
import { ADMIN_FILE, MEMBER_FILE, PARTICIPANT_FILE } from "./helpers";

const USER_ICON = "user-icon";

test.describe("No user logged in", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  })

  test("User icon is hidden", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeHidden();
  })
});

test.describe("Admin user logged in", () => {
  test.use({storageState: ADMIN_FILE})
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  })

  test("User icon exists", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeVisible();
  })
});

test.describe("Participant user logged in", () => {
  test.use({storageState: PARTICIPANT_FILE})
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  })

  test("User icon exists", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeVisible();
  })
});

test.describe("Member user logged in", () => {
  test.use({storageState: MEMBER_FILE})
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  })

  test("User icon exists", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeVisible();
  })
});
