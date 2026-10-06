import { expect, test } from "@playwright/test";
import { gotoHydrated, loginAsAdmin } from "./helpers";

test.describe("home", () => {
  test("shows brand and login affordance", async ({ page }) => {
    await gotoHydrated(page, "/");
    await expect(page.getByTestId("brand-name")).toHaveText("Bandman");
    await expect(
      page
        .getByTestId("topbar")
        .getByRole("link", { name: /log in|anmelden/i }),
    ).toBeVisible();
  });

  test("mobile logo opens navigation and preference/account controls are grouped", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await loginAsAdmin(page);

    const navigation = page.getByTestId("mobile-navigation");
    const actions = page.getByTestId("topbar-actions");
    await expect(navigation).toBeVisible();
    await expect(page.getByTestId("brand-name")).toBeHidden();
    await expect(navigation.getByTestId("mobile-brand-name")).toHaveText(
      "Bandman",
    );
    await expect(navigation.getByTestId("mobile-brand-name")).toBeVisible();
    await expect(actions.getByTestId("session-name")).toBeVisible();
    await expect(actions.getByTestId("logout-button")).toBeVisible();
    await expect(actions.getByTestId("theme-toggle")).toBeVisible();
    await expect(actions.getByTestId("lang-switcher")).toBeVisible();
    await expect(actions).toHaveCSS("border-radius", "999px");

    const trigger = page.getByTestId("mobile-navigation-trigger");
    await trigger.click();
    await expect(page.getByTestId("mobile-navigation-menu")).toBeVisible();
    await page.getByTestId("mobile-navigation-backdrop").click({
      position: { x: 380, y: 800 },
    });
    await expect(page.getByTestId("mobile-navigation-menu")).toBeHidden();

    await trigger.click();
    await navigation.getByRole("link", { name: "Wishlist" }).click();
    await expect(page).toHaveURL(/\/wishlist$/);
    await expect(page.getByTestId("mobile-navigation-menu")).toBeHidden();
  });
});
