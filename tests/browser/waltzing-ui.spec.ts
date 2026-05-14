import { AxeBuilder } from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

const blocks = [
  { id: "login-card", title: "Login Card" },
  { id: "signup-card", title: "Signup Card" },
  { id: "contact-card", title: "Contact Card" },
  { id: "stats-grid", title: "Stats Grid" },
];

test.describe("waltzing-ui browser regression", () => {
  test("library page exposes components, layouts, and blocks", async ({ page }) => {
    await goto(page, "/library/waltzing-ui");

    await expect(page.getByRole("heading", { name: "waltzing-ui" })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^Components \(/ })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^Layouts \(/ })).toBeVisible();
    await expect(page.getByRole("heading", { name: /^Blocks \(/ })).toBeVisible();

    for (const block of blocks) {
      await expect(page.getByRole("link", { name: block.title }).first()).toBeVisible();
    }
  });

  for (const block of blocks) {
    test(`${block.id} route renders without browser or axe regressions`, async ({ page }) => {
      const errors = collectBrowserErrors(page);

      await goto(page, `/library/waltzing-ui/block/${block.id}`);

      await expect(page.getByRole("heading", { name: block.title })).toBeVisible();
      await expect(page.getByRole("heading", { name: "Preview", exact: true })).toBeVisible();
      await expect(page.getByRole("heading", { name: "Usage", exact: true })).toBeVisible();
      await expect(page.locator("#main-content")).toBeVisible();
      await expect(page.locator("#block-preview")).toBeVisible();

      await assertNoSeriousA11yViolations(page);
      expect(errors, "browser console/page errors").toEqual([]);
    });
  }

  test("auth and contact block pages expose accessible form controls", async ({ page }) => {
    for (const block of ["login-card", "signup-card", "contact-card"]) {
      await goto(page, `/library/waltzing-ui/block/${block}`);

      const preview = page.locator("#block-preview");
      const textboxes = preview.getByRole("textbox");

      await expect(textboxes.first()).toBeVisible();
      await expect(preview.locator("label")).not.toHaveCount(0);
      await assertNoSeriousA11yViolations(page);
    }
  });
});

async function goto(page: Page, path: string) {
  const response = await page.goto(path, { waitUntil: "domcontentloaded" });
  expect(response?.ok(), `${path} should return a successful response`).toBe(true);
}

function collectBrowserErrors(page: Page) {
  const errors: string[] = [];

  page.on("pageerror", (error) => errors.push(error.message));
  page.on("console", (message) => {
    if (message.type() === "error") {
      errors.push(message.text());
    }
  });

  return errors;
}

async function assertNoSeriousA11yViolations(page: Page) {
  const results = await new AxeBuilder({ page })
    .include("#block-preview")
    .disableRules([
      // The showcase embeds example source code and inline SVG snippets; those
      // are audited by source tests instead of browser landmark rules.
      "landmark-one-main",
      "page-has-heading-one",
    ])
    .analyze();

  const serious = results.violations.filter((violation) =>
    ["serious", "critical"].includes(violation.impact ?? ""),
  );

  expect(
    serious.map((violation) => ({
      id: violation.id,
      impact: violation.impact,
      nodes: violation.nodes.map((node) => node.target),
    })),
  ).toEqual([]);
}
