"""Setup script for VECTRA Python bindings."""

from setuptools import setup, find_packages

setup(
    name="vectra",
    version="0.1.0",
    description="Deterministic, lossless data volume reduction for structured payloads",
    author="SYNTRIASS LABS",
    packages=find_packages(),
    python_requires=">=3.9",
    install_requires=[],
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-cov>=4.0.0",
            "mypy>=1.0.0",
        ],
    },
)










