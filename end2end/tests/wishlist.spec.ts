import {
  test,
  expect,
  APIRequestContext,
  Locator,
  Page,
} from "@playwright/test";
import { ADMIN_FILE, MEMBER_FILE, PARTICIPANT_FILE } from "./helpers";

const WISHLIST_DUPLICATE_WARNING_ERROR = "wishlist-duplicate-warning";
const testRunId = Date.now();
const heavyCrossTitle = `Heavy Cross ${testRunId}`;
const testMedleyName = `Test Medley ${testRunId}`;
const categoryName = `Party ${testRunId}`;
const labelNames = [`Short ${testRunId}`, `Two guitars ${testRunId}`];
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:3000";
const wishlistRoles = [
  ["Member", MEMBER_FILE],
  ["Participant", PARTICIPANT_FILE],
  ["Admin", ADMIN_FILE],
] as const;
type WishlistRole = (typeof wishlistRoles)[number][0];
type CreatedSong = { id: string; title: string; role?: WishlistRole };

test.describe("No user logged in", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/wishlist", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("page should not be accessible", async ({ page }) => {
    await expect(
      page
        .getByRole("main")
        .getByRole("link", { name: /^(Anmelden|Log in)$/i }),
    ).toBeVisible();
  });
});

test.describe("Wishlist", () => {
  let titleInput: Locator;
  let artistInput: Locator;
  let linkInput: Locator;
  let tempoInput: Locator;
  let musicalKeyInput: Locator;
  let addSongButton: Locator;

  const createdSongs: CreatedSong[] = [];
  let categoryId: string;
  const labelIds: string[] = [];

  async function createSong(
    request: APIRequestContext,
    songInfo: {
      title: string;
      artist: string;
      link: string;
      tempo: string;
      musical_key: string;
    },
    role?: WishlistRole,
  ): Promise<CreatedSong> {
    const response = await request.post("/api/wishlist_create", {
      form: songInfo,
    });
    if (!response.ok()) {
      throw new Error(
        `Could not create song: ${JSON.stringify(songInfo)} due to ${await response.text()}`,
      );
    }

    const song = (await response.json()) as { id: string };
    const createdSong = {
      id: song.id,
      title: songInfo.title,
      ...(role && { role }),
    };
    createdSongs.push(createdSong);
    return createdSong;
  }

  function getSongRow(page: Page, songTitle: string): Locator {
    return page.locator(`[data-song-name="${songTitle}"]`).locator("xpath=..");
  }

  test.beforeAll(async ({ browser, request }) => {
    const adminContext = await browser.newContext({
      baseURL,
      storageState: ADMIN_FILE,
    });
    try {
      const categoryResponse = await adminContext.request.post(
        "/api/admin_create_song_category",
        { form: { name: categoryName } },
      );
      if (!categoryResponse.ok()) {
        throw new Error(
          `Could not create category: ${await categoryResponse.text()}`,
        );
      }
      categoryId = ((await categoryResponse.json()) as { id: string }).id;

      for (const name of labelNames) {
        const labelResponse = await adminContext.request.post(
          "/api/admin_create_song_label",
          { form: { name, color: "#2563EB" } },
        );
        if (!labelResponse.ok()) {
          throw new Error(
            `Could not create label: ${await labelResponse.text()}`,
          );
        }
        labelIds.push(((await labelResponse.json()) as { id: string }).id);
      }
    } finally {
      await adminContext.close();
    }

    await createSong(request, {
      title: heavyCrossTitle,
      artist: "Gossip",
      link: "https://www.youtube.com/watch?v=tjc36rR4Mq0",
      tempo: "120",
      musical_key: "C major",
    });

    // Create a song from every role
    for (const [role, storageState] of wishlistRoles) {
      const context = await browser.newContext({ baseURL, storageState });
      try {
        await createSong(
          context.request,
          {
            title: `Medley Song ${role} ${testRunId}`,
            artist: "",
            link: "",
            tempo: "",
            musical_key: "",
          },
          role,
        );
      } finally {
        await context.close();
      }
    }
  });

  test.afterAll(async ({ browser, request }) => {
    await Promise.all(
      createdSongs.map((song) => {
        return request.post("/api/wishlist_delete", {
          form: { item_id: song.id },
        });
      }),
    );

    const adminContext = await browser.newContext({
      baseURL,
      storageState: ADMIN_FILE,
    });
    try {
      await Promise.all(
        labelIds.map((id) =>
          adminContext.request.post("/api/admin_delete_song_label", {
            form: { id },
          }),
        ),
      );
      if (categoryId) {
        await adminContext.request.post("/api/admin_delete_song_category", {
          form: { id: categoryId },
        });
      }
    } finally {
      await adminContext.close();
    }
  });

  for (const [role, storageState] of wishlistRoles) {
    test.describe(`${role} user logged in`, () => {
      test.use({ storageState });
      test.beforeEach(async ({ page }) => {
        await page.goto("/wishlist", { waitUntil: "networkidle" });
        await page
          .locator("[data-testid='topbar']")
          .waitFor({ state: "visible" });

        titleInput = page.getByTestId("add-song-name");
        artistInput = page.getByTestId("wishlist-artist");
        linkInput = page.getByTestId("wishlist-link");
        tempoInput = page.getByTestId("wishlist-tempo");
        musicalKeyInput = page.getByTestId("wishlist-key");
        addSongButton = page.getByTestId("create-song");
      });

      test("Add song is well formatted", async ({ page }) => {
        await expect(titleInput).toBeVisible();
        await expect(titleInput).toHaveAttribute("required");
        await expect(artistInput).toBeVisible();
        // await expect(artistInput).toHaveAttribute("required");
        await expect(linkInput).toBeVisible();
        await expect(tempoInput).toBeVisible();
        await expect(musicalKeyInput).toBeVisible();
        await expect(addSongButton).toBeVisible();
        await expect(addSongButton).toBeEnabled();

        await expect(
          page.getByTestId(WISHLIST_DUPLICATE_WARNING_ERROR),
        ).toBeHidden();
      });

      test("Add song shows warning for same title", async ({ page }) => {
        await titleInput.fill(heavyCrossTitle);
        await expect(
          page.getByTestId(WISHLIST_DUPLICATE_WARNING_ERROR),
        ).toBeVisible();
        await expect(addSongButton).toBeEnabled();
      });

      test("Add song disabeled when song already in wishlist", async ({
        page,
      }) => {
        await titleInput.fill(heavyCrossTitle);
        await artistInput.fill("Gossip");
        await expect(
          page.getByTestId(WISHLIST_DUPLICATE_WARNING_ERROR),
        ).toBeVisible();
        await expect(addSongButton).toBeDisabled();
      });

      test("New songs start uncategorized", async ({ page }) => {
        const title = `TestSong ${role} ${testRunId}`;
        await titleInput.fill(title);
        await artistInput.fill("TestInterpret");
        await addSongButton.click();

        const songline = getSongRow(page, title);
        await expect(songline).toBeVisible();
        await expect(songline.locator(".wishlist-category-badge")).toHaveCount(
          0,
        );
        await expect(songline.locator(".wishlist-label-badge")).toHaveCount(0);

        await songline.getByTestId("wishlist-edit").click();
        const editForm = page.getByTestId("edit-song-form");
        await editForm.getByTestId("wishlist-delete").click();
        await page.getByTestId("wishlist-confirm-delete").click();
      });

      test("Song suggestion saves its category and labels", async ({
        page,
      }) => {
        const title = `Metadata song ${role} ${testRunId}`;
        await titleInput.fill(title);
        await page.getByTestId("wishlist-create-metadata-open").click();
        const popup = page.getByTestId("wishlist-create-metadata-popup");
        await popup
          .getByTestId("wishlist-song-category-select")
          .selectOption(categoryId);
        for (const id of labelIds) {
          await popup.getByTestId(`wishlist-label-${id}`).check();
        }
        await popup.getByTestId("wishlist-create-metadata-done").click();
        await addSongButton.click();

        const songline = getSongRow(page, title);
        await expect(songline.locator(".wishlist-category-badge")).toHaveText(
          categoryName,
        );
        await expect(songline.locator(".wishlist-label-badge")).toHaveText(
          labelNames,
        );

        const itemId = await songline
          .getByTestId("wishlist-item")
          .getAttribute("data-song-id");
        if (!itemId) throw new Error("Created song has no data-song-id");
        createdSongs.push({ id: itemId, title });

        await songline.getByTestId("wishlist-edit").click();
        const editForm = page.getByTestId("edit-song-form");
        await editForm.getByTestId("wishlist-delete").click();
        await page.getByTestId("wishlist-confirm-delete").click();
      });

      test("Song metadata can be changed and cleared", async ({ page }) => {
        const title = `Editable metadata ${role} ${testRunId}`;
        await titleInput.fill(title);
        await addSongButton.click();
        const songline = getSongRow(page, title);
        const itemRow = songline.getByTestId("wishlist-item");
        const itemId = await itemRow.getAttribute("data-song-id");
        if (!itemId) throw new Error("Created song has no data-song-id");
        createdSongs.push({ id: itemId, title });

        await songline.getByTestId("wishlist-edit").click();
        const editForm = page.getByTestId("edit-song-form");
        const metadata = editForm.getByTestId("wishlist-metadata-details");
        await metadata.locator("summary").click();
        await editForm
          .getByTestId("wishlist-song-category-select")
          .selectOption(categoryId);
        await editForm.getByTestId(`wishlist-label-${labelIds[0]}`).check();
        await expect(
          editForm.getByTestId("wishlist-metadata-save"),
        ).toHaveCount(0);
        await editForm.getByTestId("wishlist-save-edit").click();
        const badges = itemRow.locator(".wishlist-song-summary");
        await expect(badges.locator(".wishlist-category-badge")).toHaveText(
          categoryName,
        );
        await expect(badges.locator(".wishlist-label-badge")).toHaveText(
          labelNames[0],
        );
        await expect(badges.locator(".wishlist-label-badge")).toHaveCSS(
          "background-color",
          "rgb(37, 99, 235)",
        );

        await songline.getByTestId("wishlist-edit").click();
        const clearMetadataForm = page.getByTestId("edit-song-form");
        await clearMetadataForm
          .getByTestId("wishlist-metadata-details")
          .locator("summary")
          .click();
        await clearMetadataForm
          .getByTestId("wishlist-song-category-select")
          .selectOption("");
        await clearMetadataForm
          .getByTestId(`wishlist-label-${labelIds[0]}`)
          .uncheck();
        await clearMetadataForm.getByTestId("wishlist-save-edit").click();
        await expect(badges.locator(".wishlist-category-badge")).toHaveCount(0);
        await expect(badges.locator(".wishlist-label-badge")).toHaveCount(0);

        await songline.getByTestId("wishlist-edit").click();
        const reopenedEditForm = page.getByTestId("edit-song-form");
        await reopenedEditForm.getByTestId("wishlist-delete").click();
        await page.getByTestId("wishlist-confirm-delete").click();
      });

      test("Wishlist item is well formatted", async ({ page }) => {
        const line = getSongRow(page, heavyCrossTitle);

        await expect(
          line.getByRole("heading", { name: heavyCrossTitle }),
        ).toBeVisible();
        await expect(line.getByText("Gossip")).toBeVisible();
        await expect(line.getByText("⁨120⁩ BPM")).toBeVisible();
        await expect(
          line.locator("span").filter({ hasText: "C major" }),
        ).toBeVisible();
        await expect(
          line.getByRole("link", { name: "Heavy Cross" }),
        ).toBeVisible();

        await expect(line.getByTestId("wishlist-edit")).toBeVisible();
        await expect(line.getByTestId("wishliste-feedback-edit")).toBeVisible();
      });

      test("category toggle includes uncategorized songs on mobile", async ({
        page,
      }) => {
        await page.setViewportSize({ width: 390, height: 844 });
        await page.getByTestId("filter-all-songs").click();
        const categoryToggle = page.getByTestId("wishlist-view-categories");
        await categoryToggle.check();

        await expect(page.getByTestId("wishlist-category-group")).toBeVisible();
        await expect(getSongRow(page, heavyCrossTitle)).toBeVisible();
        await expect(categoryToggle).toBeChecked();

        await categoryToggle.uncheck();
        await expect(categoryToggle).not.toBeChecked();
      });

      test("Adding feedback is possible", async ({ page }) => {
        const line = getSongRow(page, heavyCrossTitle);
        await line.getByTestId("wishliste-feedback-edit").click();

        const feedbackForm = line.getByTestId("wishlist-feedback-popup");
        const feedbackSelect = feedbackForm.getByTestId(
          "wishlist-feedback-select",
        );
        const feedbackSubmit = feedbackForm.getByTestId(
          "wishlist-feedback-save",
        );

        await expect(feedbackForm).toBeVisible();
        await expect(feedbackSelect).toBeVisible();
        await expect(feedbackSubmit).toBeVisible();
        await expect(feedbackSubmit).toBeDisabled();

        await feedbackSelect.selectOption("song");
        await expect(feedbackSubmit).toBeEnabled();
        await feedbackSubmit.click();

        await page.getByTestId("filter-feedback-needed").click();
        await expect(line).toBeHidden();

        await page.getByTestId("filter-all-songs").click();
        await expect(line).toBeVisible();

        await line.getByTestId("wishliste-feedback-edit").click();
        await expect(feedbackSubmit).toBeEnabled();
      });

      test("Editing song is possible", async ({ page }) => {
        const dummyTitle = `Dummy Title ${testRunId}`;
        await titleInput.fill(dummyTitle);
        await addSongButton.click();

        const songLine = getSongRow(page, dummyTitle);
        const editButton = songLine.getByTestId("wishlist-edit");
        await editButton.click();

        const editForm = page.getByTestId("edit-song-form");
        await expect(editForm).toBeVisible();
        await expect(
          songLine.getByTestId("wishlist-metadata-edit"),
        ).toHaveCount(0);
        const metadata = editForm.getByTestId("wishlist-metadata-details");
        await metadata.locator("summary").click();
        await expect(
          editForm.getByTestId("wishlist-song-category-select"),
        ).toBeVisible();

        const songArtistInputField = editForm.getByTestId("wishlist-artist");
        const saveButton = editForm.getByTestId("wishlist-save-edit");
        const cancelButton = editForm.getByTestId("wishlist-cancel-edit");
        const deleteButton = editForm.getByTestId("wishlist-delete");

        await expect(editForm.getByTestId("add-song-name")).toBeVisible();
        await expect(songArtistInputField).toBeVisible();
        await expect(editForm.getByTestId("wishlist-link")).toBeVisible();
        await expect(editForm.getByTestId("wishlist-tempo")).toBeVisible();
        await expect(editForm.getByTestId("wishlist-key")).toBeVisible();
        await expect(saveButton).toBeVisible();
        await expect(cancelButton).toBeVisible();
        await expect(deleteButton).toBeVisible();

        await songArtistInputField.fill("Dummy Artist");
        await saveButton.click();

        await expect(songLine.getByText("Dummy Artist")).toBeVisible();

        await editButton.click();
        await songArtistInputField.fill("Dummy Interpret");

        await cancelButton.click();
        await expect(songLine.getByText("Dummy Artist")).toBeVisible();
        await expect(songLine.getByText("Dummy Interpret")).toBeHidden();

        await editButton.click();
        await deleteButton.click();
        const deletePopup = page.getByTestId("delete-confirmation-popup");

        await expect(deletePopup).toBeVisible();
        await expect(deletePopup).toContainText(dummyTitle);
        const popupCancelButton = deletePopup.getByTestId(
          "wishlist-cancel-delete",
        );
        const popupDeleteButton = deletePopup.getByTestId(
          "wishlist-confirm-delete",
        );

        await expect(popupCancelButton).toBeVisible();
        await expect(popupDeleteButton).toBeVisible();
        await popupCancelButton.click();

        await expect(editForm).toBeVisible();
        await deleteButton.click();
        await popupDeleteButton.click();

        expect(songLine).toBeHidden();
      });

      test("only creators and admins can delete songs", async ({ page }) => {
        for (const song of createdSongs) {
          if (!song.role) continue;

          const songRow = getSongRow(page, song.title);
          await expect(songRow).toBeVisible();
          await songRow.getByTestId("wishlist-edit").click();

          const editForm = page.getByTestId("edit-song-form");
          const deleteButton = editForm.getByTestId("wishlist-delete");
          if (role === "Admin" || role === song.role) {
            await expect(deleteButton).toBeVisible();
          } else {
            await expect(deleteButton).toBeHidden();
            const deniedDelete = await page
              .context()
              .request.post("/api/wishlist_delete", {
                form: { item_id: song.id },
              });
            expect(deniedDelete.ok()).toBe(false);
          }
          await editForm.getByTestId("wishlist-cancel-edit").click();
        }
      });

      test("adding songs to medley is possible", async ({ page }) => {
        const medleySongs = createdSongs.filter((song) => song.role);
        const createMedleyButton = page.getByTestId("wishlist-create-medley");

        await expect(createMedleyButton).toBeVisible();
        await createMedleyButton.click();

        const createMedleyForm = page.getByTestId(
          "wishlist-medley-create-form",
        );
        await expect(createMedleyForm).toBeVisible();
        const medleyNameInput =
          createMedleyForm.getByTestId("medley-name-input");
        const saveMedleyButton = createMedleyForm.getByTestId("save-medley");
        const cancelMedleyButton =
          createMedleyForm.getByTestId("cancel-medley");
        await expect(medleyNameInput).toBeVisible();
        await expect(saveMedleyButton).toBeDisabled();
        await expect(cancelMedleyButton).toBeEnabled();

        await getSongRow(page, medleySongs[0].title)
          .getByTestId("song-added-to-medley-cb")
          .setChecked(true);
        await getSongRow(page, medleySongs[1].title)
          .getByTestId("song-added-to-medley-cb")
          .setChecked(true);
        await expect(saveMedleyButton).toBeDisabled();
        await medleyNameInput.fill(testMedleyName);
        await expect(saveMedleyButton).toBeEnabled();

        await getSongRow(page, medleySongs[2].title)
          .getByTestId("song-added-to-medley-cb")
          .setChecked(true);

        await saveMedleyButton.click();
        const medleyLine = page.locator(
          `[data-medley-name="${testMedleyName}"]`,
        );
        await expect(medleyLine).toBeVisible();
      });

      test("medley metadata uses the single save button", async ({ page }) => {
        const medleyLine = page.locator(
          `[data-medley-name="${testMedleyName}"]`,
        );
        await medleyLine.getByTestId("medley-manage-button").click();
        await medleyLine.getByTestId("medley-edit-button").click();
        await medleyLine
          .getByTestId("wishlist-metadata-details")
          .locator("summary")
          .click();
        await medleyLine
          .getByTestId("wishlist-song-category-select")
          .selectOption(categoryId);
        await medleyLine.getByTestId(`wishlist-label-${labelIds[1]}`).check();
        await expect(
          medleyLine.getByTestId("wishlist-metadata-save"),
        ).toHaveCount(0);
        await medleyLine.getByTestId("medley-save-button").click();
        await expect(medleyLine.locator(".wishlist-category-badge")).toHaveText(
          categoryName,
        );
        await expect(medleyLine.locator(".wishlist-label-badge")).toHaveText(
          labelNames[1],
        );
      });

      test("modifying medley is possible", async ({ page }) => {
        const medleySongs = createdSongs.filter((song) => song.role);
        const medleyLine = page.locator(
          `[data-medley-name="${testMedleyName}"]`,
        );
        const medleyManageButton = medleyLine.getByTestId(
          "medley-manage-button",
        );
        const medleyEditButton = medleyLine.getByTestId("medley-edit-button");
        const medleyDeleteButton = medleyLine.getByTestId(
          "medley-delete-button",
        );
        const medleySaveButton = medleyLine.getByTestId("medley-save-button");
        const medleyCancelButton = medleyLine.getByTestId(
          "medley-cancel-button",
        );

        await expect(medleyManageButton).toBeVisible();
        await medleyManageButton.click();

        await expect(medleyEditButton).toBeVisible();
        await expect(medleyDeleteButton).toBeVisible();

        const outsideTarget = page.locator("#wishlist-list-title");
        if (test.info().project.name === "mobile") {
          await outsideTarget.tap();
        } else {
          await outsideTarget.click();
        }
        await expect(medleyEditButton).toBeHidden();
        await expect(medleyDeleteButton).toBeHidden();

        await medleyManageButton.click();
        await medleyEditButton.click();
        const medleyNameInput = medleyLine.getByTestId("medley-name-edit");
        await expect(medleyNameInput).toBeVisible();
        await expect(medleySaveButton).toBeVisible();
        await expect(medleyCancelButton).toBeVisible();
        const medleyMetadata = medleyLine.getByTestId(
          "wishlist-metadata-details",
        );
        await expect(medleyMetadata).toBeVisible();
        await medleyMetadata.locator("summary").click();
        await expect(
          medleyLine.getByTestId("wishlist-song-category-select"),
        ).toBeVisible();
        await medleyMetadata.locator("summary").click();

        await medleyNameInput.fill("Another Name");
        await medleyCancelButton.click();

        await medleyManageButton.click();
        await medleyEditButton.click();

        const editRows = medleyLine.locator(".wishlist-medley-edit-row h3");
        await expect(editRows).toHaveText(
          medleySongs.map((song) => song.title),
        );

        const firstSong = getSongRow(page, medleySongs[0].title);
        const secondSong = getSongRow(page, medleySongs[1].title);
        const thirdSong = getSongRow(page, medleySongs[2].title);
        await expect(
          firstSong.getByTestId("medley-move-song-up-button"),
        ).toBeDisabled();
        await expect(
          thirdSong.getByTestId("medley-move-song-down-button"),
        ).toBeDisabled();

        await secondSong.getByTestId("medley-move-song-up-button").click();
        await expect(editRows).toHaveText([
          medleySongs[1].title,
          medleySongs[0].title,
          medleySongs[2].title,
        ]);
        await firstSong.getByTestId("medley-move-song-down-button").click();
        await expect(editRows).toHaveText([
          medleySongs[1].title,
          medleySongs[2].title,
          medleySongs[0].title,
        ]);

        await thirdSong.getByTestId("song-added-to-medley-cb").click();
        await expect(editRows).toHaveText([
          medleySongs[1].title,
          medleySongs[0].title,
        ]);
        await medleySaveButton.click();
        await expect(
          medleyLine.locator(".wishlist-medley-songs [data-song-name]"),
        ).toHaveCount(2);
        await expect(
          medleyLine.locator(".wishlist-medley-songs h3"),
        ).toHaveText([medleySongs[1].title, medleySongs[0].title]);
        await expect(
          medleyLine.locator(`[data-song-name="${medleySongs[2].title}"]`),
        ).toHaveCount(0);
        await expect(getSongRow(page, medleySongs[2].title)).toBeVisible();

        await medleyManageButton.click();
        await medleyDeleteButton.click();
        const deleteDialog = medleyLine.getByRole("dialog");
        await expect(deleteDialog).toBeVisible();
        await deleteDialog.getByTestId("medley-cancel-delete-button").click();
        await expect(deleteDialog).toBeHidden();
        await expect(medleyLine).toBeVisible();
        await expect(
          medleyLine.locator(".wishlist-medley-songs [data-song-name]"),
        ).toHaveCount(2);

        await medleyManageButton.click();
        await medleyDeleteButton.click();
        await expect(deleteDialog).toBeVisible();
        await deleteDialog.getByTestId("medley-confirm-delete-button").click();
        await expect(medleyLine).toHaveCount(0);
        await expect(getSongRow(page, medleySongs[0].title)).toBeVisible();
        await expect(getSongRow(page, medleySongs[1].title)).toBeVisible();
        await expect(getSongRow(page, medleySongs[2].title)).toBeVisible();
      });
    });
  }

  test.describe("Metadata picker", () => {
    test.use({ storageState: MEMBER_FILE });

    test.beforeEach(async ({ page }) => {
      await page.goto("/wishlist", { waitUntil: "networkidle" });
    });

    test("fits the mobile viewport", async ({ page }) => {
      await page.setViewportSize({ width: 390, height: 844 });
      await page.getByTestId("wishlist-create-metadata-open").click();
      const popup = page.getByTestId("wishlist-create-metadata-popup");
      await expect(
        page.getByTestId("wishlist-create-metadata-dialog"),
      ).toHaveAttribute("aria-modal", "true");

      const bounds = await popup
        .locator(".wishlist-metadata-dialog")
        .boundingBox();
      const viewportWidth = await page.evaluate(() => window.innerWidth);
      const viewportHeight = await page.evaluate(() => window.innerHeight);
      expect(bounds).not.toBeNull();
      expect(bounds!.x).toBeGreaterThanOrEqual(0);
      expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(viewportWidth);
      expect(bounds!.y).toBeGreaterThanOrEqual(0);
      expect(bounds!.y + bounds!.height).toBeLessThanOrEqual(viewportHeight);
    });

    test("closes with Escape", async ({ page }) => {
      await page.getByTestId("wishlist-create-metadata-open").click();
      await expect(
        page.getByTestId("wishlist-create-metadata-popup"),
      ).toBeVisible();
      await page.keyboard.press("Escape");
      await expect(
        page.getByTestId("wishlist-create-metadata-popup"),
      ).toBeHidden();
    });

    test("edit popup stays centered in the viewport", async ({ page }) => {
      const title = `AAA viewport popup ${testRunId}`;
      await page.getByTestId("add-song-name").fill(title);
      await page.getByTestId("create-song").click();
      const songRow = getSongRow(page, title);
      await expect(songRow).toBeVisible();
      const itemId = await songRow
        .getByTestId("wishlist-item")
        .getAttribute("data-song-id");
      if (!itemId) throw new Error("Created song has no data-song-id");
      createdSongs.push({ id: itemId, title });

      await page.locator("body > main").evaluate((main) => {
        main.style.minHeight = "500vh";
      });
      await songRow.getByTestId("wishlist-edit").click();

      const popup = page.getByTestId("edit-song-form");
      await expect(popup).toBeVisible();
      const bounds = await popup.locator(".wishlist-edit-form").boundingBox();
      const viewportHeight = await page.evaluate(() => window.innerHeight);

      expect(bounds).not.toBeNull();
      expect(
        Math.abs(bounds!.y + bounds!.height / 2 - viewportHeight / 2),
      ).toBeLessThan(20);
      await popup.getByTestId("wishlist-cancel-edit").click();
    });
  });

  test.describe("Category layout", () => {
    test.use({ storageState: MEMBER_FILE });

    test.beforeEach(async ({ page }) => {
      await page.goto("/wishlist", { waitUntil: "networkidle" });
    });

    test("pairs groups on desktop and centers an odd final group", async ({
      page,
    }) => {
      await page.locator("body > main").evaluate((main) => {
        const fixture = document.createElement("div");
        fixture.className = "wishlist-category-groups";
        fixture.dataset.testid = "category-layout-fixture";
        for (let index = 1; index <= 3; index += 1) {
          const group = document.createElement("section");
          group.className = "wishlist-category-group";
          group.textContent = `Category ${index}`;
          fixture.append(group);
        }
        main.append(fixture);
      });

      const fixture = page.getByTestId("category-layout-fixture");
      const groups = fixture.locator(".wishlist-category-group");
      const boxes = await groups.evaluateAll((elements) =>
        elements.map((element) => {
          const { x, y, width } = element.getBoundingClientRect();
          return { x, y, width };
        }),
      );
      const bounds = await fixture.boundingBox();
      const desktop = await page.evaluate(() => window.innerWidth >= 900);

      expect(bounds).not.toBeNull();
      expect(boxes).toHaveLength(3);
      if (desktop) {
        expect(Math.abs(boxes[0].y - boxes[1].y)).toBeLessThan(2);
        expect(boxes[0].x).toBeLessThan(boxes[1].x);
        expect(boxes[2].y).toBeGreaterThan(boxes[1].y);
        expect(
          Math.abs(
            boxes[2].x + boxes[2].width / 2 - (bounds!.x + bounds!.width / 2),
          ),
        ).toBeLessThan(2);
      } else {
        expect(Math.abs(boxes[0].x - boxes[1].x)).toBeLessThan(2);
        expect(Math.abs(boxes[1].x - boxes[2].x)).toBeLessThan(2);
        expect(boxes[0].y).toBeLessThan(boxes[1].y);
        expect(boxes[1].y).toBeLessThan(boxes[2].y);
      }
    });
  });
});
