const BASE_URL = "http://localhost:4000";

export const customFetch = async <T>(
    url: string,
    options?: RequestInit,
): Promise<T> => {
    const fullUrl = url.startsWith("http") ? url : `${BASE_URL}${url}`;

    const res = await fetch(fullUrl, {
        ...options,
        credentials: "include",
        headers: {
            ...options?.headers,
        },
    });

    if (!res.ok) {
        const text = await res.text();
        throw new FetchError(text || res.statusText, res.status);
    }

    const body = [204, 205, 304].includes(res.status) ? null : await res.text();
    const data = body ? JSON.parse(body) : {};

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
