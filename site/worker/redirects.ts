/**
 * Edge URL redirects for monoxus.com.
 *
 * Kept in Worker code rather than `_redirects` so rule order is deterministic
 * and covered by unit tests alongside `wrangler.jsonc` `run_worker_first`.
 * Query strings are preserved.
 */

export type RedirectRule = {
	/** Exact pathname (without trailing slash) to match. */
	from: string;
	/** Destination pathname. */
	to: string;
	status: 307 | 308;
};

export const REDIRECT_RULES: readonly RedirectRule[] = [
	{ from: "/docs", to: "/docs/overview/introduction", status: 308 },
	{ from: "/docs/introduction", to: "/docs/overview/introduction", status: 308 },
	{ from: "/docs/getting-started", to: "/docs/overview/getting-started", status: 308 },
	{ from: "/docs/components", to: "/docs/components/button", status: 308 },
];

/**
 * Path patterns that `wrangler.jsonc` `assets.run_worker_first` must include
 * so every rule above reaches the Worker.
 */
export const REDIRECT_PATHS = ["/docs", "/docs/*"] as const;

export type Redirect = {
	location: string;
	status: 307 | 308;
};

/**
 * Returns the redirect for a request URL, or `null` when no rule matches.
 */
export function resolveRedirect(url: URL): Redirect | null {
	const pathname = url.pathname.replace(/\/+$/, "") || "/";
	for (const rule of REDIRECT_RULES) {
		if (rule.from === pathname) {
			return {
				location: `${rule.to}${url.search}`,
				status: rule.status,
			};
		}
	}
	return null;
}
