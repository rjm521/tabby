# Docker

## 前提条件
* 系统必须安装并运行 Docker。
* 创建一个文件夹来存储大型模型和中间文件（例如 /llama/models）

## 镜像
我们为这个项目提供了三个 Docker 镜像：

1. `ghcr.io/ggml-org/llama.cpp:full`：此镜像包含主可执行文件和用于将 LLaMA 模型转换为 ggml 并进行 4 位量化的工具。（平台：`linux/amd64`、`linux/arm64`）
2. `ghcr.io/ggml-org/llama.cpp:light`：此镜像仅包含主可执行文件。（平台：`linux/amd64`、`linux/arm64`）
3. `ghcr.io/ggml-org/llama.cpp:server`：此镜像仅包含服务器可执行文件。（平台：`linux/amd64`、`linux/arm64`）

此外，还有以下镜像，与上述类似：

- `ghcr.io/ggml-org/llama.cpp:full-cuda`：与 `full` 相同，但使用 CUDA 支持编译。（平台：`linux/amd64`）
- `ghcr.io/ggml-org/llama.cpp:light-cuda`：与 `light` 相同，但使用 CUDA 支持编译。（平台：`linux/amd64`）
- `ghcr.io/ggml-org/llama.cpp:server-cuda`：与 `server` 相同，但使用 CUDA 支持编译。（平台：`linux/amd64`）
- `ghcr.io/ggml-org/llama.cpp:full-rocm`：与 `full` 相同，但使用 ROCm 支持编译。（平台：`linux/amd64`、`linux/arm64`）
- `ghcr.io/ggml-org/llama.cpp:light-rocm`：与 `light` 相同，但使用 ROCm 支持编译。（平台：`linux/amd64`、`linux/arm64`）
- `ghcr.io/ggml-org/llama.cpp:server-rocm`：与 `server` 相同，但使用 ROCm 支持编译。（平台：`linux/amd64`、`linux/arm64`）
- `ghcr.io/ggml-org/llama.cpp:full-musa`：与 `full` 相同，但使用 MUSA 支持编译。（平台：`linux/amd64`）
- `ghcr.io/ggml-org/llama.cpp:light-musa`：与 `light` 相同，但使用 MUSA 支持编译。（平台：`linux/amd64`）
- `ghcr.io/ggml-org/llama.cpp:server-musa`：与 `server` 相同，但使用 MUSA 支持编译。（平台：`linux/amd64`）

目前，除了构建之外，CI 尚未测试启用 GPU 的镜像。它们与 [.devops/](../.devops/) 中定义的 Dockerfile 和 [.github/workflows/docker.yml](../.github/workflows/docker.yml) 中定义的 GitHub Action 中的镜像没有任何变化。如果您需要不同的设置（例如，不同的 CUDA、ROCm 或 MUSA 库），目前您需要在本地构建镜像。

## 使用方法

下载模型、将其转换为 ggml 并优化它们的最简单方法是使用包含完整 docker 镜像的 --all-in-one 命令。

将下面的 `/path/to/models` 替换为您下载模型的实际路径。

```bash
docker run -v /path/to/models:/models ghcr.io/ggml-org/llama.cpp:full --all-in-one "/models/" 7B
```

完成后，您就可以开始使用了！

```bash
docker run -v /path/to/models:/models ghcr.io/ggml-org/llama.cpp:full --run -m /models/7B/ggml-model-q4_0.gguf -p "Building a website can be done in 10 simple steps:" -n 512
```

或使用轻量级镜像：

```bash
docker run -v /path/to/models:/models ghcr.io/ggml-org/llama.cpp:light -m /models/7B/ggml-model-q4_0.gguf -p "Building a website can be done in 10 simple steps:" -n 512
```

或使用服务器镜像：

```bash
docker run -v /path/to/models:/models -p 8000:8000 ghcr.io/ggml-org/llama.cpp:server -m /models/7B/ggml-model-q4_0.gguf --port 8000 --host 0.0.0.0 -n 512
```

## 使用 CUDA 的 Docker

