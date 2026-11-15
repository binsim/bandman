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
const medleySong1 = `Medley Song 1 ${testRunId}`;
const medleySong2 = `Medley Song 2 ${testRunId}`;
const medleySong3 = `Medley Song 3 ${testRunId}`;
const testMedleyName = `Test Medley ${testRunId}`;

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

  const createdSongs: any[] = [];

  async function createSong(
    request: APIRequestContext,
    songInfo: {
      title: string;
      artist: string;
      link: string;
      tempo: string;
      musical_key: string;
    },
  ) {
    const response = await request.post("/api/wishlist_create", {
      form: songInfo,
    });
    if (!response.ok()) {
      throw new Error(
        `Could not create song: ${JSON.stringify(songInfo)} due to ${await response.text()}`,
      );
    }

    createdSongs.push(await response.json());
  }

  function getSongRow(page: Page, songTitle: string): Locator {
    return page.locator(`[data-song-name="${songTitle}"]`).locator("xpath=..");
  }

  test.beforeAll(async ({ request }) => {
    await createSong(request, {
      title: heavyCrossTitle,
      artist: "Gossip",
      link: "https://www.youtube.com/watch?v=tjc36rR4Mq0",
      tempo: "120",
      musical_key: "C major",
    });
    await createSong(request, {
      title: medleySong1,
      artist: "",
      link: "",
      tempo: "",
      musical_key: "",
    });
    await createSong(request, {
      title: medleySong2,
      artist: "",
      link: "",
      tempo: "",
      musical_key: "",
    });
    await createSong(request, {
      title: medleySong3,
      artist: "",
      link: "",
      tempo: "",
      musical_key: "",
    });
  });
  test.afterAll(async ({ request }) => {
    await Promise.all(
      createdSongs.map((song) => {
        return request.post("/api/wishlist_delete", {
          form: { item_id: song.id },
        });
      }),
    );
  });

  test.describe("Member user logged in", () => {
    test.use({ storageState: MEMBER_FILE });
    test.beforeEach(async ({ page }) => {
      await page.goto("/wishlist", { waitUntil: "networkidle" });
      await page
        .locator("[data-testid='topbar']")
        .waitFor({ state: "visible" });
    });
  });

  test.describe("Participant user logged in", () => {
    test.use({ storageState: PARTICIPANT_FILE });
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

    test("Adding song is possible", async ({ page }) => {
      await titleInput.fill("TestSong");
      await artistInput.fill("TestInterpret");
      await addSongButton.click();

      const songline = getSongRow(page, "TestSong");

      // Delete it again
      await songline.getByTestId("wishlist-edit").click();
      await songline.getByTestId("wishlist-delete").click();
      await songline.getByTestId("wishlist-confirm-delete").click();
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

    test("Adding feedback is possible", async ({ page }) => {
      const line = getSongRow(page, heavyCrossTitle);
      await line.getByTestId("wishliste-feedback-edit").click();

      const feedbackForm = line.getByTestId("wishlist-feedback-popup");
      const feedbackSelect = feedbackForm.getByTestId(
        "wishlist-feedback-select",
      );
      const feedbackSubmit = feedbackForm.getByTestId("wishlist-feedback-save");

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
      await titleInput.fill("Dummy Title");
      await addSongButton.click();

      const songLine = getSongRow(page, "Dummy Title");
      const editButton = songLine.getByTestId("wishlist-edit");
      await editButton.click();

      const editForm = songLine.getByTestId("edit-song-form");
      await expect(editForm).toBeVisible();

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
      const deletePopup = songLine.getByTestId("delete-confirmation-popup");

      await expect(deletePopup).toBeVisible();
      await expect(deletePopup).toContainText("Dummy Title");
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

    test("adding songs to medley is possible", async ({ page }) => {
      const createMedleyButton = page.getByTestId("wishlist-create-medley");

      await expect(createMedleyButton).toBeVisible();
      await createMedleyButton.click();

      const createMedleyForm = page.getByTestId("wishlist-medley-create-form");
      await expect(createMedleyForm).toBeVisible();
      const medleyNameInput = createMedleyForm.getByTestId("medley-name-input");
      const saveMedleyButton = createMedleyForm.getByTestId("save-medley");
      const cancelMedleyButton = createMedleyForm.getByTestId("cancel-medley");
      await expect(medleyNameInput).toBeVisible();
      await expect(saveMedleyButton).toBeDisabled();
      await expect(cancelMedleyButton).toBeEnabled();

      await getSongRow(page, medleySong1)
        .getByTestId("song-added-to-medley-cb")
        .setChecked(true);
      await getSongRow(page, medleySong2)
        .getByTestId("song-added-to-medley-cb")
        .setChecked(true);
      await expect(saveMedleyButton).toBeDisabled();
      await medleyNameInput.fill(testMedleyName);
      await expect(saveMedleyButton).toBeEnabled();

      await getSongRow(page, medleySong3)
        .getByTestId("song-added-to-medley-cb")
        .setChecked(true);

      await saveMedleyButton.click();
      const medleyLine = page.locator(`[data-medley-name="${testMedleyName}"]`);
      await expect(medleyLine).toBeVisible();
    });

    test("modifying medley is possible", async ({ page }) => {
      const medleyLine = page.locator(`[data-medley-name="${testMedleyName}"]`);
      const medleyManageButton = medleyLine.getByTestId("medley-manage-button");
      const medleyEditButton = medleyLine.getByTestId("medley-edit-button");
      const medleyDeleteButton = medleyLine.getByTestId("medley-delete-button");
      const medleySaveButton = medleyLine.getByTestId("medley-save-button");
      const medleyCancelButton = medleyLine.getByTestId("medley-cancel-button");

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

      await medleyNameInput.fill("Another Name");
      await medleyCancelButton.click();

      await medleyManageButton.click();
      await medleyEditButton.click();

      const editRows = medleyLine.locator(".wishlist-medley-edit-row h3");
      await expect(editRows).toHaveText([
        medleySong1,
        medleySong2,
        medleySong3,
      ]);

      const firstSong = getSongRow(page, medleySong1);
      const secondSong = getSongRow(page, medleySong2);
      const thirdSong = getSongRow(page, medleySong3);
      await expect(
        firstSong.getByTestId("medley-move-song-up-button"),
      ).toBeDisabled();
      await expect(
        thirdSong.getByTestId("medley-move-song-down-button"),
      ).toBeDisabled();

      await secondSong.getByTestId("medley-move-song-up-button").click();
      await expect(editRows).toHaveText([
        medleySong2,
        medleySong1,
        medleySong3,
      ]);
      await firstSong.getByTestId("medley-move-song-down-button").click();
      await expect(editRows).toHaveText([
        medleySong2,
        medleySong3,
        medleySong1,
      ]);

      await thirdSong.getByTestId("song-added-to-medley-cb").click();
      await expect(editRows).toHaveText([medleySong2, medleySong1]);
      await medleySaveButton.click();
      await expect(
        medleyLine.locator(".wishlist-medley-songs [data-song-name]"),
      ).toHaveCount(2);
      await expect(medleyLine.locator(".wishlist-medley-songs h3")).toHaveText([
        medleySong2,
        medleySong1,
      ]);
      await expect(
        medleyLine.locator(`[data-song-name="${medleySong3}"]`),
      ).toHaveCount(0);
      await expect(getSongRow(page, medleySong3)).toBeVisible();

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
      await expect(getSongRow(page, medleySong1)).toBeVisible();
      await expect(getSongRow(page, medleySong2)).toBeVisible();
      await expect(getSongRow(page, medleySong3)).toBeVisible();
    });
  });

  test.describe("Admin user logged in", () => {
    test.use({ storageState: ADMIN_FILE });
    test.beforeEach(async ({ page }) => {
      await page.goto("/wishlist", { waitUntil: "networkidle" });
      await page
        .locator("[data-testid='topbar']")
        .waitFor({ state: "visible" });
    });
  });
});
