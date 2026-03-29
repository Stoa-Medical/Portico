"""
Python Step Sidecar — Executes user-defined Python steps in isolated subprocesses.
Communicates with the Rust engine via HTTP (FastAPI).
"""
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from executor import execute_step
import time

app = FastAPI(title="Portico Python Sidecar")

class StepRequest(BaseModel):
    code: str
    input_json: str
    timeout_seconds: int = 30

class StepResponse(BaseModel):
    success: bool
    output_json: str = ""
    error: str = ""
    execution_ms: int = 0

@app.post("/execute", response_model=StepResponse)
async def execute(request: StepRequest) -> StepResponse:
    start = time.monotonic()
    try:
        result = execute_step(
            code=request.code,
            input_json=request.input_json,
            timeout_seconds=request.timeout_seconds,
        )
        elapsed_ms = int((time.monotonic() - start) * 1000)
        return StepResponse(success=True, output_json=result, execution_ms=elapsed_ms)
    except TimeoutError:
        elapsed_ms = int((time.monotonic() - start) * 1000)
        return StepResponse(success=False, error="Step execution timed out", execution_ms=elapsed_ms)
    except Exception as e:
        elapsed_ms = int((time.monotonic() - start) * 1000)
        return StepResponse(success=False, error=str(e), execution_ms=elapsed_ms)

@app.get("/health")
async def health():
    return {"status": "ok"}
