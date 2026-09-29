import { expect, Locator, Page, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";
import { randomUUID } from "node:crypto";

function isArraySorted(arr: string[]): boolean {
  for (let i = 0; i < arr.length - 1; i++) {
    if (arr[i].localeCompare(arr[i + 1]) > 0) {
      return false; // Found an element that is greater than the next one
    }
  }
  return true; // All elements are in order
}

async function loginAsAdmin(page: Parameters<typeof gotoHydrated>[0]) {
  await gotoHydrated(page, "/login");
  await page
    .getByTestId("login-member-select")
    .selectOption({ label: "Admin" });
  await page.getByTestId("login-submit").click();
  await expect(page.getByTestId("session-name")).toHaveText("Admin");
}

test.describe("Unauthorized access", () => {
  test("denies access when not logged in", async ({ page }) => {
    await gotoHydrated(page, "/admin");

    await expect(page.getByTestId("admin-access-denied")).toBeVisible();
    await expect(
      page.getByTestId("topbar").getByRole("link", { name: /log in/i }),
    ).toBeVisible();
  });
});

test.describe("Admin member management", () => {
  let nameInput: Locator;
  let roleSelect: Locator;
  let createButton: Locator;

  async function createMember(
    page: Page,
    prefix: string | undefined = undefined,
  ): Promise<[locator: Locator, memberName: string]> {
    const memberName = prefix ? `${prefix} ${randomUUID()}` : randomUUID();
    await nameInput.fill(memberName);
    await createButton.click();

    return [
      page.locator(
        `[data-member-id="${await page.locator(`[data-member-name="${memberName}"]`).getAttribute("data-member-id")}"]`,
      ),
      memberName,
    ];
  }

  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page);
    await page.getByRole("link", { name: "Admin" }).click();

    nameInput = page.getByTestId("admin-member-name");
    roleSelect = page.getByTestId("admin-member-role");
    createButton = page.getByTestId("admin-create-member");
  });

  test("Add member is well formatted", async ({ page }) => {
    await expect(nameInput).toBeVisible();
    await expect(roleSelect).toBeVisible();
    await expect(createButton).toBeVisible();

    const createNameInputHeight = (await nameInput.boundingBox())!.height;
    expect((await roleSelect.boundingBox())?.height).toBeCloseTo(
      createNameInputHeight,
      0,
    );
    expect((await createButton.boundingBox())?.height).toBeCloseTo(
      createNameInputHeight,
      0,
    );

    await expect(nameInput).toHaveAttribute("required", "");
    expect(
      await nameInput.evaluate((input) =>
        (input as HTMLInputElement).checkValidity(),
      ),
    ).toBe(false);
    await createButton.click();
    await expect(page.getByTestId("admin-create-error")).toHaveCount(0);
  });

  test("Members list is well formatted", async ({ page }) => {
    const adminRow = page.locator('[data-member-name="Admin"]');

    const input = adminRow.getByTestId("admin-member-name-input");
    const select = adminRow.getByTestId("admin-member-role-select");
    const status = adminRow.getByTestId("admin-toggle-active");
    const saveButton = adminRow.getByTestId("admin-save-member");
    const resetButton = adminRow.getByTestId("admin-reset-member");

    await expect(adminRow).toBeVisible();
    await expect(input).toBeVisible();
    await expect(select).toBeVisible();
    await expect(status).toBeVisible();
    await expect(saveButton).toBeVisible();
    await expect(resetButton).toBeVisible();

    const inputHeight = (await input.boundingBox())!.height;
    expect((await select.boundingBox())?.height).toBeCloseTo(inputHeight, 0);
    expect((await saveButton.boundingBox())?.height).toBeCloseTo(
      inputHeight,
      0,
    );
    expect((await resetButton.boundingBox())?.height).toBeCloseTo(
      inputHeight,
      0,
    );

    await expect(saveButton).toHaveClass("btn btn-ghost btn-sm");
    await expect(resetButton).toHaveClass("btn btn-ghost btn-sm");
  });

  test("reject case-insensitive duplicates", async ({ page }) => {
    await nameInput.fill("admin");
    await roleSelect.selectOption("member");
    await createButton.click();

    await expect(page.getByTestId("admin-create-error")).toContainText(
      "A member with this name already exists",
    );
    await expect(page.locator('[data-member-name="Admin"]')).toHaveCount(1);
  });

  test("admin can not delete themselves", async ({ page }) => {
    const adminRow = page.locator('[data-member-name="Admin"]');
    await expect(adminRow.getByTestId("admin-delete-member")).toHaveCount(0);
  });

  test("Save and reset disabled state updates when name is changed", async ({
    page,
  }) => {
    function getAdminRow(adminName: string): Locator {
      return page.locator(`[data-member-name="${adminName}"]`);
    }
    function getSaveButton(adminName: string): Locator {
      return getAdminRow(adminName).getByTestId("admin-save-member");
    }
    function getResetButton(adminName: string): Locator {
      return getAdminRow(adminName).getByTestId("admin-reset-member");
    }
    function getNameInput(adminName: string): Locator {
      return getAdminRow(adminName).getByTestId("admin-member-name-input");
    }

    await expect(getSaveButton("Admin")).toBeDisabled();
    await expect(getResetButton("Admin")).toBeDisabled();

    await getNameInput("Admin").fill("");
    await expect(getSaveButton("")).toBeEnabled();
    await expect(getResetButton("")).toBeEnabled();

    await getNameInput("").fill("Admin");
    await expect(getSaveButton("Admin")).toBeDisabled();
    await expect(getResetButton("Admin")).toBeDisabled();

    await expect(getAdminRow("Admin")).toBeVisible();
  });

  test("Save and reset disabled state updates when status is changed", async ({
    page,
  }) => {
    const adminRow = page.locator('[data-member-name="Admin"]');
    const saveButton = adminRow.getByTestId("admin-save-member");
    const resetButton = adminRow.getByTestId("admin-reset-member");
    const status = adminRow.getByTestId("admin-toggle-active");

    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await status.click();
    await expect(saveButton).toBeEnabled();
    await expect(resetButton).toBeEnabled();

    await status.click();
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await expect(adminRow).toBeVisible();
  });

  test("Save and reset disabled state updates when role is changed", async ({
    page,
  }) => {
    const adminRow = page.locator('[data-member-name="Admin"]');
    const saveButton = adminRow.getByTestId("admin-save-member");
    const resetButton = adminRow.getByTestId("admin-reset-member");
    const role = adminRow.getByTestId("admin-member-role-select");

    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await role.selectOption("member");
    await expect(saveButton).toBeEnabled();
    await expect(resetButton).toBeEnabled();

    await role.selectOption("admin");
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await expect(adminRow).toBeVisible();
  });

  test("create member adds a member to the list", async ({ page }) => {
    const [createdMember, _] = await createMember(page);
    await expect(createdMember).toBeVisible();

    const saveButton = createdMember.getByTestId("admin-save-member");
    const resetButton = createdMember.getByTestId("admin-reset-member");
    const deleteButton = createdMember.getByTestId("admin-delete-member");
    const saveButtonHeight = (await saveButton.boundingBox())!.height;
    expect((await deleteButton.boundingBox())?.height).toBeCloseTo(
      saveButtonHeight,
      0,
    );

    await expect(saveButton).toBeVisible();
    await expect(resetButton).toBeVisible();
    await expect(deleteButton).toBeVisible();
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();
    await expect(deleteButton).toBeEnabled();

    await deleteButton.click();
    await createdMember.getByTestId("admin-confirm-delete").click();
    await expect(createdMember).not.toBeVisible();
  });

  test("renaming fails when renaming to an existing name", async ({ page }) => {
    const [createdMember, createdMemberName] = await createMember(page);
    await page.reload({ waitUntil: "networkidle" });
    const [createdMember2, ] = await createMember(page);

    await expect(createdMember).toBeVisible();
    await expect(createdMember2).toBeVisible();

    await createdMember2
      .getByTestId("admin-member-name-input")
      .fill(createdMemberName);
    await createdMember2.getByTestId("admin-save-member").isEnabled();
    await createdMember2.getByTestId("admin-save-member").click();
    await expect(createdMember2.getByTestId("admin-row-error")).toContainText(
      "A member with this name already exists",
    );

    await createdMember.getByTestId("admin-delete-member").click();
    await createdMember.getByTestId("admin-confirm-delete").click();
    await createdMember2.getByTestId("admin-delete-member").click();
    await createdMember2.getByTestId("admin-confirm-delete").click();
  });

  test("resets member uses previously saved values", async ({ page }) => {
    const [createdMember, memberName] = await createMember(page);
    const memberInput = createdMember.getByTestId("admin-member-name-input");
    const memberSelect = createdMember.getByTestId("admin-member-role-select");
    const memberStatus = createdMember.getByTestId("admin-toggle-active");
    const saveButton = createdMember.getByTestId("admin-save-member");
    const resetButton = createdMember.getByTestId("admin-reset-member");

    await expect(memberInput).toHaveValue(memberName);
    await expect(memberSelect).toHaveValue("member");
    await expect(memberStatus).toHaveText("Active");
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await memberInput.fill(`${memberName} Edited`);
    await expect(saveButton).toBeEnabled();
    await expect(resetButton).toBeEnabled();

    await memberInput.fill(memberName);
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await memberInput.fill(`${memberName} Edited`);
    await memberSelect.selectOption("admin");
    await memberStatus.click();

    await expect(memberStatus).toHaveText("Inactive");
    await expect(saveButton).toBeEnabled();
    await expect(resetButton).toBeEnabled();

    await resetButton.click();

    await expect(memberInput).toHaveValue(memberName);
    await expect(memberSelect).toHaveValue("member");
    await expect(memberStatus).toHaveText("Active");
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await createdMember.getByTestId("admin-delete-member").click();
    await createdMember.getByTestId("admin-confirm-delete").click();
  });

  test("save member uses saves values", async ({ page }) => {
    const [createdMember, memberName] = await createMember(page);
    const memberInput = createdMember.getByTestId("admin-member-name-input");
    const memberSelect = createdMember.getByTestId("admin-member-role-select");
    const memberStatus = createdMember.getByTestId("admin-toggle-active");
    const saveButton = createdMember.getByTestId("admin-save-member");

    await expect(memberInput).toHaveValue(memberName);
    await expect(memberSelect).toHaveValue("member");
    await expect(memberStatus).toHaveText("Active");

    await memberInput.fill(`${memberName} Edited`);
    await memberSelect.selectOption("admin");
    await memberStatus.click();
    await expect(memberStatus).toHaveText("Inactive");
    await saveButton.click();

    // TODO: remove waitForTimeout
    await page.waitForTimeout(20);
    await page.reload({ waitUntil: "networkidle" });

    await expect(createdMember).toBeVisible();
    await expect(memberInput).toHaveValue(`${memberName} Edited`);
    await expect(memberSelect).toHaveValue("admin");
    await expect(memberStatus).toHaveText("Inactive");

    await expect(saveButton).toBeDisabled();
    await createdMember.getByTestId("admin-delete-member").click();
    await createdMember.getByTestId("admin-confirm-delete").click();
  });

  test("list of members is ordered after adding new member", async ({
    page,
  }) => {
    const [createdMember1] = await createMember(page, "AAA");
    const [createdMember2] = await createMember(page, "BBB");
    const [createdMember3] = await createMember(page, "CCC");

    await expect
      .poll(async () => {
        const rowNames = await page
          .getByTestId("admin-member-row")
          .evaluateAll((rows) =>
            rows.map((row) => row.getAttribute("data-member-name")!),
          );

        return isArraySorted(rowNames);
      })
      .toBe(true);

    await createdMember1.getByTestId("admin-delete-member").click();
    await createdMember1.getByTestId("admin-confirm-delete").click();
    await createdMember2.getByTestId("admin-delete-member").click();
    await createdMember2.getByTestId("admin-confirm-delete").click();
    await createdMember3.getByTestId("admin-delete-member").click();
    await createdMember3.getByTestId("admin-confirm-delete").click();
  });

  test("list of members is ordered after renaming member", async ({ page }) => {
    const [createdMember1] = await createMember(page, "AAA");
    const [createdMember2] = await createMember(page, "DDD");
    const [createdMember3] = await createMember(page, "CCC");
    const memberName2AfterRename = `BBB ${randomUUID()}`;

    await createdMember2
      .getByTestId("admin-member-name-input")
      .fill(memberName2AfterRename);
    await createdMember2.getByTestId("admin-save-member").click();

    await expect
      .poll(async () => {
        const rowNames = await page
          .getByTestId("admin-member-row")
          .evaluateAll((rows) =>
            rows.map((row) => row.getAttribute("data-member-name")!),
          );
        return isArraySorted(rowNames);
      })
      .toBe(true);

    await createdMember1.getByTestId("admin-delete-member").click();
    await createdMember1.getByTestId("admin-confirm-delete").click();
    await createdMember2.getByTestId("admin-delete-member").click();
    await createdMember2.getByTestId("admin-confirm-delete").click();
    await createdMember3.getByTestId("admin-delete-member").click();
    await createdMember3.getByTestId("admin-confirm-delete").click();
  });

  test("At least one active admin must remain", async ({ page }) => {
    const adminRow = page.locator('[data-member-name="Admin"]');
    const errorMessage = adminRow.getByTestId("admin-row-error");

    const select = adminRow.getByTestId("admin-member-role-select");
    const status = adminRow.getByTestId("admin-toggle-active");
    const saveButton = adminRow.getByTestId("admin-save-member");
    const resetButton = adminRow.getByTestId("admin-reset-member");

    await select.selectOption("member");
    await saveButton.click();
    await expect(errorMessage).toContainText(
      "At least one active admin must remain",
    );

    await resetButton.click();
    expect(errorMessage).toBeHidden();

    await status.click();
    await expect(adminRow).toContainText("Inactive");
    await saveButton.click();
    await expect(errorMessage).toContainText(
      "At least one active admin must remain",
    );

    await expect(adminRow.getByTestId("admin-delete-member")).toBeHidden();
    await expect(adminRow.getByTestId("admin-confirm-delete")).toBeHidden();
  });

  test("deleting member shows confirmation dialog", async ({ page }) => {
    const [createdMember, memberName] = await createMember(page);

    await createdMember.getByTestId("admin-delete-member").click();
    const deleteDialog = page.getByRole("dialog", { name: "Delete member?" });
    await expect(deleteDialog).toBeVisible();
    await expect
      .poll(async () =>
        (await deleteDialog.locator("p").textContent())?.replace(
          /[\u2066-\u2069]/g,
          "",
        ),
      )
      .toBe(`Permanently delete ${memberName}?`);
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
    await createdMember.getByTestId("admin-cancel-delete").click();
    await expect(deleteDialog).toBeHidden();
    await expect(createdMember).toBeVisible();

    await createdMember.getByTestId("admin-delete-member").click();
    await expect(deleteDialog).toBeVisible();
    await createdMember.getByTestId("admin-confirm-delete").click();
    await expect(deleteDialog).toBeHidden();
    await expect(createdMember).toBeHidden();
    await expect(page.getByTestId("admin-table-feedback")).toHaveCount(0);
  });
});
