import { expect, test } from "@playwright/test";
import { ADMIN_FILE, MEMBER_FILE, PARTICIPANT_FILE } from "./helpers";
import { exportBuffer } from "node:ffi";

const USER_ICON = "user-icon";
const LOGIN_BUTTON = "login-btn";
const TOPBAR_NAVIGATION_CONTAINER = "topbar-navigation-container";

test.describe("No user logged in", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/login", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("Titlebar shows reduced information", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeHidden();
    // TODO:
    // await expect(page.getByTestId(LOGIN_BUTTON)).toBeHidden();
    await expect(page.getByTestId(TOPBAR_NAVIGATION_CONTAINER)).toBeHidden();
  });

  test("Login form gets displayed in the center", async ({ page }) => {
    await expect(page.getByTestId("login-form")).toBeVisible();
    const select = page.getByTestId("login-member-select");
    await expect(select).toBeVisible();
  });

  test("requires choosing a member", async ({ page }) => {
    await page.getByTestId("login-submit").click();
    await expect(page.getByTestId("login-error")).toBeVisible();
  });

  test("Logging in changes page", async ({ page }) => {
    const select = page.getByTestId("login-member-select");
    await select.selectOption({ label: "Admin" });
    await page.getByTestId("login-submit").click();

    await expect(page).toHaveURL(/\/$/);
  });
});

test.describe("Admin logged in", () => {
  test.use({ storageState: ADMIN_FILE });
  test.beforeEach(async ({ page }) => {
    await page.goto("/login", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  // TODO: Need to redirect
});
test.describe("Participant logged in", () => {
  test.use({ storageState: PARTICIPANT_FILE });
  test.beforeEach(async ({ page }) => {
    await page.goto("/login", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  // TODO: Need to redirect
});
test.describe("Member logged in", () => {
  test.use({ storageState: MEMBER_FILE });
  test.beforeEach(async ({ page }) => {
    await page.goto("/login", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  // TODO: Need to redirect
});
