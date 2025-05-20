# 本地构建 llama.cpp

**获取代码：**

```bash
git clone https://github.com/ggml-org/llama.cpp
cd llama.cpp
```

以下部分描述了如何使用不同的后端和选项进行构建。

## CPU 构建

使用 `CMake` 构建 llama.cpp：

```bash
cmake -B build
cmake --build build --config Release
```

**注意事项**：

- 为了加快编译速度，可以添加 `-j` 参数来并行运行多个任务，或使用自动执行此操作的生成器（如 Ninja）。例如，`cmake --build build --config Release -j 8` 将并行运行 8 个任务。
- 为了加快重复编译速度，请安装 [ccache](https://ccache.dev/)
- 对于调试构建，有两种情况：

    1. 单配置生成器（例如默认的 `Unix Makefiles`；注意它们会忽略 `--config` 标志）：

       ```bash
       cmake -B build -DCMAKE_BUILD_TYPE=Debug
       cmake --build build
       ```

    2. 多配置生成器（`-G` 参数设置为 Visual Studio、XCode...）：

       ```bash
       cmake -B build -G "Xcode"
       cmake --build build --config Debug
       ```

    有关更多详细信息和支持的生成器列表，请参阅 [CMake 文档](https://cmake.org/cmake/help/latest/manual/cmake-generators.7.html)。
- 对于静态构建，添加 `-DBUILD_SHARED_LIBS=OFF`：
  ```
  cmake -B build -DBUILD_SHARED_LIBS=OFF
  cmake --build build --config Release
  ```

- 使用 MSVC 或 clang 编译器为 Windows（x86、x64 和 arm64）构建：
    - 安装 Visual Studio 2022，例如通过[社区版](https://visualstudio.microsoft.com/vs/community/)。在安装程序中，至少选择以下选项（这也会自动安装所需的额外工具，如 CMake 等）：
    - 工作负载选项卡：使用 C++ 的桌面开发
    - 组件选项卡（通过搜索快速选择）：Windows 的 C++-_CMake_ 工具、Windows 的 _Git_、Windows 的 C++-_Clang_ 编译器、LLVM-Toolset 的 MS-Build 支持（clang）
    - 请记住始终使用 VS2022 的开发者命令提示符/PowerShell 进行 git、构建、测试
    - 对于 Windows on ARM（arm64，WoA）构建：
    ```bash
    cmake --preset arm64-windows-llvm-release -D GGML_OPENMP=OFF
    cmake --build build-arm64-windows-llvm-release
    ```
    也可以使用 MSVC 编译器通过 build-arm64-windows-MSVC 预设或标准 CMake 构建指令为 arm64 构建。但是，请注意 MSVC 编译器不支持内联 ARM 汇编代码，例如用于加速的 Q4_0_N_M CPU 内核。

    使用 ninja 生成器和 clang 编译器作为默认值进行构建：
      - 设置路径：set LIB=C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\um\x64;C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC\14.41.34120\lib\x64\uwp;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\ucrt\x64
      ```bash
      cmake --preset x64-windows-llvm-release
      cmake --build build-x64-windows-llvm-release
      ```

## BLAS 构建

使用 BLAS 支持构建程序可能会在使用批量大小高于 32（默认为 512）的提示处理中带来一些性能改进。使用 BLAS 不会影响生成性能。目前有几种不同的 BLAS 实现可用于构建和使用：

### Accelerate Framework

这仅在 Mac PC 上可用，并且默认启用。您可以使用普通指令进行构建。

### OpenBLAS

这仅使用 CPU 提供 BLAS 加速。确保您的机器上安装了 OpenBLAS。

- 在 Linux 上使用 `CMake`：

    ```bash
    cmake -B build -DGGML_BLAS=ON -DGGML_BLAS_VENDOR=OpenBLAS
    cmake --build build --config Release
    ```

### BLIS

查看 [BLIS.md](./backend/BLIS.md) 获取更多信息。

### Intel oneMKL

通过 oneAPI 编译器构建将使不支持 avx512 和 avx512_vnni 的英特尔处理器可以使用 avx_vnni 指令集。请注意，此构建配置**不支持英特尔 GPU**。有关英特尔 GPU 支持，请参阅 [llama.cpp for SYCL](./backend/SYCL.md)。

- 使用手动 oneAPI 安装：
  默认情况下，`GGML_BLAS_VENDOR` 设置为 `Generic`，因此如果您已经获取了 intel 环境脚本并在 cmake 中指定了 `-DGGML_BLAS=ON`，将自动选择 mkl 版本的 Blas。否则，请安装 oneAPI 并按照以下步骤操作：
    ```bash
    source /opt/intel/oneapi/setvars.sh # 如果您在 oneapi-basekit docker 镜像中，可以跳过此步骤，仅手动安装时需要
    cmake -B build -DGGML_BLAS=ON -DGGML_BLAS_VENDOR=Intel10_64lp -DCMAKE_C_COMPILER=icx -DCMAKE_CXX_COMPILER=icpx -DGGML_NATIVE=ON
    cmake --build build --config Release
    ```

- 使用 oneAPI docker 镜像：
  如果您不想手动获取环境变量和安装 oneAPI，也可以使用 intel docker 容器构建代码：[oneAPI-basekit](https://hub.docker.com/r/intel/oneapi-basekit)。然后，您可以使用上面给出的命令。

查看 [在英特尔® CPU 上优化和运行 LLaMA2](https://www.intel.com/content/www/us/en/content-details/791610/optimizing-and-running-llama2-on-intel-cpu.html) 获取更多信息。

### 其他 BLAS 库

通过设置 `GGML_BLAS_VENDOR` 选项可以使用任何其他 BLAS 库。有关支持的供应商列表，请参阅 [CMake 文档](https://cmake.org/cmake/help/latest/module/FindBLAS.html#blas-lapack-vendors)。

## Metal 构建

在 MacOS 上，Metal 默认启用。使用 Metal 使计算在 GPU 上运行。
要在编译时禁用 Metal 构建，请使用 `-DGGML_METAL=OFF` cmake 选项。

当使用 Metal 支持构建时，您可以使用 `--n-gpu-layers 0` 命令行参数显式禁用 GPU 推理。

## SYCL

SYCL 是一个高级编程模型，用于提高各种硬件加速器上的编程效率。

基于 SYCL 的 llama.cpp 用于**支持英特尔 GPU**（数据中心 Max 系列、Flex 系列、Arc 系列、内置 GPU 和 iGPU）。

有关详细信息，请参阅 [llama.cpp for SYCL](./backend/SYCL.md)。

## CUDA

这使用 NVIDIA GPU 提供 GPU 加速。确保已安装 [CUDA 工具包](https://developer.nvidia.com/cuda-toolkit)。

#### 直接从 NVIDIA 下载
您可以在以下位置找到官方下载：[NVIDIA 开发者网站](https://developer.nvidia.com/cuda-downloads)。

#### 在 Fedora Toolbox 容器中编译和运行
我们还有一个[指南](./backend/CUDA-FEDORA.md)，用于在 Fedora [toolbox 容器](https://containertoolbx.org/)中设置 CUDA 工具包。

**推荐用于：**
- [Fedora 原子桌面](https://fedoraproject.org/atomic-desktops/)的用户***必需***；例如：[Silverblue](https://fedoraproject.org/atomic-desktops/silverblue/) 和 [Kinoite](https://fedoraproject.org/atomic-desktops/kinoite/)。
  -（这些系统没有支持的 CUDA 包）
- 主机不是[支持的 Nvidia CUDA 发布平台](https://developer.nvidia.com/cuda-downloads)的用户***必需***。
  -（例如，您的主机操作系统可能是 [Fedora 42 Beta](https://fedoramagazine.org/announcing-fedora-linux-42-beta/)）
- 对于运行 [Fedora Workstation](https://fedoraproject.org/workstation/) 或 [Fedora KDE Plasma Desktop](https://fedoraproject.org/spins/kde) 并希望保持主机系统清洁的用户***方便***。
- *可选*工具箱包可用于：[Arch Linux](https://archlinux.org/)、[Red Hat Enterprise Linux >= 8.5](https://www.redhat.com/en/technologies/linux-platforms/enterprise-linux) 或 [Ubuntu](https://ubuntu.com/download)

### 编译
```bash
cmake -B build -DGGML_CUDA=ON
cmake --build build --config Release
```

### 覆盖计算能力规范

如果 `nvcc` 无法检测到您的 GPU，您可能会收到如下编译警告：
 ```text
nvcc warning : Cannot find valid GPU for '-arch=native', default arch is used
```

要覆盖 `native` GPU 检测：

#### 1. 记下您的 NVIDIA 设备的`计算能力`：["CUDA：您的 GPU 计算能力"](https://developer.nvidia.com/cuda-gpus)。

```text
GeForce RTX 4090      8.9
GeForce RTX 3080 Ti   8.6
GeForce RTX 3070      8.6
```

#### 2. 在 `CMAKE_CUDA_ARCHITECTURES` 列表中手动列出每个不同的`计算能力`。

```bash
cmake -B build -DGGML_CUDA=ON -DCMAKE_CUDA_ARCHITECTURES="86;89"
```

### 运行时 CUDA 环境变量

您可以在运行时设置 [cuda 环境变量](https://docs.nvidia.com/cuda/cuda-c-programming-guide/index.html#env-vars)。

```bash
# 使用 `CUDA_VISIBLE_DEVICES` 隐藏第一个计算设备。
CUDA_VISIBLE_DEVICES="-0" ./build/bin/llama-server --model /srv/models/llama.gguf
```

### 统一内存

环境变量 `GGML_CUDA_ENABLE_UNIFIED_MEMORY=1` 可用于在 Linux 中启用统一内存。这允许在 GPU VRAM 耗尽时交换到系统 RAM，而不是崩溃。在 Windows 中，此设置在 NVIDIA 控制面板中可用，称为`系统内存回退`。

### 性能调优

以下编译选项也可用于调整性能：

| 选项                        | 合法值           | 默认值 | 描述                                                                                                                                                                                                                                                                             |
|-------------------------------|------------------------|---------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| GGML_CUDA_FORCE_MMQ           | Boolean                | false   | 强制使用自定义矩阵乘法内核进行量化模型，而不是 FP16 cuBLAS，即使没有 int8 张量核心实现可用（影响 V100、CDNA 和 RDNA3+）。在支持 int8 张量核心的 GPU 上默认启用 MMQ 内核。启用 MMQ 强制后，大批量大小的速度会更差，但 VRAM 消耗会更低。                       |
| GGML_CUDA_FORCE_CUBLAS        | Boolean                | false   | 强制使用 FP16 cuBLAS 而不是自定义矩阵乘法内核进行量化模型                                                                                                                                                                                       |
| GGML_CUDA_F16                 | Boolean                | false   | 如果启用，对 CUDA 反量化 + mul mat vec 内核以及 q4_1 和 q5_1 矩阵矩阵乘法内核使用半精度浮点运算。可以在相对较新的 GPU 上提高性能。                                                           |
| GGML_CUDA_PEER_MAX_BATCH_SIZE | Positive integer       | 128     | 启用多个 GPU 之间对等访问的最大批量大小。对等访问需要 Linux 或 NVLink。使用 NVLink 时，为更大的批量大小启用对等访问可能有益。                                                                         |
| GGML_CUDA_FA_ALL_QUANTS       | Boolean                | false   | 为 FlashAttention CUDA 内核编译所有 KV 缓存量化类型（组合）的支持。对 KV 缓存大小进行更细粒度的控制，但编译时间更长。                                                                                                  |

## MUSA

这使用摩尔线程 GPU 提供 GPU 加速。确保已安装 [MUSA SDK](https://developer.mthreads.com/musa/musa-sdk)。

#### 直接从摩尔线程下载

您可以在以下位置找到官方下载：[摩尔线程开发者网站](https://developer.mthreads.com/sdk/download/musa)。

### 编译

```bash
cmake -B build -DGGML_MUSA=ON
cmake --build build --config Release
```

#### 覆盖计算能力规范

默认情况下，启用所有支持的计算能力。要自定义此行为，您可以在 CMake 命令中指定 `MUSA_ARCHITECTURES` 选项：

```bash
cmake -B build -DGGML_MUSA=ON -DMUSA_ARCHITECTURES="21"
cmake --build build --config Release
```

此配置在编译期间仅启用计算能力 `2.1`（MTT S80），这可以帮助减少编译时间。

#### 编译选项

大多数可用于 CUDA 的编译选项也应该可用于 MUSA，尽管它们尚未经过充分测试。

- 对于静态构建，添加 `-DBUILD_SHARED_LIBS=OFF` 和 `-DCMAKE_POSITION_INDEPENDENT_CODE=ON`：
  ```
  cmake -B build -DGGML_MUSA=ON \
    -DBUILD_SHARED_LIBS=OFF -DCMAKE_POSITION_INDEPENDENT_CODE=ON
  cmake --build build --config Release
  ```

### 运行时 MUSA 环境变量

您可以在运行时设置 [musa 环境变量](https://docs.mthreads.com/musa-sdk/musa-sdk-doc-online/programming_guide/Z%E9%99%84%E5%BD%95/)。

```bash
# 使用 `MUSA_VISIBLE_DEVICES` 隐藏第一个计算设备。
MUSA_VISIBLE_DEVICES="-0" ./build/bin/llama-server --model /srv/models/llama.gguf
```

### 统一内存

环境变量 `GGML_CUDA_ENABLE_UNIFIED_MEMORY=1` 可用于在 Linux 中启用统一内存。这允许在 GPU VRAM 耗尽时交换到系统 RAM，而不是崩溃。

## HIP

这在使用 HIP 支持的 AMD GPU 上提供 GPU 加速。
确保已安装 ROCm。
您可以从 Linux 发行版的包管理器或从以下位置下载：[ROCm 快速入门（Linux）](https://rocm.docs.amd.com/projects/install-on-linux/en/latest/tutorial/quick-start.html#rocm-install-quick)。

- 在 Linux 上使用 `CMake`（假设使用兼容 gfx1030 的 AMD GPU）：
  ```bash
  HIPCXX="$(hipconfig -l)/clang" HIP_PATH="$(hipconfig -R)" \
      cmake -S . -B build -DGGML_HIP=ON -DAMDGPU_TARGETS=gfx1030 -DCMAKE_BUILD_TYPE=Release \
      && cmake --build build --config Release -- -j 16
  ```

  要在 RDNA3+ 或 CDNA 架构上增强 flash attention 性能，您可以通过启用 `-DGGML_HIP_ROCWMMA_FATTN=ON` 选项来利用 rocWMMA 库。这需要在构建系统上安装 rocWMMA 头文件。

  当使用 AMD 提供的 `rocm` 元包安装 ROCm SDK 时，默认包含 rocWMMA 库。或者，如果您不使用元包，可以使用 `rocwmma-dev` 或 `rocwmma-devel` 包安装库，具体取决于您系统的包管理器。

  作为替代方案，您可以通过从官方 [GitHub 仓库](https://github.com/ROCm/rocWMMA)克隆它，检出相应的版本标签（例如 `rocm-6.2.4`）并在 CMake 中设置 `-DCMAKE_CXX_FLAGS="-I<path/to/rocwmma>/library/include/"` 来手动安装库。尽管 AMD 官方不支持，但这在 Windows 下也有效。

  请注意，如果您收到以下错误：
  ```
  clang: error: cannot find ROCm device library; provide its path via '--rocm-path' or '--rocm-device-lib-path', or pass '-nogpulib' to build without ROCm device library
  ```
  尝试在 `HIP_PATH` 下搜索包含文件 `oclc_abi_version_400.bc` 的目录。然后，在命令开头添加以下内容：`HIP_DEVICE_LIB_PATH=<directory-you-just-found>`，例如：
  ```bash
  HIPCXX="$(hipconfig -l)/clang" HIP_PATH="$(hipconfig -p)" \
  HIP_DEVICE_LIB_PATH=<directory-you-just-found> \
      cmake -S . -B build -DGGML_HIP=ON -DAMDGPU_TARGETS=gfx1030 -DCMAKE_BUILD_TYPE=Release \
      && cmake --build build -- -j 16
  ```

- 在 Windows 上使用 `CMake`（使用 VS 的 x64 本机工具命令提示符，假设使用兼容 gfx1100 的 AMD GPU）：
  ```bash
  set PATH=%HIP_PATH%\bin;%PATH%
  cmake -S . -B build -G Ninja -DAMDGPU_TARGETS=gfx1100 -DGGML_HIP=ON -DCMAKE_C_COMPILER=clang -DCMAKE_CXX_COMPILER=clang++ -DCMAKE_BUILD_TYPE=Release
  cmake --build build
  ```
  确保 `AMDGPU_TARGETS` 设置为要编译的 GPU 架构。上面的示例使用 `gfx1100`，对应于 Radeon RX 7900XTX/XT/GRE。您可以在[这里](https://llvm.org/docs/AMDGPUUsage.html#processors)找到目标列表
  通过将 `rocminfo | grep gfx | head -1 | awk '{print $2}'` 的输出与处理器列表匹配来找到您的 GPU 版本字符串，例如 `gfx1035` 映射到 `gfx1030`。

环境变量 [`HIP_VISIBLE_DEVICES`](https://rocm.docs.amd.com/en/latest/understand/gpu_isolation.html#hip-visible-devices) 可用于指定将使用哪些 GPU。
如果您的 GPU 不受官方支持，您可以使用环境变量 [`HSA_OVERRIDE_GFX_VERSION`] 设置为类似的 GPU，例如 RDNA2 上的 10.3.0（例如 gfx1030、gfx1031 或 gfx1035）或 RDNA3 上的 11.0.0。

### 统一内存

在 Linux 上，可以通过设置环境变量 `GGML_CUDA_ENABLE_UNIFIED_MEMORY=1` 使用统一内存架构（UMA）在 CPU 和集成 GPU 之间共享主内存。但是，这会降低非集成 GPU 的性能（但启用与集成 GPU 一起工作）。

## Vulkan

**Windows**

### w64devkit

下载并解压 [`w64devkit`](https://github.com/skeeto/w64devkit/releases)。

下载并安装 [`Vulkan SDK`](https://vulkan.lunarg.com/sdk/home#windows)，使用默认设置。

启动 `w64devkit.exe` 并运行以下命令复制 Vulkan 依赖项：
```sh
SDK_VERSION=1.3.283.0
cp /VulkanSDK/$SDK_VERSION/Bin/glslc.exe $W64DEVKIT_HOME/bin/
cp /VulkanSDK/$SDK_VERSION/Lib/vulkan-1.lib $W64DEVKIT_HOME/x86_64-w64-mingw32/lib/
cp -r /VulkanSDK/$SDK_VERSION/Include/* $W64DEVKIT_HOME/x86_64-w64-mingw32/include/
cat > $W64DEVKIT_HOME/x86_64-w64-mingw32/lib/pkgconfig/vulkan.pc <<EOF
Name: Vulkan-Loader
Description: Vulkan Loader
Version: $SDK_VERSION
Libs: -lvulkan-1
EOF
```

切换到 `llama.cpp` 目录并使用 CMake 构建。
```sh
cmake -B build -DGGML_VULKAN=ON
cmake --build build --config Release
```

### Git Bash MINGW64

下载并安装 [`Git-SCM`](https://git-scm.com/downloads/win)，使用默认设置

下载并安装 [`Visual Studio Community Edition`](https://visualstudio.microsoft.com/)，确保选择 `C++`

下载并安装 [`CMake`](https://cmake.org/download/)，使用默认设置

下载并安装 [`Vulkan SDK`](https://vulkan.lunarg.com/sdk/home#windows)，使用默认设置。

进入您的 `llama.cpp` 目录，右键单击，选择 `Open Git Bash Here`，然后运行以下命令

```
cmake -B build -DGGML_VULKAN=ON
cmake --build build --config Release
```

现在您可以在对话模式下使用 `Vulkan` 加载模型

```sh
build/bin/Release/llama-cli -m "[PATH TO MODEL]" -ngl 100 -c 16384 -t 10 -n -2 -cnv
```

### MSYS2
安装 [MSYS2](https://www.msys2.org/)，然后在 UCRT 终端中运行以下命令安装依赖项。
```sh
pacman -S git \
    mingw-w64-ucrt-x86_64-gcc \
    mingw-w64-ucrt-x86_64-cmake \
    mingw-w64-ucrt-x86_64-vulkan-devel \
    mingw-w64-ucrt-x86_64-shaderc
```

切换到 `llama.cpp` 目录并使用 CMake 构建。
```sh
cmake -B build -DGGML_VULKAN=ON
cmake --build build --config Release
```

**使用 docker**：

您不需要安装 Vulkan SDK。它将在容器内安装。

```sh
# 构建镜像
docker build -t llama-cpp-vulkan --target light -f .devops/vulkan.Dockerfile .

# 然后，使用它：
docker run -it --rm -v "$(pwd):/app:Z" --device /dev/dri/renderD128:/dev/dri/renderD128 --device /dev/dri/card1:/dev/dri/card1 llama-cpp-vulkan -m "/app/models/YOUR_MODEL_FILE" -p "Building a website can be done in 10 simple steps:" -n 400 -e -ngl 33
```

**不使用 docker**：

首先，您需要确保已安装 [Vulkan SDK](https://vulkan.lunarg.com/doc/view/latest/linux/getting_started_ubuntu.html)

例如，在 Ubuntu 22.04（jammy）上，使用以下命令：

```bash
wget -qO - https://packages.lunarg.com/lunarg-signing-key-pub.asc | apt-key add -
wget -qO /etc/apt/sources.list.d/lunarg-vulkan-jammy.list https://packages.lunarg.com/vulkan/lunarg-vulkan-jammy.list
apt update -y
apt-get install -y vulkan-sdk
# 要验证安装，使用以下命令：
vulkaninfo
```

或者，您的包管理器可能能够提供适当的库。
例如，对于 Ubuntu 22.04，您可以安装 `libvulkan-dev`。
对于 Fedora 40，您可以安装 `vulkan-devel`、`glslc` 和 `glslang` 包。

然后，使用以下 cmake 命令构建 llama.cpp：

```bash
cmake -B build -DGGML_VULKAN=1
cmake --build build --config Release
# 测试输出二进制文件（使用 "-ngl 33" 将所有层卸载到 GPU）
./bin/llama-cli -m "PATH_TO_MODEL" -p "Hi you how are you" -n 50 -e -ngl 33 -t 4

# 您应该在输出中看到，ggml_vulkan 检测到您的 GPU。例如：
# ggml_vulkan: Using Intel(R) Graphics (ADL GT2) | uma: 1 | fp16: 1 | warp size: 32
```

## CANN
这使用您的昇腾 NPU 的 AI 核心提供 NPU 加速。而 [CANN](https://www.hiascend.com/en/software/cann) 是一个分层 API，可帮助您快速构建基于昇腾 NPU 的 AI 应用程序和服务。

有关昇腾 NPU 的更多信息，请访问[昇腾社区](https://www.hiascend.com/en/)。

确保已安装 CANN 工具包。您可以从以下位置下载：[CANN 工具包](https://www.hiascend.com/developer/download/community/result?module=cann)

进入 `llama.cpp` 目录并使用 CMake 构建。
```bash
cmake -B build -DGGML_CANN=on -DCMAKE_BUILD_TYPE=release
cmake --build build --config release
```

您可以使用以下命令测试：

```bash
./build/bin/llama-cli -m PATH_TO_MODEL -p "Building a website can be done in 10 steps:" -ngl 32
```

如果屏幕上输出以下信息，您正在使用带有 CANN 后端的 `llama.cpp`：
```bash
llm_load_tensors:       CANN model buffer size = 13313.00 MiB
llama_new_context_with_model:       CANN compute buffer size =  1260.81 MiB
```

有关详细信息，例如模型/设备支持、CANN 安装，请参阅 [llama.cpp for CANN](./backend/CANN.md)。

## Arm® KleidiAI™
KleidiAI 是一个针对 AI 工作负载优化的微内核库，专为 Arm CPU 设计。这些微内核可以提高性能，可以为 CPU 后端启用。

要启用 KleidiAI，进入 llama.cpp 目录并使用 CMake 构建
```bash
cmake -B build -DGGML_CPU_KLEIDIAI=ON
cmake --build build --config Release
```
您可以通过运行以下命令验证是否正在使用 KleidiAI
```bash
./build/bin/llama-cli -m PATH_TO_MODEL -p "What is a car?"
```
如果启用了 KleidiAI，输出将包含类似以下的行：
```
load_tensors: CPU_KLEIDIAI model buffer size =  3474.00 MiB
```
KleidiAI 的微内核使用 Arm CPU 功能（如 dotprod、int8mm 和 SME）实现优化的张量操作。llama.cpp 基于运行时 CPU 功能检测选择最高效的内核。但是，在支持 SME 的平台上，您必须通过设置环境变量 `GGML_KLEIDIAI_SME=1` 手动启用 SME 微内核。

根据您的构建目标，其他更高优先级的后端可能默认启用。要确保使用 CPU 后端，您必须在编译时禁用更高优先级的后端，例如 -DGGML_METAL=OFF，或使用命令行选项 `--device none` 在运行时禁用。

## OpenCL

这通过 OpenCL 在最近的 Adreno GPU 上提供 GPU 加速。
有关 OpenCL 后端的更多信息，请参阅 [OPENCL.md](./backend/OPENCL.md)。

### Android

假设 NDK 在 `$ANDROID_NDK` 中可用。首先，如果不可用，安装 OpenCL 头文件和 ICD 加载器库，

```sh
mkdir -p ~/dev/llm
cd ~/dev/llm

git clone https://github.com/KhronosGroup/OpenCL-Headers && \
cd OpenCL-Headers && \
cp -r CL $ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/include

cd ~/dev/llm

git clone https://github.com/KhronosGroup/OpenCL-ICD-Loader && \
cd OpenCL-ICD-Loader && \
mkdir build_ndk && cd build_ndk && \
cmake .. -G Ninja -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK/build/cmake/android.toolchain.cmake \
  -DOPENCL_ICD_LOADER_HEADERS_DIR=$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/include \
  -DANDROID_ABI=arm64-v8a \
  -DANDROID_PLATFORM=24 \
  -DANDROID_STL=c++_shared && \
ninja && \
cp libOpenCL.so $ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/aarch64-linux-android
```

然后使用 OpenCL 启用构建 llama.cpp，

```sh
cd ~/dev/llm

git clone https://github.com/ggml-org/llama.cpp && \
cd llama.cpp && \
mkdir build-android && cd build-android

cmake .. -G Ninja \
  -DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK/build/cmake/android.toolchain.cmake \
  -DANDROID_ABI=arm64-v8a \
  -DANDROID_PLATFORM=android-28 \
  -DBUILD_SHARED_LIBS=OFF \
  -DGGML_OPENCL=ON

ninja
```

### Windows Arm64

首先，如果不可用，安装 OpenCL 头文件和 ICD 加载器库，

```powershell
mkdir -p ~/dev/llm

cd ~/dev/llm
git clone https://github.com/KhronosGroup/OpenCL-Headers && cd OpenCL-Headers
mkdir build && cd build
cmake .. -G Ninja `
  -DBUILD_TESTING=OFF `
  -DOPENCL_HEADERS_BUILD_TESTING=OFF `
  -DOPENCL_HEADERS_BUILD_CXX_TESTS=OFF `
  -DCMAKE_INSTALL_PREFIX="$HOME/dev/llm/opencl"
cmake --build . --target install

cd ~/dev/llm
git clone https://github.com/KhronosGroup/OpenCL-ICD-Loader && cd OpenCL-ICD-Loader
mkdir build && cd build
cmake .. -G Ninja `
  -DCMAKE_BUILD_TYPE=Release `
  -DCMAKE_PREFIX_PATH="$HOME/dev/llm/opencl" `
  -DCMAKE_INSTALL_PREFIX="$HOME/dev/llm/opencl"
cmake --build . --target install
```

然后使用 OpenCL 启用构建 llama.cpp，

```powershell
cmake .. -G Ninja `
  -DCMAKE_TOOLCHAIN_FILE="$HOME/dev/llm/llama.cpp/cmake/arm64-windows-llvm.cmake" `
  -DCMAKE_BUILD_TYPE=Release `
  -DCMAKE_PREFIX_PATH="$HOME/dev/llm/opencl" `
  -DBUILD_SHARED_LIBS=OFF `
  -DGGML_OPENCL=ON
ninja
```

## Android

要阅读有关如何在 Android 上构建的文档，[点击这里](./android.md)

## 关于 GPU 加速后端的注意事项

即使使用 `-ngl 0` 选项，GPU 仍可能用于加速计算的某些部分。您可以使用 `--device none` 完全禁用 GPU 加速。

在大多数情况下，可以同时构建和使用多个后端。例如，您可以通过使用 CMake 的 `-DGGML_CUDA=ON -DGGML_VULKAN=ON` 选项同时构建具有 CUDA 和 Vulkan 支持的 llama.cpp。在运行时，您可以使用 `--device` 选项指定要使用的后端设备。要查看可用设备列表，请使用 `--list-devices` 选项。

后端可以构建为可以在运行时动态加载的动态库。这允许您在不同 GPU 的不同机器上使用相同的 llama.cpp 二进制文件。要启用此功能，请在构建时使用 `GGML_BACKEND_DL` 选项。