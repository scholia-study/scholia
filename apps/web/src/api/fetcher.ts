import { createIsomorphicFn } from "@tanstack/react-start";
import { getRequestHeader } from "@tanstack/react-start/server";
import config from "../config";

/** API origin. In the browser this comes from the runtime profile registry
 *  (same-origin "" in prod, http://localhost:4000 locally). On the SSR server
 *  it comes from env — set `API_BASE_URL` to the in-cluster Rust API URL when
 *  running in k3s; defaults to http://localhost:4000 for local dev. */
const getApiBaseUrl = createIsomorphicFn()
    .server(() => process.env.API_BASE_URL ?? "http://localhost:4000")
    .client(() => config.API_BASE_URL);

/** Forward the incoming request's Cookie header to the API so authenticated
 *  pages render correctly during SSR. The browser handles this automatically
 *  via `credentials: "include"`; on the server we have to do it by hand. */
const getRequestCookie = createIsomorphicFn()
    .server(() => getRequestHeader("cookie") ?? "")
    .client(() => "");

const getBody = <T>(c: Response | Request): Promise<T> => {
    const contentType = c.headers.get("content-type");

    if (contentType?.includes("application/json")) {
        return c.json();
    }

    if (contentType?.includes("application/pdf")) {
        return c.blob() as Promise<T>;
    }

    return c.text() as Promise<T>;
};

export const customFetch = async <T>(
    url: string,
    options?: RequestInit,
): Promise<T> => {
    const fullUrl = url.startsWith("http") ? url : `${getApiBaseUrl()}${url}`;

    const cookie = getRequestCookie();

    const res = await fetch(fullUrl, {
        ...options,
        credentials: "include",
        headers: {
            ...options?.headers,
            ...(cookie ? { cookie } : {}),
        },
    });

    if (!res.ok) {
        const text = await res.text();
        throw new FetchError(text || res.statusText, res.status);
    }

    const data = [204, 205, 304].includes(res.status)
        ? {}
        : await getBody<T>(res);

    return { data, status: res.status, headers: res.headers } as T;
};

export class FetchError extends Error {
    constructor(
        message: string,
        public status: number,
    ) {
        super(message);
    }
}

export default customFetch;
