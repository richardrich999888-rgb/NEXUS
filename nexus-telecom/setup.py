"""
NEXUS Telecom Setup
Copyright (c) 2025 SYNTRIASS Labs Private Limited
"""
from setuptools import setup, find_packages

setup(
    name="nexus-telecom",
    version="0.1.0",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    python_requires=">=3.10",
    install_requires=["numpy>=1.24.0"],
)
