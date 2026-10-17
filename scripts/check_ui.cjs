#!/usr/bin/env node
// Requires Playwright externally; starts an isolated world and never touches live services.
const { chromium } = require(process.env.CURIOUS_PLAYWRIGHT || "playwright");
const { spawn } = require("node:child_process");
const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const net = require("node:net");
const assert = require("node:assert/strict");
(async () => {
  const tmp = await fs.mkdtemp(path.join(os.tmpdir(), "curious-ui-"));
  const socket = net.createServer();
  await new Promise((r) => socket.listen(0, "127.0.0.1", r));
  const port = socket.address().port;
  await new Promise((r) => socket.close(r));
  const base = `http://127.0.0.1:${port}`,
    token = "curious-ui-integration-only";
  const child = spawn(
    path.resolve(process.argv[2] || "target/release/curious"),
    [
      "--serve",
      "--paused",
      "--shard-mb",
      "1",
      "--scenario",
      path.resolve(process.argv[3] || "scenarios/meadow.json"),
      "--data-dir",
      tmp,
      "--bind",
      `127.0.0.1:${port}`,
    ],
    { env: { ...process.env, CURIOUS_TOKEN: token }, stdio: "ignore" },
  );
  let browser;
  try {
    const request = async (body) => {
      const r = await fetch(base + (body ? "/control" : "/status"), {
        method: body ? "POST" : "GET",
        headers: {
          Authorization: "Bearer " + token,
          "Content-Type": "application/json",
        },
        body: body ? JSON.stringify(body) : undefined,
      });
      assert.equal(r.status, 200);
      return r.json();
    };
    for (let i = 0; i < 100; i++) {
      try {
        await request();
        break;
      } catch {
        await new Promise((r) => setTimeout(r, 50));
      }
    }
    await request({ action: "step", ticks: 100 });
    browser = await chromium.launch({ headless: true, args: ["--no-sandbox"] });
    const page = await browser.newPage({
      viewport: { width: 1440, height: 1100 },
    });
    const errors = [];
    page.on("pageerror", (error) => errors.push(error.message));
    await page.goto(base);
    await page.locator("#auth").waitFor({ state: "visible" });
    await page.locator("#token").fill(token);
    await page.locator("#auth button").click();
    await page.waitForFunction(
      () => document.querySelector("#tick").textContent === "100",
    );
    await page.locator("#roster button").first().click();
    await page.locator("#individual-body").waitFor({ state: "visible" });
    await page.locator("#step").click();
    await page.waitForFunction(
      () => document.querySelector("#tick").textContent === "101",
    );
    await page.locator("#history").click();
    await page.waitForFunction(() =>
      document.querySelector("#journal-note").textContent.includes("Kalıcı"),
    );
    await page.locator("#full").click();
    await page.waitForFunction(() => !document.querySelector("#full").disabled);
    const output = path.resolve(
      process.env.CURIOUS_UI_OUTPUT || "data/ui-validation",
    );
    await fs.mkdir(output, { recursive: true });
    await page.screenshot({
      path: path.join(output, "desktop.png"),
      fullPage: true,
    });
    await page.setViewportSize({ width: 390, height: 844 });
    await page.screenshot({
      path: path.join(output, "mobile.png"),
      fullPage: true,
    });
    assert.equal(
      await page.evaluate(
        () => document.documentElement.scrollWidth > innerWidth,
      ),
      false,
    );
    await page.locator("#speed").selectOption("1");
    await page.waitForFunction(
      () => !document.querySelector("#speed").disabled,
    );
    await page.locator("#toggle").click();
    await page.waitForFunction(() =>
      document.querySelector("#toggle").textContent.includes("Duraklat"),
    );
    await page.waitForFunction(
      () => Number(document.querySelector("#tick").textContent) > 101,
    );
    await page.locator("#toggle").click();
    await page.waitForFunction(() =>
      document.querySelector("#toggle").textContent.includes("Başlat"),
    );
    // Founders have left the current world: selection must recover their archive.
    await request({ action: "step", ticks: 60 });
    await request({ action: "step", ticks: 60 });
    const inventory = await fetch(base + "/databases", {headers: {Authorization: "Bearer " + token}}).then(r => r.json());
    assert.ok(inventory.some(s => s.compressed), "genealogy must traverse compressed archives");
    await page.locator("#archive-id").fill("1");
    await page.locator("#archive-search button").click();
    await page.waitForFunction(() => !document.querySelector("#archive-load").disabled);
    for (let i = 0; i < 40; i++) {
      if (await page.locator("#individual-sub").textContent().then(t => t.includes("arşiv görüntüsü"))) break;
      if (await page.locator("#archive-more").isHidden()) break;
      await page.locator("#archive-more").click();
      await page.waitForFunction(() => !document.querySelector("#archive-load").disabled);
    }
    assert.match(await page.locator("#individual-sub").textContent(), /arşiv görüntüsü/);
    assert.equal(await page.locator("#full").isDisabled(), true);
    assert.equal(await page.locator("#follow").isDisabled(), true);
    await page.waitForTimeout(1200); // Live observation must preserve historical selection.
    assert.match(await page.locator("#individual-title").textContent(), /#1$/);
    for (let i = 0; i < 40 && !await page.locator("#family-children button").count(); i++) {
      if (await page.locator("#archive-more").isHidden()) break;
      await page.locator("#archive-more").click();
      await page.waitForFunction(() => !document.querySelector("#archive-load").disabled);
    }
    assert.ok(await page.locator("#family-children button").count(), "archived parent links to offspring");
    await page.locator("#family-children button").first().click();
    await page.waitForFunction(() => !document.querySelector("#archive-load").disabled);
    assert.ok(await page.locator("#family-parents button").count(), "offspring links back to parents");
    await page.setViewportSize({ width: 1440, height: 1100 });
    await page.screenshot({ path: path.join(output, "lineage-desktop.png"), fullPage: true });
    await page.setViewportSize({ width: 390, height: 844 });
    await page.screenshot({ path: path.join(output, "lineage-mobile.png"), fullPage: true });
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false);
    assert.deepEqual(errors, []);
    console.log(
      "Browser: auth, coherent observation, selection, history, step, speed, pause/resume, mobile overflow, deceased ancestor and offspring navigation: OK",
    );
  } finally {
    if (browser) await browser.close();
    child.kill("SIGKILL");
    await new Promise((r) =>
      child.exitCode !== null ? r() : child.once("exit", r),
    );
    await fs.rm(tmp, { recursive: true, force: true });
  }
})().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
