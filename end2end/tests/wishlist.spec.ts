import {
  test,
  expect,
  APIRequestContext,
  Locator,
  Page,
} from "@playwright/test";
import { ADMIN_FILE, MEMBER_FILE, PARTICIPANT_FILE } from "./helpers";

const WISHLIST_DUPLICATE_WARNING_ERROR = "wishlist-duplicate-warning";

test.describe("No user logged in", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/wishlist", { waitUntil: "networkidle" });
    await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  });

  test("page should not be accessible", async ({ page }) => {
    await expect(
      page
        .getByRole("main")
        .getByRole("link", { name: /^(Anmelden|Sign In)$/i }),
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
      throw new Error(`Could not create song: ${JSON.stringify(songInfo)}`);
    }

    createdSongs.push(await response.json());
  }

  async function getSongRow(page: Page, songTitle: string): Promise<Locator> {
    return page.locator(`[data-song-name="${songTitle}"]`);
  }

  test.beforeAll(async ({ request }) => {
    await createSong(request, {
      title: "Heavy Cross",
      artist: "Gossip",
      link: "https://www.youtube.com/watch?v=tjc36rR4Mq0",
      tempo: "120",
      musical_key: "C major",
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
      await titleInput.fill("Heavy Cross");
      await expect(
        page.getByTestId(WISHLIST_DUPLICATE_WARNING_ERROR),
      ).toBeVisible();
      await expect(addSongButton).toBeEnabled();
    });

    test("Add song disabeled when song already in wishlist", async ({
      page,
    }) => {
      await titleInput.fill("Heavy Cross");
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

      const songline = await getSongRow(page, "TestSong");

      // Delete it again
      await songline.getByTestId("wishlist-edit").click();
      await songline.getByTestId("wishlist-delete").click();
      await songline.getByTestId("wishlist-confirm-delete").click();
    });

    test("Wishlist item is well formatted", async ({ page }) => {
      const line = await getSongRow(page, "Heavy Cross");

      await expect(
        line.getByRole("heading", { name: "Heavy Cross" }),
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
      const line = await getSongRow(page, "Heavy Cross");
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

      const songLine = await getSongRow(page, "Dummy Title");
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
