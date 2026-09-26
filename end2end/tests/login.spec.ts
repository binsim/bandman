import { expect, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";

test.describe("login", () => {
  test("logs in as Admin and shows session name", async ({ page }) => {
    await gotoHydrated(page, "/login");

    await expect(page.getByTestId("login-form")).toBeVisible();
    const select = page.getByTestId("login-member-select");
    await expect(select).toBeVisible();
    await select.selectOption({ label: "Admin" });
    await page.getByTestId("login-submit").click();

    await expect(page).toHaveURL(/\/$/);
    await expect(page.getByTestId("session-name")).toHaveText("Admin");
  });

  test("requires choosing a member", async ({ page }) => {
    await gotoHydrated(page, "/login");
    await page.getByTestId("login-submit").click();
    await expect(page.getByTestId("login-error")).toBeVisible();
  });
});
