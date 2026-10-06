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
    await expect(page.locator("html")).toHaveAttribute("lang", "de");

    await page.getByTestId("lang-en").click();
    await expect(page.getByTestId("login-title")).toHaveText("Who are you?");
    await expect(page.locator("html")).toHaveAttribute("lang", "en");
  });

  test("keeps the selected language across route loads and reloads", async ({
    page,
  }) => {
    await gotoHydrated(page, "/");

    await page.getByTestId("lang-de").click();
    await expect(page.locator("h1")).toHaveText("Programm");
    await expect(page.locator("html")).toHaveAttribute("lang", "de");
    await expect.poll(() => page.evaluate(() => localStorage.getItem("lang"))).toBe("de");
    await expect.poll(async () =>
      (await page.context().cookies()).find((cookie) => cookie.name === "lf-lang")
        ?.value,
    ).toBe("de");

    await page.reload({ waitUntil: "networkidle" });
    await expect(page.locator("h1")).toHaveText("Programm");
    await expect(page.locator("html")).toHaveAttribute("lang", "de");

    await page.getByTestId("lang-en").click();
    await expect(page.locator("h1")).toHaveText("Program");
  });

  test("restores a stored language when no language cookie exists", async ({
    page,
  }) => {
    await page.addInitScript(() => localStorage.setItem("lang", "de"));
    await gotoHydrated(page, "/");

    await expect(page.locator("h1")).toHaveText("Programm");
    await expect(page.locator("html")).toHaveAttribute("lang", "de");
  });
});
