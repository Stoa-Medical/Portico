"""
Sandboxed subprocess executor for user-defined Python steps.

Writes user code and input to temp files, executes in a subprocess with timeout
enforcement, captures stdout as the result, and cleans up temp files afterward.
"""
import json
import os
import subprocess
import sys
import tempfile
import textwrap

# Maximum execution time in seconds, overridable via environment variable.
MAX_EXECUTION_TIME = int(os.environ.get("MAX_EXECUTION_TIME", "60"))

# ---------------------------------------------------------------------------
# Wrapper script executed inside the subprocess.  It is written to a temp file
# and invoked as a standalone Python process so the user code runs in complete
# isolation from the sidecar server process.
# ---------------------------------------------------------------------------
_WRAPPER_SCRIPT = textwrap.dedent(r'''
    """Subprocess wrapper — runs user code in a restricted namespace."""
    import builtins
    import importlib
    import json
    import sys
    import os

    # ---- 1. Read input JSON from the temp file path passed as argv[1] ----
    input_path = sys.argv[1]
    with open(input_path, "r") as f:
        _input_data = json.load(f)

    # ---- 2. Build restricted builtins ----

    _BLOCKED_MODULES = frozenset({
        "subprocess", "ctypes", "ctypes.util",
        "shutil", "signal", "multiprocessing",
        "webbrowser", "antigravity", "turtle",
    })

    _BLOCKED_OS_ATTRS = frozenset({
        "system", "popen", "exec", "execl", "execle", "execlp", "execlpe",
        "execv", "execve", "execvp", "execvpe", "spawn", "spawnl", "spawnle",
        "spawnlp", "spawnlpe", "spawnv", "spawnve", "spawnvp", "spawnvpe",
        "fork", "forkpty", "kill", "killpg",
    })

    _BLOCKED_SOCKET_ATTRS = frozenset({
        "socket", "create_connection", "create_server",
    })

    _original_import = builtins.__import__

    def _restricted_import(name, *args, **kwargs):
        """Import hook that blocks dangerous modules and patches risky attrs."""
        # Block entirely prohibited modules
        top_level = name.split(".")[0]
        if name in _BLOCKED_MODULES or top_level in _BLOCKED_MODULES:
            raise ImportError(f"Import of '{name}' is not allowed in step code")

        mod = _original_import(name, *args, **kwargs)

        # Patch os — remove dangerous callables
        if name == "os" or top_level == "os":
            for attr in _BLOCKED_OS_ATTRS:
                if hasattr(mod, attr):
                    setattr(mod, attr, _blocked_callable(f"os.{attr}"))

        # Patch socket — block direct socket creation
        if name == "socket" or top_level == "socket":
            for attr in _BLOCKED_SOCKET_ATTRS:
                if hasattr(mod, attr):
                    setattr(mod, attr, _blocked_callable(f"socket.{attr}"))

        return mod

    def _blocked_callable(name):
        """Return a callable that raises when invoked."""
        def _blocked(*a, **kw):
            raise PermissionError(f"Use of '{name}' is not allowed in step code")
        return _blocked

    # Restrict open() to /tmp only
    _original_open = builtins.open

    def _restricted_open(path, *args, **kwargs):
        resolved = os.path.realpath(str(path))
        if not resolved.startswith("/tmp"):
            raise PermissionError(
                f"File access outside /tmp is not allowed: {path}"
            )
        return _original_open(path, *args, **kwargs)

    # ---- 3. Build the execution namespace ----
    _namespace = {
        "__builtins__": {
            **{k: getattr(builtins, k) for k in dir(builtins) if not k.startswith("_")},
            "__import__": _restricted_import,
            "open": _restricted_open,
            # Keep dunder helpers that Python itself needs
            "__name__": "__main__",
            "__build_class__": builtins.__build_class__,
        },
        "input_data": _input_data,
        "output_data": None,
    }

    # ---- 4. Read and execute user code ----
    code_path = sys.argv[2]
    with open(code_path, "r") as f:
        user_code = f.read()

    try:
        compiled = compile(user_code, "<user_step>", "exec")
        exec(compiled, _namespace)
    except Exception as exc:
        print(json.dumps({"__error__": f"{type(exc).__name__}: {exc}"}))
        sys.exit(1)

    # ---- 5. Emit result JSON to stdout ----
    result = _namespace.get("output_data")
    if result is None:
        print(json.dumps(None))
    else:
        try:
            print(json.dumps(result))
        except (TypeError, ValueError) as exc:
            print(json.dumps({"__error__": f"output_data is not JSON-serializable: {exc}"}))
            sys.exit(1)
''')


