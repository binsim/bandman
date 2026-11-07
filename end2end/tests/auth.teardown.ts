import { test } from "@playwright/test";
import { ADMIN_FILE, MEMBER_USERNAME, PARTICIPANT_USERNAME } from "./helpers";
import { UUID } from "node:crypto";

test("remove test users", async ({ browser }) => {
  var context = await browser.newContext();
  const response = await context.request.post("/api/login_as_member", {
    form: { name: "admin" },
  });
  if (!response.ok()) {
    throw new Error(
      `Login failed: ${response.status()} ${await response.text()}`,
    );
  }

  await context.storageState({ path: ADMIN_FILE });

  const repsonseJson = await (
    await context.request.post("/api/admin_list_members")
  ).json();
  await Promise.all(
    repsonseJson.map((user: { name: string; id: UUID }) => {
      if (user.name !== PARTICIPANT_USERNAME && user.name !== MEMBER_USERNAME) {
        return;
      }

      return context.request.post("/api/admin_delete_member", {
        form: { member_id: user.id },
      });
    }),
  );
});
