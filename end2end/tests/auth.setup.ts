import { test as setup, BrowserContext } from "@playwright/test";
import {
  ADMIN_FILE,
  MEMBER_FILE,
  MEMBER_USERNAME,
  PARTICIPANT_FILE,
  PARTICIPANT_USERNAME,
} from "./helpers";

export async function loginAsMember(context: BrowserContext, username: string) {
  const response = await context.request.post("/api/login_as_member", {
    form: { name: username },
  });
  if (!response.ok()) {
    throw new Error(
      `Login failed: ${response.status()} ${await response.text()}`,
    );
  }
}
async function createMember(
  context: BrowserContext,
  role: "member" | "participant",
) {
  const response = await context.request.post("/api/admin_create_member", {
    form: {
      name: role == "member" ? MEMBER_USERNAME : PARTICIPANT_USERNAME,
      role,
    },
  });
  if (!response.ok()) {
    var errorMessage = await response.text();
    if (errorMessage === "A member with this name already exists") {
      return;
    }
    throw new Error(
      `Creating member failed: ${response.status()} ${errorMessage}`,
    );
  }
}

setup("authenticate as admin", async ({ browser, page }) => {
  var context = await browser.newContext();
  await loginAsMember(context, "admin");

  await context.storageState({ path: ADMIN_FILE });
  context.close();

  await page.goto("/", { waitUntil: "networkidle" });
  await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
});

setup("authenticate as participant", async ({ browser, page }) => {
  var context = await browser.newContext();
  await loginAsMember(context, "admin");
  await context.storageState({ path: PARTICIPANT_FILE });
  await createMember(context, "participant");
  await loginAsMember(context, PARTICIPANT_USERNAME);

  await context.storageState({ path: PARTICIPANT_FILE });
  context.close();

  await page.goto("/", { waitUntil: "networkidle" });
  await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
});

setup("authenticate as member", async ({ browser, page }) => {
  var context = await browser.newContext();
  await loginAsMember(context, "admin");
  await context.storageState({ path: MEMBER_FILE });

  await page.goto("/", { waitUntil: "networkidle" });
  await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
  await createMember(context, "member");
  await loginAsMember(context, MEMBER_USERNAME);

  await context.storageState({ path: MEMBER_FILE });
  context.close();
  await page.goto("/", { waitUntil: "networkidle" });
  await page.locator("[data-testid='topbar']").waitFor({ state: "visible" });
});
