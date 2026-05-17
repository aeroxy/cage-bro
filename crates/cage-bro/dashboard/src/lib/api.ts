import { getMockResponse } from "./mock"

let mockMode: boolean | null = null

async function detectMockMode(): Promise<boolean> {
  if (import.meta.env.VITE_MOCK === 'true') {
    return true
  }
  try {
    const resp = await fetch(`${import.meta.env.BASE_URL}health`, { signal: AbortSignal.timeout(2000) })
    const data = await resp.json()
    return data.status !== "ok"
  } catch {
    return true
  }
}

export async function isMockMode(): Promise<boolean> {
  if (mockMode === null) {
    mockMode = await detectMockMode()
  }
  return mockMode
}

export function forceMockMode() {
  mockMode = true
}

export async function apiFetch(path: string, init?: RequestInit): Promise<Response> {
  if (await isMockMode()) {
    const data = getMockResponse(path)
    return new Response(JSON.stringify(data), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    })
  }
  return fetch(`${import.meta.env.BASE_URL}${path}`, init)
}

export async function apiPost<T = any>(path: string, body?: Record<string, unknown>): Promise<T> {
  if (await isMockMode()) {
    return getMockResponse(path) as T
  }
  const resp = await fetch(`${import.meta.env.BASE_URL}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: body ? JSON.stringify(body) : undefined,
  })
  return resp.json()
}

export function getWsUrl(path: string): string {
  const proto = location.protocol === "https:" ? "wss:" : "ws:"
  const base = import.meta.env.BASE_URL.replace(/\/$/, '')
  return `${proto}//${location.host}${base}${path}`
}
