import axios from 'axios'
import type { AxiosRequestConfig } from 'axios'

export const customFetch = async <T>(
  url: string,
  options?: RequestInit,
): Promise<T> => {
  const config: AxiosRequestConfig = {
    url,
    method: (options?.method as AxiosRequestConfig['method']) ?? 'GET',
    signal: options?.signal as AbortSignal | undefined,
    headers: options?.headers as Record<string, string> | undefined,
    data: options?.body,
  }

  const response = await axios(config)

  return { data: response.data, status: response.status, headers: response.headers } as T
}