def execute_step(code: str, input_json: str, timeout_seconds: int = 30) -> str:
    """
    Execute user-provided Python *code* in an isolated subprocess.

    Parameters
    ----------
    code : str
        The Python source code to execute.  The code has access to
        ``input_data`` (parsed from *input_json*) and must set
        ``output_data`` to its result.
    input_json : str
        JSON string passed as input to the user code.
    timeout_seconds : int
        Maximum wall-clock seconds the subprocess is allowed to run.
        Capped at ``MAX_EXECUTION_TIME``.

    Returns
    -------
    str
        JSON string produced by the subprocess (the serialized ``output_data``).

    Raises
    ------
    TimeoutError
        If the subprocess exceeds the allowed time.
    RuntimeError
        If the subprocess exits with a non-zero code or produces invalid output.
    """
    # Cap the timeout to the server-wide maximum.
    effective_timeout = min(timeout_seconds, MAX_EXECUTION_TIME)

    # Validate that input_json is actually valid JSON before handing it off.
    try:
        json.loads(input_json)
    except json.JSONDecodeError as exc:
        raise ValueError(f"input_json is not valid JSON: {exc}") from exc

    # Write temp files for the wrapper script, user code, and input data.
    tmp_wrapper = tmp_code = tmp_input = None
    try:
        tmp_wrapper = tempfile.NamedTemporaryFile(
            mode="w", suffix="_wrapper.py", dir="/tmp", delete=False,
        )
        tmp_wrapper.write(_WRAPPER_SCRIPT)
        tmp_wrapper.close()

        tmp_code = tempfile.NamedTemporaryFile(
            mode="w", suffix="_step.py", dir="/tmp", delete=False,
        )
        tmp_code.write(code)
        tmp_code.close()

        tmp_input = tempfile.NamedTemporaryFile(
            mode="w", suffix="_input.json", dir="/tmp", delete=False,
        )
        tmp_input.write(input_json)
        tmp_input.close()

        # Launch the subprocess.
        proc = subprocess.run(
            [sys.executable, tmp_wrapper.name, tmp_input.name, tmp_code.name],
            capture_output=True,
            text=True,
            timeout=effective_timeout,
        )

        if proc.returncode != 0:
            # Try to extract a structured error from stdout first.
            error_detail = ""
            try:
                parsed = json.loads(proc.stdout)
                if isinstance(parsed, dict) and "__error__" in parsed:
                    error_detail = parsed["__error__"]
            except (json.JSONDecodeError, TypeError):
                pass

            if not error_detail:
                error_detail = proc.stderr.strip() or proc.stdout.strip() or "Unknown error"

            raise RuntimeError(f"Step failed: {error_detail}")

        # The subprocess printed result JSON to stdout.
        return proc.stdout.strip()

    except subprocess.TimeoutExpired:
        raise TimeoutError(
            f"Step execution exceeded {effective_timeout}s timeout"
        )
    finally:
        # Clean up all temp files.
        for tmp in (tmp_wrapper, tmp_code, tmp_input):
            if tmp is not None:
                try:
                    os.unlink(tmp.name)
                except OSError:
                    pass
