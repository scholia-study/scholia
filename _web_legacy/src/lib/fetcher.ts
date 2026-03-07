import axios from 'axios'
import type { AxiosRequestConfig } from 'axios'

function getBaseUrl(): string {
  if (typeof window === 'undefined') {
    return process.env.API_URL || 'http://localhost:4000'
  }
  return ''
}

export const customFetch = async <T>(
  url: string,
  options?: RequestInit,
): Promise<T> => {
  const config: AxiosRequestConfig = {
    url: `${getBaseUrl()}${url}`,
    method: (options?.method as AxiosRequestConfig['method']) ?? 'GET',
    signal: options?.signal as AbortSignal | undefined,
    headers: options?.headers as Record<string, string> | undefined,
    data: options?.body,
  }

  const response = await axios(config)

  return { data: response.data, status: response.status, headers: response.headers } as T
}
