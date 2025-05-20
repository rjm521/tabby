# 安装预构建版本的 llama.cpp

## Homebrew

在 Mac 和 Linux 上，可以使用 Homebrew 包管理器通过以下命令安装：

```sh
brew install llama.cpp
```

该公式会随着 `llama.cpp` 的新版本发布自动更新。更多信息：https://github.com/ggml-org/llama.cpp/discussions/7668

## MacPorts

```sh
sudo port install llama.cpp
```

更多信息：https://ports.macports.org/port/llama.cpp/details/

## Nix

在 Mac 和 Linux 上，可以使用 Nix 包管理器通过以下命令安装：

```sh
nix profile install nixpkgs#llama-cpp
```

适用于启用 flake 的安装。

或者

```sh
nix-env --file '<nixpkgs>' --install --attr llama-cpp
```

适用于未启用 flake 的安装。

此表达式在 [nixpkgs 仓库](https://github.com/NixOS/nixpkgs/blob/nixos-24.05/pkgs/by-name/ll/llama-cpp/package.nix#L164) 中自动更新。

## Flox

在 Mac 和 Linux 上，可以使用 Flox 在 Flox 环境中安装 llama.cpp：

```sh
flox install llama-cpp
```

Flox 遵循 nixpkgs 的 llama.cpp 构建。 