假设在 Linux 上正确安装了 [nvidia-container-toolkit](https://github.com/NVIDIA/nvidia-container-toolkit)，或者正在使用支持 GPU 的云服务，容器内应该可以访问 `cuBLAS`。

## 本地构建 Docker

```bash
docker build -t local/llama.cpp:full-cuda --target full -f .devops/cuda.Dockerfile .
docker build -t local/llama.cpp:light-cuda --target light -f .devops/cuda.Dockerfile .
docker build -t local/llama.cpp:server-cuda --target server -f .devops/cuda.Dockerfile .
```

根据容器主机支持的 CUDA 环境以及 GPU 架构，您可能需要传入一些不同的 `ARGS`。

默认值为：

- `CUDA_VERSION` 设置为 `12.4.0`
- `CUDA_DOCKER_ARCH` 设置为 cmake 构建默认值，包括所有支持的架构

生成的镜像与非 CUDA 镜像基本相同：

1. `local/llama.cpp:full-cuda`：此镜像包含主可执行文件和用于将 LLaMA 模型转换为 ggml 并进行 4 位量化的工具。
2. `local/llama.cpp:light-cuda`：此镜像仅包含主可执行文件。
3. `local/llama.cpp:server-cuda`：此镜像仅包含服务器可执行文件。

## 使用方法

本地构建后，使用方法与非 CUDA 示例类似，但您需要添加 `--gpus` 标志。您还需要使用 `--n-gpu-layers` 标志。

```bash
docker run --gpus all -v /path/to/models:/models local/llama.cpp:full-cuda --run -m /models/7B/ggml-model-q4_0.gguf -p "Building a website can be done in 10 simple steps:" -n 512 --n-gpu-layers 1
docker run --gpus all -v /path/to/models:/models local/llama.cpp:light-cuda -m /models/7B/ggml-model-q4_0.gguf -p "Building a website can be done in 10 simple steps:" -n 512 --n-gpu-layers 1
docker run --gpus all -v /path/to/models:/models local/llama.cpp:server-cuda -m /models/7B/ggml-model-q4_0.gguf --port 8000 --host 0.0.0.0 -n 512 --n-gpu-layers 1
```

## 使用 MUSA 的 Docker

假设在 Linux 上正确安装了 [mt-container-toolkit](https://developer.mthreads.com/musa/native)，容器内应该可以访问 `muBLAS`。

## 本地构建 Docker

```bash
docker build -t local/llama.cpp:full-musa --target full -f .devops/musa.Dockerfile .
docker build -t local/llama.cpp:light-musa --target light -f .devops/musa.Dockerfile .
docker build -t local/llama.cpp:server-musa --target server -f .devops/musa.Dockerfile .
```

根据容器主机支持的 MUSA 环境以及 GPU 架构，您可能需要传入一些不同的 `ARGS`。

默认值为：

- `MUSA_VERSION` 设置为 `rc3.1.1`

生成的镜像与非 MUSA 镜像基本相同：

1. `local/llama.cpp:full-musa`：此镜像包含主可执行文件和用于将 LLaMA 模型转换为 ggml 并进行 4 位量化的工具。
2. `local/llama.cpp:light-musa`：此镜像仅包含主可执行文件。
3. `local/llama.cpp:server-musa`：此镜像仅包含服务器可执行文件。

## 使用方法

本地构建后，使用方法与非 MUSA 示例类似，但您需要将 `mthreads` 设置为默认 Docker 运行时。这可以通过执行 `(cd /usr/bin/musa && sudo ./docker setup $PWD)` 来完成，并通过在主机上执行 `docker info | grep mthreads` 来验证更改。您还需要使用 `--n-gpu-layers` 标志。

```bash
docker run -v /path/to/models:/models local/llama.cpp:full-musa --run -m /models/7B/ggml-model-q4_0.gguf -p "Building a website can be done in 10 simple steps:" -n 512 --n-gpu-layers 1
docker run -v /path/to/models:/models local/llama.cpp:light-musa -m /models/7B/ggml-model-q4_0.gguf -p "Building a website can be done in 10 simple steps:" -n 512 --n-gpu-layers 1
docker run -v /path/to/models:/models local/llama.cpp:server-musa -m /models/7B/ggml-model-q4_0.gguf --port 8000 --host 0.0.0.0 -n 512 --n-gpu-layers 1
``` 