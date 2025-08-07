from setuptools import setup, find_packages

with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="flame",
    version="0.1.0",
    author="Flame Authors",
    author_email="klaus1982.cn@gmail.com",
    description="Python SDK for Flame distributed computing framework",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/flame-sh/flame",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
    ],
    python_requires=">=3.8",
    install_requires=[
        "grpcio>=1.50.0",
        "grpcio-tools>=1.50.0",
        "protobuf>=4.21.0",
    ],
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-asyncio>=0.21.0",
            "black>=22.0.0",
            "isort>=5.0.0",
            "mypy>=1.0.0",
        ]
    },
) 