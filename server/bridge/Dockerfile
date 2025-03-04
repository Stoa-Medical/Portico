FROM python:3.12-slim

WORKDIR /app

# Install dependencies
COPY pyproject.toml uv.lock ./
RUN pip install --no-cache-dir uv && \
    uv pip install --system -r uv.lock

# Copy application code
COPY ./src .

# Run the bridge service
CMD ["python", "main.py"] 