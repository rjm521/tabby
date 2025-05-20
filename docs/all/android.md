# Android

## 使用 Termux 在 Android 上构建

[Termux](https://termux.dev/en/) 是一个 Android 终端模拟器和 Linux 环境应用（不需要 root 权限）。在撰写本文时，Termux 在 Google Play Store 中处于实验性阶段；否则，可以直接从项目仓库或 F-Droid 获取。

使用 Termux，您可以像在 Linux 环境中一样安装和运行 `llama.cpp`。在 Termux shell 中：

```
$ apt update && apt upgrade -y
$ apt install git cmake
```

然后，按照[构建说明](https://github.com/ggml-org/llama.cpp/blob/master/docs/build.md)，特别是 CMake 部分。

构建完成后，下载您选择的模型（例如，从 Hugging Face）。建议将其放在 `~/` 目录中以获得最佳性能：

```
$ curl -L {model-url} -o ~/{model}.gguf
```

然后，如果您还没有在仓库目录中，请 `cd` 进入 `llama.cpp` 并执行：

```
$ ./build/bin/llama-cli -m ~/{model}.gguf -c {context-size} -p "{your-prompt}"
```

这里我们展示了 `llama-cli`，但理论上 `examples` 下的任何可执行文件都应该可以工作。请确保将 `context-size` 设置为合理的数字（例如，4096）开始；否则，内存可能会激增并导致终端崩溃。

要了解它在视觉上的表现，这里有一个在 Pixel 5 手机上运行的交互式会话的旧演示：

https://user-images.githubusercontent.com/271616/225014776-1d567049-ad71-4ef2-b050-55b0b3b9274c.mp4

## 使用 Android NDK 交叉编译
可以通过 CMake 和 Android NDK 在您的主机系统上为 Android 构建 `llama.cpp`。如果您对此感兴趣，请确保您已经准备好为 Android 交叉编译程序的环境（即安装 Android SDK）。请注意，与桌面环境不同，Android 环境附带了一组有限的本地库，因此在使用 Android NDK 构建时，CMake 只能使用这些库（参见：https://developer.android.com/ndk/guides/stable_apis）。

准备好并克隆 `llama.cpp` 后，在项目目录中执行以下命令：

```
$ cmake \
  -DCMAKE_TOOLCHAIN_FILE=$ANDROID_NDK/build/cmake/android.toolchain.cmake \
  -DANDROID_ABI=arm64-v8a \
  -DANDROID_PLATFORM=android-28 \
  -DCMAKE_C_FLAGS="-march=armv8.7a" \
  -DCMAKE_CXX_FLAGS="-march=armv8.7a" \
  -DGGML_OPENMP=OFF \
  -DGGML_LLAMAFILE=OFF \
  -B build-android
```

注意事项：
  - 虽然较新版本的 Android NDK 附带 OpenMP，但它仍然需要由 CMake 作为依赖项安装，目前不支持
  - `llamafile` 似乎不支持 Android 设备（参见：https://github.com/Mozilla-Ocho/llamafile/issues/325）

上述命令应该为现代设备配置 `llama.cpp` 的最优选项。即使您的设备不是运行 `armv8.7a`，`llama.cpp` 也包含对可用 CPU 功能的运行时检查。

可以根据您的目标调整 Android ABI。项目配置完成后：

```
$ cmake --build build-android --config Release -j{n}
$ cmake --install build-android --prefix {install-dir} --config Release
```

安装后，继续在您的主机系统上下载您选择的模型。然后：

```
$ adb shell "mkdir /data/local/tmp/llama.cpp"
$ adb push {install-dir} /data/local/tmp/llama.cpp/
$ adb push {model}.gguf /data/local/tmp/llama.cpp/
$ adb shell
```

在 `adb shell` 中：

```
$ cd /data/local/tmp/llama.cpp
$ LD_LIBRARY_PATH=lib ./bin/llama-simple -m {model}.gguf -c {context-size} -p "{your-prompt}"
```

就是这样！

请注意，Android 不会自动找到 `lib` 库路径，因此我们必须指定 `LD_LIBRARY_PATH` 才能运行已安装的可执行文件。Android 在较新的 API 级别中支持 `RPATH`，所以这在未来可能会改变。有关 `context-size`（非常重要！）和运行其他 `examples` 的信息，请参考前面的部分。 