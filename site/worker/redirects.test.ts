// Run with: node --test worker/redirects.test.ts
import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import { REDIRECT_PATHS, REDIRECT_RULES, resolveRedirect } from "./redirects.ts";

const url = (path: string) => new URL(path, "https://monoxus.com");

const EXPECTED: Array<[string, number, string]> = [
	["/docs", 308, "/docs/overview/introduction"],
	["/docs/", 308, "/docs/overview/introduction"],
	["/docs/introduction", 308, "/docs/overview/introduction"],
	["/docs/getting-started", 308, "/docs/overview/getting-started"],
	["/docs/roadmap", 308, "/docs/overview/introduction"],
	["/docs/overview/roadmap", 308, "/docs/overview/introduction"],
	["/docs/overview/roadmap/", 308, "/docs/overview/introduction"],
	["/docs/components", 308, "/docs/components/button"],
];

for (const [from, status, to] of EXPECTED) {
	test(`${from} -> ${status} ${to}`, () => {
		assert.deepEqual(resolveRedirect(url(from)), { location: to, status });
	});
}

test("preserves query strings", () => {
	assert.deepEqual(resolveRedirect(url("/docs?tab=api")), {
		location: "/docs/overview/introduction?tab=api",
		status: 308,
	});
	assert.deepEqual(resolveRedirect(url("/docs/components/?q=dialog")), {
		location: "/docs/components/button?q=dialog",
		status: 308,
	});
});

test("leaves real pages and markdown twins alone", () => {
	for (const path of [
		"/",
		"/docs/overview/introduction",
		"/docs/overview/getting-started",
		"/docs/components/button",
		"/docs/overview/introduction.md",
		"/llms.txt",
		"/search-index.json",
	]) {
		assert.equal(resolveRedirect(url(path)), null, path);
	}
});

test("wrangler.jsonc run_worker_first covers every redirect rule", () => {
	const config = fs.readFileSync(new URL("../wrangler.jsonc", import.meta.url), "utf8");
	for (const prefix of REDIRECT_PATHS) {
		assert.ok(config.includes(`"${prefix}"`), `wrangler.jsonc run_worker_first lacks ${prefix}`);
	}

	const covered = (path: string) =>
		REDIRECT_PATHS.some((prefix) =>
			prefix.endsWith("/*")
				? path === prefix.slice(0, -2) || path.startsWith(prefix.slice(0, -1))
				: path === prefix,
		);

	for (const rule of REDIRECT_RULES) {
		assert.ok(covered(rule.from), `${rule.from} is not covered by run_worker_first`);
	}
});
