FROM python:3.12-slim

# Security: non-root user
RUN groupadd -r portico && useradd -r -g portico -u 1001 portico-executor

WORKDIR /app
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY server.py executor.py ./

# Security: read-only filesystem except /tmp
RUN chmod -R 555 /app

USER portico-executor

EXPOSE 8001
CMD ["uvicorn", "server:app", "--host", "0.0.0.0", "--port", "8001"]
