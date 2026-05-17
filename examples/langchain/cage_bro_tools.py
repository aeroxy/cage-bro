"""LangChain integration example for cage-bro."""

from langchain.tools import BaseTool
from typing import Optional, Type
from pydantic import BaseModel, Field
from cage_bro import CageBro


class ShellExecInput(BaseModel):
    command: str = Field(description="Shell command to execute")


class ShellExecTool(BaseTool):
    name: str = "shell_exec"
    description: str = "Execute a shell command in the cage-bro sandbox"
    args_schema: Type[BaseModel] = ShellExecInput

    def _run(self, command: str) -> str:
        with CageBro() as cage:
            result = cage.shell_exec(command)
            return f"exit_code: {result['exit_code']}\nstdout: {result['stdout']}\nstderr: {result['stderr']}"


class PythonExecInput(BaseModel):
    code: str = Field(description="Python code to execute")


class PythonExecTool(BaseTool):
    name: str = "python_exec"
    description: str = "Execute Python code in the cage-bro sandbox"
    args_schema: Type[BaseModel] = PythonExecInput

    def _run(self, code: str) -> str:
        with CageBro() as cage:
            result = cage.python(code)
            return f"exit_code: {result['exit_code']}\nstdout: {result['stdout']}\nstderr: {result['stderr']}"


class FileReadInput(BaseModel):
    path: str = Field(description="Path to the file to read")


class FileReadTool(BaseTool):
    name: str = "file_read"
    description: str = "Read a file from the cage-bro sandbox"
    args_schema: Type[BaseModel] = FileReadInput

    def _run(self, path: str) -> str:
        with CageBro() as cage:
            return cage.file_read(path)


class BrowserNavigateInput(BaseModel):
    url: str = Field(description="URL to navigate to")


class BrowserNavigateTool(BaseTool):
    name: str = "browser_navigate"
    description: str = "Navigate the browser to a URL"
    args_schema: Type[BaseModel] = BrowserNavigateInput

    def _run(self, url: str) -> str:
        with CageBro() as cage:
            result = cage.browser_navigate(url)
            return f"Title: {result['title']}\nURL: {result['url']}\n\n{result['text']}"


# Example usage with LangChain agent
if __name__ == "__main__":
    from langchain.agents import AgentExecutor, create_react_agent
    from langchain_openai import ChatOpenAI

    tools = [ShellExecTool(), PythonExecTool(), FileReadTool(), BrowserNavigateTool()]
    llm = ChatOpenAI(model="gpt-4")

    # Create agent with cage-bro tools
    agent = create_react_agent(llm, tools, prompt="You are a helpful assistant with access to a sandboxed environment.")
    executor = AgentExecutor(agent=agent, tools=tools, verbose=True)

    # Run
    result = executor.invoke({"input": "What files are in the current directory?"})
    print(result["output"])
