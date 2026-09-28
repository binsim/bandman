import { expect, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";

test.describe("Admin member management", () => {
  test("creates members, updates access, and preserves an active admin", async ({
    page,
  }) => {
    const memberName = `AAA E2E Participant ${Date.now()}`;

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
    const createNameInputHeight = await page
      .getByTestId("admin-member-name")
      .evaluate((element) => element.getBoundingClientRect().height);
    await expect(page.getByTestId("admin-member-role")).toHaveCSS(
      "height",
      `${createNameInputHeight}px`,
    );
    await page.getByTestId("admin-create-member").click();

    const createdMemberRow = page.locator(`[data-member-name="${memberName}"]`);
    await expect(createdMemberRow).toBeVisible();
    const memberId = await createdMemberRow.getAttribute("data-member-id");
    if (!memberId) {
      throw new Error("Created member row is missing its stable member ID");
    }
    const memberRow = page.locator(`[data-member-id="${memberId}"]`);

    const nameInputHeight = await memberRow
      .getByTestId("admin-member-name-input")
      .evaluate((element) => element.getBoundingClientRect().height);
    await expect(memberRow.getByTestId("admin-save-member")).toHaveCSS(
      "height",
      `${nameInputHeight}px`,
    );
    await expect(memberRow.getByTestId("admin-save-member")).toBeDisabled();
    await expect(memberRow.getByTestId("admin-save-member-role")).toBeDisabled();
    await expect(memberRow.getByTestId("admin-reset-member")).toBeDisabled();

    const roleSelectHeight = await memberRow
      .locator("select")
      .evaluate((element) => element.getBoundingClientRect().height);
    await expect(memberRow.getByTestId("admin-save-member-role")).toHaveCSS(
      "height",
      `${roleSelectHeight}px`,
    );

    await memberRow.locator("select").selectOption("participant");
    await memberRow.getByTestId("admin-member-name-input").fill("Admin");
    await expect(memberRow.getByTestId("admin-save-member")).toBeEnabled();
    await memberRow.getByTestId("admin-save-member").click();
    await expect(memberRow.getByTestId("admin-row-error")).toContainText(
      "A member with this name already exists",
    );
    await memberRow.getByTestId("admin-reset-member").click();
    await expect(memberRow.getByTestId("admin-member-name-input")).toHaveValue(
      memberName,
    );
    await expect(memberRow.locator("select")).toHaveValue("member");
    await expect(memberRow.getByTestId("admin-save-member")).toBeDisabled();
    await expect(memberRow.getByTestId("admin-save-member-role")).toBeDisabled();
    await memberRow.locator("select").selectOption("participant");

    await memberRow
      .getByTestId("admin-member-name-input")
      .fill(`${memberName} Draft`);
    await memberRow.getByTestId("admin-toggle-active").click();
    await expect(memberRow).toContainText("Inactive");
    await expect(memberRow.getByTestId("admin-member-name-input")).toHaveValue(
      `${memberName} Draft`,
    );
    await expect(memberRow.getByTestId("admin-row-error")).toHaveCount(0);
    await expect(page.getByTestId("admin-member-row").first()).toHaveAttribute(
      "data-member-id",
      memberId,
    );
    await memberRow.getByTestId("admin-reset-member").click();
    await memberRow.getByTestId("admin-toggle-active").click();
    await expect(memberRow).toContainText("Active");

    const renamedMember = `${memberName} Renamed`;
    await memberRow.getByTestId("admin-member-name-input").fill(renamedMember);
    await memberRow.getByTestId("admin-save-member").click();
    const renamedRow = page.locator(`[data-member-name="${renamedMember}"]`);
    await expect(renamedRow).toBeVisible();
    await memberRow.locator("select").selectOption("participant");
    await renamedRow.getByTestId("admin-save-member-role").click();
    await expect(renamedRow.locator("select")).toHaveValue("participant");
    await expect(renamedRow.getByTestId("admin-row-error")).toHaveCount(0);

    const adminRow = page.locator('[data-member-name="Admin"]');
    await adminRow.getByTestId("admin-toggle-active").click();
    await expect(adminRow.getByTestId("admin-row-error")).toContainText(
      "At least one active admin must remain",
    );
    await expect(adminRow.getByTestId("admin-delete-member")).toBeHidden();

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

    await page.getByTestId("logout-button").click();
    await expect(page).toHaveURL(/\/login$/);
    await page
      .getByTestId("login-member-select")
      .selectOption({ label: "Admin" });
    await page.getByTestId("login-submit").click();
    await expect(page.getByTestId("session-name")).toHaveText("Admin");
    await page.getByRole("link", { name: "Admin" }).click();

    await renamedRow.getByTestId("admin-delete-member").click();
    const deleteDialog = page.getByRole("dialog", { name: "Delete member?" });
    await expect(deleteDialog).toBeVisible();
    await expect(deleteDialog).toContainText(
      `Permanently delete ${renamedMember}?`,
    );
    await expect(deleteDialog).toHaveCSS("text-align", "left");
    const deleteButtonBox = await deleteDialog
      .getByTestId("admin-confirm-delete")
      .boundingBox();
    const cancelButtonBox = await deleteDialog
      .getByTestId("admin-cancel-delete")
      .boundingBox();
    expect(deleteButtonBox).not.toBeNull();
    expect(cancelButtonBox).not.toBeNull();
    expect(cancelButtonBox!.y).toBe(deleteButtonBox!.y);
    await renamedRow.getByTestId("admin-cancel-delete").click();
    await expect(deleteDialog).toBeHidden();
    await expect(renamedRow).toBeVisible();

    await renamedRow.getByTestId("admin-delete-member").click();
    await expect(deleteDialog).toBeVisible();
    await renamedRow.getByTestId("admin-confirm-delete").click();
    await expect(deleteDialog).toBeHidden();
    await expect(renamedRow).toBeHidden();
    await expect(page.getByTestId("admin-table-feedback")).toHaveCount(0);
  });
});
