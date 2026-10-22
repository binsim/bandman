import { test, expect } from "@playwright/test";
import { ADMIN_FILE, MEMBER_FILE, PARTICIPANT_FILE } from "./helpers";

const USER_ICON = "user-icon";
const USER_CARD = "user-card";
const LANG_SWITCHER = "lang-switcher";
const THEME_TOGGLE = "theme-toggle";
const LOGIN_BUTTON = "login-btn";
const TOPBAR_NAVIGATION_CONTAINER = "topbar-navigation-container";

test.describe("No user logged in", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("Titlebar shows correct information", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeHidden();
    await expect(page.getByTestId(LOGIN_BUTTON)).toBeVisible();
    await expect(page.getByTestId("brand-name")).toHaveText("Bandman");
    await expect(page.getByTestId(TOPBAR_NAVIGATION_CONTAINER)).toBeHidden();
  });
});

test.describe("Admin user logged in", () => {
  test.use({ storageState: ADMIN_FILE });
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("Titlebar shows correct information", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeVisible();
    await expect(page.getByTestId(LOGIN_BUTTON)).toBeHidden();
    await expect(page.getByTestId("brand-name")).toHaveText("Bandman");
  });

  test("Navigation is accessible", async ({ page }) => {
    expect(
      await page
        .getByTestId(TOPBAR_NAVIGATION_CONTAINER)
        .locator("ul > li")
        .allInnerTexts(),
    ).toStrictEqual(["Program", "Wishlist", "Plan", "Finance", "Admin"]);

    const viewport = page.viewportSize();
    if (viewport!.width < 700) {
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeHidden();
      await page.getByTestId("brand-name").click();
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeVisible();

      // TODO: add checks for formatting
    } else {
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeVisible();
    }
  });

  test("Clicking user icon opens user card", async ({ page }) => {
    await page.getByTestId(USER_ICON).click();

    await expect(page.getByTestId(USER_CARD)).toBeVisible();
    await expect(page.getByTestId(LANG_SWITCHER)).toBeVisible();
    await expect(page.getByTestId(THEME_TOGGLE)).toBeVisible();

    await expect(page.getByTestId(LOGIN_BUTTON)).toBeVisible();
  });
});

test.describe("Participant user logged in", () => {
  test.use({ storageState: PARTICIPANT_FILE });
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("Titlebar shows correct information", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeVisible();
    await expect(page.getByTestId(LOGIN_BUTTON)).toBeHidden();
    await expect(page.getByTestId("brand-name")).toHaveText("Bandman");
  });

  test("Navigation is accessible", async ({ page }) => {
    expect(
      await page
        .getByTestId(TOPBAR_NAVIGATION_CONTAINER)
        .locator("ul > li")
        .allInnerTexts(),
    ).toStrictEqual(["Program", "Wishlist", "Plan", "Finance"]);

    const viewport = page.viewportSize();
    if (viewport!.width < 700) {
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeHidden();
      await page.getByTestId("brand-name").click();
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeVisible();

      // TODO: add checks for formatting
    } else {
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeVisible();
    }
  });

  test("Clicking user icon opens user card", async ({ page }) => {
    await page.getByTestId(USER_ICON).click();

    await expect(page.getByTestId(USER_CARD)).toBeVisible();
    await expect(page.getByTestId(LANG_SWITCHER)).toBeVisible();
    await expect(page.getByTestId(THEME_TOGGLE)).toBeVisible();

    await expect(page.getByTestId(LOGIN_BUTTON)).toBeVisible();
  });
});

test.describe("Member user logged in", () => {
  test.use({ storageState: MEMBER_FILE });
  test.beforeEach(async ({ page }) => {
    await page.goto("/", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("Titlebar shows correct information", async ({ page }) => {
    await expect(page.getByTestId(USER_ICON)).toBeVisible();
    await expect(page.getByTestId(LOGIN_BUTTON)).toBeHidden();
    await expect(page.getByTestId("brand-name")).toHaveText("Bandman");
  });

  test("Navigation is accessible", async ({ page }) => {
    expect(
      await page
        .getByTestId(TOPBAR_NAVIGATION_CONTAINER)
        .locator("ul > li")
        .allInnerTexts(),
    ).toStrictEqual(["Program", "Wishlist", "Plan"]);

    const viewport = page.viewportSize();
    if (viewport!.width < 700) {
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeHidden();
      await page.getByTestId("brand-name").click();
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeVisible();

      // TODO: add checks for formatting
    } else {
      expect(
        page.getByTestId(TOPBAR_NAVIGATION_CONTAINER).locator("ul"),
      ).toBeVisible();
    }
  });

  test("Clicking user icon opens user card", async ({ page }) => {
    await page.getByTestId(USER_ICON).click();

    await expect(page.getByTestId(USER_CARD)).toBeVisible();
    await expect(page.getByTestId(LANG_SWITCHER)).toBeVisible();
    await expect(page.getByTestId(THEME_TOGGLE)).toBeVisible();

    await expect(page.getByTestId(LOGIN_BUTTON)).toBeVisible();
  });
});
