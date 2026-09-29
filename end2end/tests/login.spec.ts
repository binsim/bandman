import { expect, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";

test.describe("login", () => {
  test.beforeAll(async ({ browser }) => {

  });

  test("logs in as member and shwos session anem", async ({ page }) => {
    await gotoHydrated(page, "/login");

  })

  test("logs in as Admin and shows session name", async ({ page }) => {
    await gotoHydrated(page, "/login");

    await expect(page.getByRole("link", { name: "Admin" })).toBeHidden();
    await expect(page.getByRole("link", { name: "Finance" })).toBeHidden();

    await expect(page.getByTestId("login-form")).toBeVisible();
    const select = page.getByTestId("login-member-select");
    await expect(select).toBeVisible();
    await select.selectOption({ label: "Admin" });
    await page.getByTestId("login-submit").click();

    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByTestId("session-name")).toHaveText("Admin");
    await expect(page.getByRole("link", { name: "Admin" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Finance" })).toBeVisible();
  });

  test("requires choosing a member", async ({ page }) => {
    await gotoHydrated(page, "/login");
    await page.getByTestId("login-submit").click();
    await expect(page.getByTestId("login-error")).toBeVisible();
  });
});
