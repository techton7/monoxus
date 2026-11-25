/**
 * Edge worker for monoxus.com.
 *
 * Runs only for paths listed in `wrangler.jsonc` `assets.run_worker_first`:
 * 1. Resolves canonical `/docs` redirects (`worker/redirects.ts`).
 * 2. Negotiates Markdown (`Accept: text/markdown` or `text/x-markdown`) on
 *    extensionless `/docs/*` routes by serving `<pathname>.md` from `ASSETS`.
 * 3. Falls through to `env.ASSETS.fetch(request)` for all other requests.
 */

import { resolveRedirect } from "./redirects.ts";

interface Env {
	ASSETS: { fetch(request: Request): Promise<Response> };
}

const MARKDOWN_TYPES = ["text/markdown", "text/x-markdown"];

const worker = {
	async fetch(request: Request, env: Env): Promise<Response> {
		const url = new URL(request.url);

		const redirect = resolveRedirect(url);
		if (redirect) {
			return new Response(null, {
				status: redirect.status,
				headers: { Location: redirect.location },
			});
		}

		if (
			(request.method === "GET" || request.method === "HEAD") &&
			url.pathname.startsWith("/docs/")
		) {
			const accept = request.headers.get("accept") ?? "";
			const lastSegment = url.pathname.slice(url.pathname.lastIndexOf("/") + 1);
			const wantsMarkdown = MARKDOWN_TYPES.some((type) => accept.includes(type));

			if (wantsMarkdown && lastSegment !== "" && !lastSegment.includes(".")) {
				const markdownUrl = new URL(url);
				markdownUrl.pathname = `${url.pathname.replace(/\/+$/, "")}.md`;
				const markdown = await env.ASSETS.fetch(new Request(markdownUrl, request));
				if (markdown.ok) {
					const headers = new Headers(markdown.headers);
					headers.set("Content-Type", "text/markdown; charset=utf-8");
					if (!headers.get("Vary")?.toLowerCase().includes("accept")) {
						headers.append("Vary", "Accept");
					}
					return new Response(markdown.body, {
						status: markdown.status,
						headers,
					});
				}
			}
		}

		return env.ASSETS.fetch(request);
	},
};

export default worker;
