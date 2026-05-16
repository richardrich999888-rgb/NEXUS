# Multi-Stage Build for Production
# Stage 1: Builder
FROM python:3.9-slim as builder

WORKDIR /app
COPY requirements.txt .
RUN pip install --user --no-cache-dir -r requirements.txt

# Stage 2: Runtime (Hardened)
FROM python:3.9-slim

# Create a non-root user
RUN useradd -m asimuser
USER asimuser
WORKDIR /app

# Copy dependencies from builder
COPY --from=builder /root/.local /home/asimuser/.local
ENV PATH=/home/asimuser/.local/bin:$PATH

# Copy Code
COPY --chown=asimuser:asimuser . .

# Environment Variables
ENV PYTHONUNBUFFERED=1
ENV PYTHONPATH=/app

# Healthcheck
HEALTHCHECK --interval=30s --timeout=5s \
  CMD curl -f http://localhost:8000/health || exit 1

# Expose API Port
EXPOSE 8000

# Run with Gunicorn (Production Server)
CMD ["uvicorn", "server.api:app", "--host", "0.0.0.0", "--port", "8000", "--workers", "4"]
