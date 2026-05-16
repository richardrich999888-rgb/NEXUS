# Multi-stage build for FYNTRAX xApp
# Stage 1: Builder
FROM python:3.10-slim as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir --user -r requirements.txt

# Stage 2: Runtime
FROM python:3.10-slim

LABEL maintainer="SYNTRIASS Labs <contact@syntriass.com>"
LABEL description="FYNTRAX - Physics-First Entropy-Optimized RAN Control Platform"
LABEL version="0.1.0"

# Create non-root user
RUN useradd -m -u 1000 fyntrax && \
    mkdir -p /app /data /logs && \
    chown -R fyntrax:fyntrax /app /data /logs

WORKDIR /app

# Copy Python dependencies from builder
COPY --from=builder /root/.local /home/fyntrax/.local

# Copy application code
COPY --chown=fyntrax:fyntrax src/fyntrax ./fyntrax
COPY --chown=fyntrax:fyntrax pyproject.toml .

# Set environment variables
ENV PATH=/home/fyntrax/.local/bin:$PATH \
    PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    FYNTRAX_DATA_DIR=/data \
    FYNTRAX_LOG_DIR=/logs

# Switch to non-root user
USER fyntrax

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python -c "import fyntrax; print('healthy')" || exit 1

# Expose ports (if needed for APIs)
# EXPOSE 8080

# Default command
CMD ["python", "-m", "fyntrax.simulator.site_sim"]
