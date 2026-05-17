import json
from openai import OpenAI
from cage_bro import CageBro

client = OpenAI()
cage = CageBro()

tools = [
    {
        "type": "function",
        "function": {
            "name": "shell_exec",
            "description": "Execute a shell command in the sandbox",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command to execute"}
                },
                "required": ["command"]
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "python_exec",
            "description": "Execute Python code in the sandbox",
            "parameters": {
                "type": "object",
                "properties": {
                    "code": {"type": "string", "description": "Python code to execute"}
                },
                "required": ["code"]
            }
        }
    },
    {
        "type": "function",
        "function": {
            "name": "file_read",
            "description": "Read a file from the sandbox",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "File path"}
                },
                "required": ["path"]
            }
        }
    }
]

def handle_tool_call(tool_call):
    name = tool_call.function.name
    args = json.loads(tool_call.function.arguments)

    if name == "shell_exec":
        result = cage.shell_exec(args["command"])
        return json.dumps(result)
    elif name == "python_exec":
        result = cage.python(args["code"])
        return json.dumps(result)
    elif name == "file_read":
        return cage.file_read(args["path"])
    else:
        return json.dumps({"error": f"Unknown tool: {name}"})

# Chat loop
messages = [{"role": "system", "content": "You are a helpful assistant with access to a sandboxed execution environment."}]

while True:
    user_input = input("You: ")
    if user_input.lower() in ("exit", "quit"):
        break

    messages.append({"role": "user", "content": user_input})

    response = client.chat.completions.create(
        model="gpt-4",
        messages=messages,
        tools=tools,
        tool_choice="auto"
    )

    msg = response.choices[0].message
    messages.append(msg)

    if msg.tool_calls:
        for tool_call in msg.tool_calls:
            result = handle_tool_call(tool_call)
            messages.append({
                "role": "tool",
                "tool_call_id": tool_call.id,
                "content": result
            })
        # Get final response
        response = client.chat.completions.create(model="gpt-4", messages=messages, tools=tools)
        msg = response.choices[0].message
        messages.append(msg)

    if msg.content:
        print(f"Assistant: {msg.content}")