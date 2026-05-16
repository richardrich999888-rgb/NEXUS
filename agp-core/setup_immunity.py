"""
Setup for AIS-ASI: Artificial Immune System for ASI Safety

This package implements a bio-inspired multi-layered defense system
for AI safety, based on the biological immune system.
"""

from setuptools import setup, find_packages

with open("README_IMMUNITY.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="ais-asi",
    version="1.0.0",
    author="NEXUS Team",
    author_email="research@nexus.ai",
    description="Artificial Immune System for ASI Safety",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/nexus/ais-asi",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Science/Research",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Security",
    ],
    python_requires=">=3.9",
    install_requires=[
        "torch>=2.0.0",
        "numpy>=1.21.0",
        "tqdm>=4.64.0",
    ],
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-cov>=4.0.0",
            "black>=23.0.0",
            "isort>=5.12.0",
            "mypy>=1.0.0",
        ],
        "viz": [
            "matplotlib>=3.5.0",
            "seaborn>=0.12.0",
        ],
        "full": [
            "pytest>=7.0.0",
            "pytest-cov>=4.0.0",
            "matplotlib>=3.5.0",
            "seaborn>=0.12.0",
        ],
    },
    entry_points={
        "console_scripts": [
            "ais-demo=examples.immunity.basic_demo:main",
        ],
    },
)
