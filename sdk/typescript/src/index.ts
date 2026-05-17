export interface CageBroOptions {
  baseUrl?: string;
  timeout?: number;
}

export interface ShellResult {
  exit_code: number;
  stdout: string;
  stderr: string;
  duration_ms: number;
}

export interface FileEntry {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
}

export interface SearchResult {
  path: string;
  line_number: number;
  line_content: string;
}

export interface PageContent {
  url: string;
  title: string;
  text: string;
}

export interface ScreenshotData {
  data: string;
  format: string;
  width: number;
  height: number;
}

export class CageBro {
  private baseUrl: string;
  private timeout: number;

  constructor(options: CageBroOptions = {}) {
    this.baseUrl = (options.baseUrl || "http://localhost:8080").replace(/\/$/, "");
    this.timeout = options.timeout || 30000;
  }

  private async request<T>(path: string, body: Record<string, unknown> = {}): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal: AbortSignal.timeout(this.timeout),
    });
    return resp.json();
  }

  private async get<T>(path: string): Promise<T> {
    const resp = await fetch(`${this.baseUrl}${path}`, {
      signal: AbortSignal.timeout(this.timeout),
    });
    return resp.json();
  }

  // --- Sandbox ---

  async info(): Promise<Record<string, unknown>> {
    return this.get("/v1/sandbox/info");
  }

  async health(): Promise<Record<string, unknown>> {
    return this.get("/health");
  }

  // --- Shell ---

  async shellExec(command: string, timeoutMs?: number): Promise<ShellResult> {
    const payload: Record<string, unknown> = { command };
    if (timeoutMs) payload.timeout_ms = timeoutMs;
    return this.request("/v1/shell/exec", payload);
  }

  async shellCreateSession(shell?: string): Promise<{ session_id: string; ws_url: string }> {
    const payload: Record<string, unknown> = {};
    if (shell) payload.shell = shell;
    return this.request("/v1/shell/session", payload);
  }

  // --- Files ---

  async fileRead(path: string): Promise<string> {
    const data = await this.request<{ content?: string; error?: string }>("/v1/file/read", { path });
    if (data.error) throw new Error(data.error);
    return data.content!;
  }

  async fileWrite(path: string, content: string): Promise<void> {
    const data = await this.request<{ error?: string }>("/v1/file/write", { path, content });
    if (data.error) throw new Error(data.error);
  }

  async fileEdit(path: string, oldText: string, newText: string): Promise<void> {
    const data = await this.request<{ error?: string }>("/v1/file/edit", {
      path, old_text: oldText, new_text: newText,
    });
    if (data.error) throw new Error(data.error);
  }

  async fileList(path = "."): Promise<FileEntry[]> {
    const data = await this.request<{ entries: FileEntry[] }>("/v1/file/list", { path });
    return data.entries;
  }

  async fileSearch(query: string, path?: string): Promise<SearchResult[]> {
    const payload: Record<string, unknown> = { query };
    if (path) payload.path = path;
    const data = await this.request<{ results: SearchResult[] }>("/v1/file/search", payload);
    return data.results;
  }

  async fileDelete(path: string): Promise<void> {
    const data = await this.request<{ error?: string }>("/v1/file/delete", { path });
    if (data.error) throw new Error(data.error);
  }

  // --- Code ---

  async python(code: string, timeoutMs?: number): Promise<ShellResult> {
    const payload: Record<string, unknown> = { code };
    if (timeoutMs) payload.timeout_ms = timeoutMs;
    return this.request("/v1/code/python", payload);
  }

  async node(code: string, timeoutMs?: number): Promise<ShellResult> {
    const payload: Record<string, unknown> = { code };
    if (timeoutMs) payload.timeout_ms = timeoutMs;
    return this.request("/v1/code/node", payload);
  }

  // --- Browser ---

  async browserLaunch(port?: number, stealth = true): Promise<Record<string, unknown>> {
    const payload: Record<string, unknown> = { stealth };
    if (port) payload.port = port;
    return this.request("/v1/browser/launch", payload);
  }

  async browserNavigate(url: string): Promise<PageContent> {
    return this.request("/v1/browser/navigate", { url });
  }

  async browserScreenshot(): Promise<ScreenshotData> {
    return this.request("/v1/browser/screenshot", {});
  }

  async browserClick(selector: string): Promise<void> {
    await this.request("/v1/browser/click", { selector });
  }

  async browserType(selector: string, text: string): Promise<void> {
    await this.request("/v1/browser/type", { selector, text });
  }

  async browserEvaluate(expression: string): Promise<unknown> {
    const data = await this.request<{ result: unknown }>("/v1/browser/evaluate", { expression });
    return data.result;
  }

  async browserContent(): Promise<PageContent> {
    return this.request("/v1/browser/content", {});
  }

  async browserClose(): Promise<void> {
    await this.request("/v1/browser/close", {});
  }
}
