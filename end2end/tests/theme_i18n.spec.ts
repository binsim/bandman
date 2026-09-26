import { expect, test } from "@playwright/test";
import { gotoHydrated } from "./helpers";

test.describe("theme and language", () => {
  test("toggles dark and light theme", async ({ page }) => {
    await gotoHydrated(page, "/");
    const html = page.locator("html");

    const first = await html.getAttribute("data-theme");
    expect(first === "light" || first === "dark").toBeTruthy();

    await page.getByTestId("theme-toggle").click();
    const second = await html.getAttribute("data-theme");
    expect(second).not.toEqual(first);
    expect(second === "light" || second === "dark").toBeTruthy();

    await page.getByTestId("theme-toggle").click();
    await expect(html).toHaveAttribute("data-theme", first!);
  });

  test("switches UI language to German and back", async ({ page }) => {
    await gotoHydrated(page, "/login");

    await page.getByTestId("lang-de").click();
    await expect(page.getByTestId("login-title")).toHaveText("Wer bist du?");

    await page.getByTestId("lang-en").click();
    await expect(page.getByTestId("login-title")).toHaveText("Who are you?");
  });
});
