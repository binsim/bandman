import { expect, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";

test.describe("Admin member management", () => {
  test("creates members, updates access, and preserves an active admin", async ({
    page,
  }) => {
    const memberName = `E2E Participant ${Date.now()}`;

    await gotoHydrated(page, "/login");
    await page
      .getByTestId("login-member-select")
      .selectOption({ label: "Admin" });
    await page.getByTestId("login-submit").click();
    await expect(page.getByTestId("session-name")).toHaveText("Admin");
    await expect(
      page.getByTestId("topbar").getByRole("link", { name: "Finance" }),
    ).toBeHidden();
    await page.getByRole("link", { name: "Admin" }).click();

    await expect(page.getByRole("heading", { name: "Members" })).toBeVisible();
    await page.getByTestId("admin-member-name").fill(memberName);
    await page.getByTestId("admin-member-role").selectOption("member");
    await page.getByTestId("admin-create-member").click();

    const memberRow = page.locator(`[data-member-name="${memberName}"]`);
    await expect(memberRow).toBeVisible();

    await memberRow.locator("select").selectOption("participant");
    await memberRow.getByTestId("admin-save-role").click();
    await expect(memberRow.locator("select")).toHaveValue("participant");

    await memberRow.getByTestId("admin-toggle-active").click();
    await expect(memberRow).toContainText("Inactive");
    await memberRow.getByTestId("admin-toggle-active").click();
    await expect(memberRow).toContainText("Active");

    const renamedMember = `${memberName} Renamed`;
    await memberRow.getByTestId("admin-member-name-input").fill("Admin");
    await memberRow.getByTestId("admin-save-name").click();
    await expect(page.getByTestId("admin-feedback")).toContainText(
      "A member with this name already exists",
    );
    await memberRow.getByTestId("admin-member-name-input").fill(renamedMember);
    await memberRow.getByTestId("admin-save-name").click();
    const renamedRow = page.locator(`[data-member-name="${renamedMember}"]`);
    await expect(renamedRow).toBeVisible();

    const adminRow = page.locator('[data-member-name="Admin"]');
    await adminRow.getByTestId("admin-toggle-active").click();
    await expect(page.getByTestId("admin-feedback")).toContainText(
      "At least one active admin must remain",
    );

    await page.getByTestId("logout-button").click();
    await expect(page).toHaveURL(/\/login$/);
    await page
      .getByTestId("login-member-select")
      .selectOption({ label: renamedMember });
    await page.getByTestId("login-submit").click();
    await expect(page.getByTestId("session-name")).toHaveText(renamedMember);
    await expect(
      page.getByTestId("topbar").getByRole("link", { name: "Finance" }),
    ).toBeVisible();
    await expect(
      page.getByTestId("topbar").getByRole("link", { name: "Admin" }),
    ).toBeHidden();

    await page.goto("/admin");
    await expect(page.getByTestId("admin-access-denied")).toBeVisible();
  });
});
