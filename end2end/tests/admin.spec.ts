import { expect, Locator, Page, test } from "@playwright/test";
import { gotoHydrated, isArraySorted, loginAsAdmin } from "./helpers";
import { randomUUID } from "node:crypto";

const ADD_MEMBER_NAME_INPUT = "admin-member-name";
const ADD_MEMBER_ROLE_SELECT = "admin-member-role";
const ADD_MEMBER_CREATE_BUTTON = "admin-create-member";
const ADD_MEMBER_ERROR = "admin-member-error";

const MEMBER_ROW_NAME_INPUT = "admin-member-name-input";
const MEMBER_ROW_ROLE_SELECT = "admin-member-role-select";
const MEMBER_ROW_STATUS = "admin-toggle-active";
const MEMBER_ROW_SAVE_BUTTON = "admin-save-member";
const MEMBER_ROW_RESET_BUTTON = "admin-reset-member";
const MEMBER_ROW_DELETE_BUTTON = "admin-delete-member";
const MEMBER_ROW_ERROR = "admin-row-error";

const DELETE_POPUP_CONFIRM_DELETE_BUTTON = "admin-confirm-delete";
const DELETE_POPUP_CANCEL_DELETE_BUTTON = "admin-cancel-delete";

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

    return [await getMemberRow(page, memberName), memberName];
  }

  async function getMemberRow(
    page: Page,
    memberName: string,
  ): Promise<Locator> {
    return page.locator(
      `[data-member-id="${await page.locator(`[data-member-name="${memberName}"]`).getAttribute("data-member-id")}"]`,
    );
  }

  test.beforeEach(async ({ page }) => {
    await loginAsAdmin(page);
    await page.getByRole("link", { name: "Admin" }).click();

    nameInput = page.getByTestId(ADD_MEMBER_NAME_INPUT);
    roleSelect = page.getByTestId(ADD_MEMBER_ROLE_SELECT);
    createButton = page.getByTestId(ADD_MEMBER_CREATE_BUTTON);
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
    await expect(page.getByTestId(ADD_MEMBER_ERROR)).toHaveCount(0);
  });

  test("Members list is well formatted", async ({ page }) => {
    const adminRow = await getMemberRow(page, "Admin");

    const input = adminRow.getByTestId(MEMBER_ROW_NAME_INPUT);
    const select = adminRow.getByTestId(MEMBER_ROW_ROLE_SELECT);
    const status = adminRow.getByTestId(MEMBER_ROW_STATUS);
    const saveButton = adminRow.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = adminRow.getByTestId(MEMBER_ROW_RESET_BUTTON);

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
    await expect(adminRow.getByTestId(MEMBER_ROW_DELETE_BUTTON)).toHaveCount(0);
  });

  test("Save and reset disabled state updates when name is changed", async ({
    page,
  }) => {
    const adminRow = await getMemberRow(page, "Admin");
    const saveButton = adminRow.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = adminRow.getByTestId(MEMBER_ROW_RESET_BUTTON);
    const nameInput = adminRow.getByTestId(MEMBER_ROW_NAME_INPUT);

    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();
    await expect(nameInput).toHaveValue("Admin");

    await nameInput.fill("");
    await expect(saveButton).toBeEnabled();
    await expect(resetButton).toBeEnabled();

    await nameInput.fill("Admin");
    await expect(saveButton).toBeDisabled();
    await expect(resetButton).toBeDisabled();

    await expect(adminRow).toBeVisible();
  });

  test("Save and reset disabled state updates when status is changed", async ({
    page,
  }) => {
    const adminRow = await getMemberRow(page, "Admin");
    const saveButton = adminRow.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = adminRow.getByTestId(MEMBER_ROW_RESET_BUTTON);
    const status = adminRow.getByTestId(MEMBER_ROW_STATUS);

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
    const adminRow = await getMemberRow(page, "Admin");
    const saveButton = adminRow.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = adminRow.getByTestId(MEMBER_ROW_RESET_BUTTON);
    const role = adminRow.getByTestId(MEMBER_ROW_ROLE_SELECT);

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

    const saveButton = createdMember.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = createdMember.getByTestId(MEMBER_ROW_RESET_BUTTON);
    const deleteButton = createdMember.getByTestId(MEMBER_ROW_DELETE_BUTTON);
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
    await createdMember.getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON).click();
    await expect(createdMember).not.toBeVisible();
  });

  test("renaming fails when renaming to an existing name", async ({ page }) => {
    const [createdMember, createdMemberName] = await createMember(page);
    const [createdMember2] = await createMember(page);

    const saveButton = createdMember2.getByTestId(MEMBER_ROW_SAVE_BUTTON);

    await expect(createdMember).toBeVisible();
    await expect(createdMember2).toBeVisible();

    await createdMember2
      .getByTestId(MEMBER_ROW_NAME_INPUT)
      .fill(createdMemberName);
    await saveButton.isEnabled();
    await saveButton.click();
    await expect(createdMember2.getByTestId(MEMBER_ROW_ERROR)).toContainText(
      "A member with this name already exists",
    );

    await createdMember.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember.getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON).click();
    await createdMember2.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember2
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
  });

  test("resets member uses previously saved values", async ({ page }) => {
    const [createdMember, memberName] = await createMember(page);
    const memberInput = createdMember.getByTestId(MEMBER_ROW_NAME_INPUT);
    const memberSelect = createdMember.getByTestId(MEMBER_ROW_ROLE_SELECT);
    const memberStatus = createdMember.getByTestId(MEMBER_ROW_STATUS);
    const saveButton = createdMember.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = createdMember.getByTestId(MEMBER_ROW_RESET_BUTTON);

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

    await createdMember.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember.getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON).click();
  });

  test("save member uses saves values", async ({ page }) => {
    const [createdMember, memberName] = await createMember(page);
    const memberInput = createdMember.getByTestId(MEMBER_ROW_NAME_INPUT);
    const memberSelect = createdMember.getByTestId(MEMBER_ROW_ROLE_SELECT);
    const memberStatus = createdMember.getByTestId(MEMBER_ROW_STATUS);
    const saveButton = createdMember.getByTestId(MEMBER_ROW_SAVE_BUTTON);

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
    await createdMember.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember.getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON).click();
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

    await createdMember1.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember1
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
    await createdMember2.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember2
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
    await createdMember3.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember3
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
  });

  test("list of members is ordered after renaming member", async ({ page }) => {
    const [createdMember1] = await createMember(page, "AAA");
    const [createdMember2] = await createMember(page, "DDD");
    const [createdMember3] = await createMember(page, "CCC");
    const memberName2AfterRename = `BBB ${randomUUID()}`;

    await createdMember2
      .getByTestId(MEMBER_ROW_NAME_INPUT)
      .fill(memberName2AfterRename);
    await createdMember2.getByTestId(MEMBER_ROW_SAVE_BUTTON).click();

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

    await createdMember1.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember1
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
    await createdMember2.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember2
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
    await createdMember3.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await createdMember3
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .click();
  });

  test("At least one active admin must remain", async ({ page }) => {
    const adminRow = await getMemberRow(page, "Admin");
    const errorMessage = adminRow.getByTestId(MEMBER_ROW_ERROR);

    const select = adminRow.getByTestId(MEMBER_ROW_ROLE_SELECT);
    const status = adminRow.getByTestId(MEMBER_ROW_STATUS);
    const saveButton = adminRow.getByTestId(MEMBER_ROW_SAVE_BUTTON);
    const resetButton = adminRow.getByTestId(MEMBER_ROW_RESET_BUTTON);

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

    await expect(adminRow.getByTestId(MEMBER_ROW_DELETE_BUTTON)).toBeHidden();
    await expect(
      adminRow.getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON),
    ).toBeHidden();
  });

  test("deleting member shows confirmation dialog", async ({ page }) => {
    const [createdMember, memberName] = await createMember(page);

    await createdMember.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
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
      .getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON)
      .boundingBox();
    const cancelButtonBox = await deleteDialog
      .getByTestId(DELETE_POPUP_CANCEL_DELETE_BUTTON)
      .boundingBox();
    expect(deleteButtonBox).not.toBeNull();
    expect(cancelButtonBox).not.toBeNull();
    expect(cancelButtonBox!.y).toBe(deleteButtonBox!.y);
    await createdMember.getByTestId(DELETE_POPUP_CANCEL_DELETE_BUTTON).click();
    await expect(deleteDialog).toBeHidden();
    await expect(createdMember).toBeVisible();

    await createdMember.getByTestId(MEMBER_ROW_DELETE_BUTTON).click();
    await expect(deleteDialog).toBeVisible();
    await createdMember.getByTestId(DELETE_POPUP_CONFIRM_DELETE_BUTTON).click();
    await expect(deleteDialog).toBeHidden();
    await expect(createdMember).toBeHidden();
    await expect(page.getByTestId("admin-table-feedback")).toHaveCount(0);
  });
});
