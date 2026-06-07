use serde_json::{json, Value};
use cage_bro_runtime::{Filesystem, SandboxRuntime};
use crate::mcp::server::JsonRpcError;
use crate::server::AppState;

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn new() -> Self {
        Self
    }

    pub fn list(&self) -> Vec<Value> {
        vec![
            tool("shell_exec", "Execute a shell command", json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute"}
                },
                "required": ["command"]
            })),
            tool("file_read", "Read a file", json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path"}
                },
                "required": ["path"]
            })),
            tool("file_write", "Write a file", json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path"},
                    "content": {"type": "string", "description": "File content"}
                },
                "required": ["path", "content"]
            })),
            tool("file_edit", "Edit a file with string replacement", json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path"},
                    "old_text": {"type": "string", "description": "Text to replace"},
                    "new_text": {"type": "string", "description": "Replacement text"}
                },
                "required": ["path", "old_text", "new_text"]
            })),
            tool("file_list", "List directory contents", json!({
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Directory path"}
                },
                "required": ["path"]
            })),
            tool("file_search", "Search files for text", json!({
                "type": "object",
                "properties": {
                    "query": {"type": "string", "description": "Search query"},
                    "path": {"type": "string", "description": "Directory to search in"}
                },
                "required": ["query"]
            })),
            tool("python_exec", "Execute Python code", json!({
                "type": "object",
                "properties": {
                    "code": {"type": "string", "description": "Python code to execute"}
                },
                "required": ["code"]
            })),
            tool("node_exec", "Execute Node.js code", json!({
                "type": "object",
                "properties": {
                    "code": {"type": "string", "description": "Node.js code to execute"}
                },
                "required": ["code"]
            })),
            tool("browser_navigate", "Navigate browser to URL", json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string", "description": "URL to navigate to"}
                },
                "required": ["url"]
            })),
            tool("browser_screenshot", "Take a browser screenshot", json!({
                "type": "object",
                "properties": {}
            })),
            tool("browser_click", "Click an element by CSS selector", json!({
                "type": "object",
                "properties": {
                    "selector": {"type": "string", "description": "CSS selector"}
                },
                "required": ["selector"]
            })),
            tool("browser_type", "Type text into an element", json!({
                "type": "object",
                "properties": {
                    "selector": {"type": "string", "description": "CSS selector"},
                    "text": {"type": "string", "description": "Text to type"}
                },
                "required": ["selector", "text"]
            })),
            tool("browser_evaluate", "Evaluate JavaScript in browser", json!({
                "type": "object",
                "properties": {
                    "expression": {"type": "string", "description": "JavaScript expression"}
                },
                "required": ["expression"]
            })),
            tool("browser_snapshot", "Get current page content", json!({
                "type": "object",
                "properties": {}
            })),
            tool("sandbox_info", "Get sandbox information", json!({
                "type": "object",
                "properties": {}
            })),
        ]
    }

    pub async fn call(&self, state: &AppState, name: &str, args: Value) -> Result<Value, JsonRpcError> {
        match name {
            "shell_exec" => {
                let cmd = args["command"].as_str().ok_or_else(|| err("Missing command"))?;
                let ws = state.workspace.to_string_lossy().to_string();
                let config = cage_bro_runtime::SandboxConfig {
                    workspace_dir: Some(ws.clone()),
                    ..Default::default()
                };
                let sandbox = state.runtime.create(config).await.map_err(|e| err(&e.to_string()))?;
                let parts = shell_words::split(cmd).map_err(|e| err(&e.to_string()))?;
                let exec_cmd = cage_bro_runtime::ExecCommand {
                    program: parts[0].clone(),
                    args: parts[1..].to_vec(),
                    env: std::collections::HashMap::new(),
                    working_dir: Some(ws),
                    timeout_ms: None,
                };
                let result = state.runtime.exec(&sandbox, exec_cmd).await;
                let _ = state.runtime.destroy(&sandbox).await;
                match result {
                    Ok(r) => Ok(json!({"content": [{"type": "text", "text": format!("exit_code: {}\nstdout:\n{}\nstderr:\n{}", r.exit_code, r.stdout, r.stderr)}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "file_read" => {
                let path = args["path"].as_str().ok_or_else(|| err("Missing path"))?;
                match state.filesystem.read(path).await {
                    Ok(r) => Ok(json!({"content": [{"type": "text", "text": r.content}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "file_write" => {
                let path = args["path"].as_str().ok_or_else(|| err("Missing path"))?;
                let content = args["content"].as_str().ok_or_else(|| err("Missing content"))?;
                match state.filesystem.write(cage_bro_runtime::FileWriteRequest {
                    path: path.to_string(),
                    content: content.to_string(),
                    encoding: None,
                }).await {
                    Ok(()) => Ok(json!({"content": [{"type": "text", "text": "File written successfully"}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "file_edit" => {
                let path = args["path"].as_str().ok_or_else(|| err("Missing path"))?;
                let old_text = args["old_text"].as_str().ok_or_else(|| err("Missing old_text"))?;
                let new_text = args["new_text"].as_str().ok_or_else(|| err("Missing new_text"))?;
                match state.filesystem.edit(cage_bro_runtime::FileEditRequest {
                    path: path.to_string(),
                    old_text: old_text.to_string(),
                    new_text: new_text.to_string(),
                }).await {
                    Ok(()) => Ok(json!({"content": [{"type": "text", "text": "File edited successfully"}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "file_list" => {
                let path = args["path"].as_str().ok_or_else(|| err("Missing path"))?;
                match state.filesystem.list(path).await {
                    Ok(entries) => {
                        let text = entries.iter().map(|e| {
                            format!("{}{}  {} bytes", if e.is_dir { "[DIR] " } else { "" }, e.name, e.size)
                        }).collect::<Vec<_>>().join("\n");
                        Ok(json!({"content": [{"type": "text", "text": text}]}))
                    }
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "file_search" => {
                let query = args["query"].as_str().ok_or_else(|| err("Missing query"))?;
                let path = args["path"].as_str().map(|s| s.to_string());
                match state.filesystem.search(cage_bro_runtime::FileSearchRequest {
                    query: query.to_string(),
                    path,
                    file_pattern: None,
                    max_results: Some(20),
                }).await {
                    Ok(results) => {
                        let text = results.iter().map(|r| {
                            format!("{}:{}: {}", r.path, r.line_number, r.line_content)
                        }).collect::<Vec<_>>().join("\n");
                        Ok(json!({"content": [{"type": "text", "text": if text.is_empty() { "No results found".to_string() } else { text }}]}))
                    }
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "python_exec" => {
                let code = args["code"].as_str().ok_or_else(|| err("Missing code"))?;
                let ws = state.workspace.to_string_lossy().to_string();
                let temp_file = format!("/tmp/cage_bro_mcp_{}.py", uuid::Uuid::new_v4());
                tokio::fs::write(&temp_file, code).await.map_err(|e| err(&e.to_string()))?;
                let config = cage_bro_runtime::SandboxConfig {
                    workspace_dir: Some(ws.clone()),
                    ..Default::default()
                };
                let sandbox = state.runtime.create(config).await.map_err(|e| err(&e.to_string()))?;
                let exec_cmd = cage_bro_runtime::ExecCommand {
                    program: "python3".to_string(),
                    args: vec![temp_file.clone()],
                    env: std::collections::HashMap::new(),
                    working_dir: Some(ws),
                    timeout_ms: Some(30000),
                };
                let result = state.runtime.exec(&sandbox, exec_cmd).await;
                let _ = state.runtime.destroy(&sandbox).await;
                let _ = tokio::fs::remove_file(&temp_file).await;
                match result {
                    Ok(r) => Ok(json!({"content": [{"type": "text", "text": format!("exit_code: {}\nstdout:\n{}\nstderr:\n{}", r.exit_code, r.stdout, r.stderr)}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "node_exec" => {
                let code = args["code"].as_str().ok_or_else(|| err("Missing code"))?;
                let ws = state.workspace.to_string_lossy().to_string();
                let temp_file = format!("/tmp/cage_bro_mcp_{}.js", uuid::Uuid::new_v4());
                tokio::fs::write(&temp_file, code).await.map_err(|e| err(&e.to_string()))?;
                let config = cage_bro_runtime::SandboxConfig {
                    workspace_dir: Some(ws.clone()),
                    ..Default::default()
                };
                let sandbox = state.runtime.create(config).await.map_err(|e| err(&e.to_string()))?;
                let exec_cmd = cage_bro_runtime::ExecCommand {
                    program: "node".to_string(),
                    args: vec![temp_file.clone()],
                    env: std::collections::HashMap::new(),
                    working_dir: Some(ws),
                    timeout_ms: Some(30000),
                };
                let result = state.runtime.exec(&sandbox, exec_cmd).await;
                let _ = state.runtime.destroy(&sandbox).await;
                let _ = tokio::fs::remove_file(&temp_file).await;
                match result {
                    Ok(r) => Ok(json!({"content": [{"type": "text", "text": format!("exit_code: {}\nstdout:\n{}\nstderr:\n{}", r.exit_code, r.stdout, r.stderr)}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "browser_navigate" => {
                let url = args["url"].as_str().ok_or_else(|| err("Missing url"))?;
                match state.browser.navigate(url).await {
                    Ok(page) => Ok(json!({"content": [{"type": "text", "text": format!("{}\n\n{}", page.title, page.text)}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "browser_screenshot" => {
                match state.browser.screenshot(None).await {
                    Ok(s) => Ok(json!({"content": [{"type": "image", "data": s.data, "mimeType": "image/png"}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "browser_click" => {
                let selector = args["selector"].as_str().ok_or_else(|| err("Missing selector"))?;
                match state.browser.click(selector).await {
                    Ok(()) => Ok(json!({"content": [{"type": "text", "text": "Clicked successfully"}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "browser_type" => {
                let selector = args["selector"].as_str().ok_or_else(|| err("Missing selector"))?;
                let text = args["text"].as_str().ok_or_else(|| err("Missing text"))?;
                match state.browser.type_text(selector, text).await {
                    Ok(()) => Ok(json!({"content": [{"type": "text", "text": "Typed successfully"}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "browser_evaluate" => {
                let expression = args["expression"].as_str().ok_or_else(|| err("Missing expression"))?;
                match state.browser.execute_js(expression).await {
                    Ok(val) => Ok(json!({"content": [{"type": "text", "text": val.to_string()}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "browser_snapshot" => {
                match state.browser.get_content().await {
                    Ok(page) => Ok(json!({"content": [{"type": "text", "text": format!("URL: {}\nTitle: {}\n\n{}", page.url, page.title, page.text)}]})),
                    Err(e) => Ok(json!({"content": [{"type": "text", "text": format!("Error: {}", e)}], "isError": true})),
                }
            }
            "sandbox_info" => {
                let browser_status = state.browser.status().await;
                Ok(json!({"content": [{"type": "text", "text": format!("cage-bro v{}\nRuntime: process\nBrowser: {}", env!("CARGO_PKG_VERSION"), if browser_status["running"].as_bool().unwrap_or(false) { "running" } else { "stopped" })}]}))
            }
            _ => Err(JsonRpcError {
                code: -32602,
                message: format!("Unknown tool: {}", name),
            }),
        }
    }
}

fn tool(name: &str, description: &str, input_schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": input_schema
    })
}

fn err(msg: &str) -> JsonRpcError {
    JsonRpcError {
        code: -32602,
        message: msg.to_string(),
    }
}
