import { expect, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";

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
});
