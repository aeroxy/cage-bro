import httpx
from typing import Optional, Dict, Any, List


class CageBro:
    """Python client for cage-bro sandbox API."""

    def __init__(self, base_url: str = "http://localhost:8080", timeout: float = 30.0):
        self.base_url = base_url.rstrip("/")
        self._client = httpx.Client(base_url=self.base_url, timeout=timeout)

    def close(self):
        self._client.close()

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    # --- Sandbox ---

    def info(self) -> Dict[str, Any]:
        """Get sandbox info."""
        return self._client.get("/v1/sandbox/info").json()

    def health(self) -> Dict[str, Any]:
        """Health check."""
        return self._client.get("/health").json()

    # --- Shell ---

    def shell_exec(self, command: str, timeout_ms: Optional[int] = None) -> Dict[str, Any]:
        """Execute a shell command."""
        payload: Dict[str, Any] = {"command": command}
        if timeout_ms:
            payload["timeout_ms"] = timeout_ms
        return self._client.post("/v1/shell/exec", json=payload).json()

    def shell_create_session(self, shell: Optional[str] = None) -> Dict[str, Any]:
        """Create a persistent shell session."""
        payload = {}
        if shell:
            payload["shell"] = shell
        return self._client.post("/v1/shell/session", json=payload).json()

    # --- Files ---

    def file_read(self, path: str) -> str:
        """Read a file and return its content."""
        resp = self._client.post("/v1/file/read", json={"path": path})
        data = resp.json()
        if "error" in data:
            raise FileNotFoundError(data["error"])
        return data["content"]

    def file_write(self, path: str, content: str) -> None:
        """Write content to a file."""
        resp = self._client.post("/v1/file/write", json={"path": path, "content": content})
        data = resp.json()
        if "error" in data:
            raise IOError(data["error"])

    def file_edit(self, path: str, old_text: str, new_text: str) -> None:
        """Edit a file by replacing old_text with new_text."""
        resp = self._client.post("/v1/file/edit", json={
            "path": path, "old_text": old_text, "new_text": new_text
        })
        data = resp.json()
        if "error" in data:
            raise IOError(data["error"])

    def file_list(self, path: str = ".") -> List[Dict[str, Any]]:
        """List directory contents."""
        resp = self._client.post("/v1/file/list", json={"path": path})
        return resp.json().get("entries", [])

    def file_search(self, query: str, path: Optional[str] = None) -> List[Dict[str, Any]]:
        """Search files for text."""
        payload: Dict[str, Any] = {"query": query}
        if path:
            payload["path"] = path
        resp = self._client.post("/v1/file/search", json=payload)
        return resp.json().get("results", [])

    def file_delete(self, path: str) -> None:
        """Delete a file or directory."""
        resp = self._client.post("/v1/file/delete", json={"path": path})
        data = resp.json()
        if "error" in data:
            raise IOError(data["error"])

    # --- Code ---

    def python(self, code: str, timeout_ms: Optional[int] = None) -> Dict[str, Any]:
        """Execute Python code."""
        payload: Dict[str, Any] = {"code": code}
        if timeout_ms:
            payload["timeout_ms"] = timeout_ms
        return self._client.post("/v1/code/python", json=payload).json()

    def node(self, code: str, timeout_ms: Optional[int] = None) -> Dict[str, Any]:
        """Execute Node.js code."""
        payload: Dict[str, Any] = {"code": code}
        if timeout_ms:
            payload["timeout_ms"] = timeout_ms
        return self._client.post("/v1/code/node", json=payload).json()

    # --- Browser ---

    def browser_launch(self, port: Optional[int] = None, stealth: bool = True) -> Dict[str, Any]:
        """Launch the browser."""
        payload: Dict[str, Any] = {"stealth": stealth}
        if port:
            payload["port"] = port
        return self._client.post("/v1/browser/launch", json=payload).json()

    def browser_navigate(self, url: str) -> Dict[str, Any]:
        """Navigate to a URL."""
        return self._client.post("/v1/browser/navigate", json={"url": url}).json()

    def browser_screenshot(self) -> Dict[str, Any]:
        """Take a screenshot."""
        return self._client.post("/v1/browser/screenshot", json={}).json()

    def browser_click(self, selector: str) -> Dict[str, Any]:
        """Click an element."""
        return self._client.post("/v1/browser/click", json={"selector": selector}).json()

    def browser_type(self, selector: str, text: str) -> Dict[str, Any]:
        """Type text into an element."""
        return self._client.post("/v1/browser/type", json={"selector": selector, "text": text}).json()

    def browser_evaluate(self, expression: str) -> Any:
        """Evaluate JavaScript."""
        resp = self._client.post("/v1/browser/evaluate", json={"expression": expression})
        return resp.json().get("result")

    def browser_content(self) -> Dict[str, Any]:
        """Get current page content."""
        return self._client.post("/v1/browser/content", json={}).json()

    def browser_close(self) -> None:
        """Close the browser."""
        self._client.post("/v1/browser/close", json={})
