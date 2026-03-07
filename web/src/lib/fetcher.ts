import type { AxiosRequestConfig } from "axios";
import axios from "axios";

function getBaseUrl(): string {
    if (typeof window === "undefined") {
        return process.env.API_URL || "http://localhost:4000";
    }
    return "";
}

// export const customFetch = async <T>(
//     url: string,
//     options?: RequestInit,
// ): Promise<T> => {
//     const config: AxiosRequestConfig = {
//         url: `${getBaseUrl()}${url}`,
//         method: (options?.method as AxiosRequestConfig["method"]) ?? "GET",
//         signal: options?.signal as AbortSignal | undefined,
//         headers: options?.headers as Record<string, string> | undefined,
//         data: options?.body,
//         ...(typeof window === "undefined" && { timeout: 5000 }),
//     };

//     const response = await axios(config);

//     return {
//         data: response.data,
//         status: response.status,
//         headers: response.headers,
//     } as T;
// };

export const customFetch = async <T>(
    url: string,
    options?: RequestInit,
): Promise<T> => {
    // 1. Construct the absolute URL
    const baseUrl = "http://localhost:4000";

    const fullUrl = `${baseUrl}${url}`;

    console.log(`Fetching: ${fullUrl}`); // This will show up in your terminal

    const response = await fetch(fullUrl, {
        ...options,
        headers: {
            "Content-Type": "application/json",
            ...options?.headers,
        },
    });

    if (!response.ok) {
        throw new Error(`API Error: ${response.status}`);
    }

    const data = await response.json();

    return {
        data,
        status: response.status,
        headers: {}, // Keep this empty to avoid the serialization bug
    } as T;
};
