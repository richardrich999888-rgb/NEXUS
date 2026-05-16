from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

with open("requirements.txt", "r", encoding="utf-8") as fh:
    requirements = [line.strip() for line in fh if line.strip() and not line.startswith('#')]

setup(
    name="aura-protocol",
    version="1.0.0",
    author="SYNTRIASS Labs",
    author_email="contact@syntriass.com",
    description="Quantum-resistant, infrastructure-less verification protocol",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/syntriass/aura-protocol",
    packages=find_packages(where="src"),
    package_dir={"": "src"},
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Intended Audience :: Financial and Insurance Industry",
        "Topic :: Security :: Cryptography",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.8",
    install_requires=requirements,
    entry_points={
        "console_scripts": [
            "aura-mvp=mvp.72hour_mvp:main",
        ],
    },
    keywords="cryptography quantum-resistant verification payment dns pki",
    project_urls={
        "Documentation": "https://docs.aura-protocol.com",
        "Source": "https://github.com/syntriass/aura-protocol",
    },
)
