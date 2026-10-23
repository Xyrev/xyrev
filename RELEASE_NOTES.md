# stool v0.6.0

**待发布，当前开发版本。** 项目与命令从 xyrev 更名为 stool，代码仓库为 `szwathub/stool`。历史 xyrev v0.5.0 及更早版本的记录、脚本、资源和校验和保留原样。

- Cargo package/bin、CLI、五种补全、man、Debian 元数据、新安装器与六平台资源统一为 stool；新资源使用 `stool-v0.6.0-<target>`，包内 binary 为 `stool` / `stool.exe`，另包含 docs。
- 控制项、模板命名、环境变量和默认目录统一为 `.stool*`、`stool.*` / `stool:template:`、`STOOL_*`、`~/.stool`。旧环境使用显式迁移流程；不隐式改写用户仓库或重新加密 GPG 备份。
- 新增 `stool dotfile migrate [--from <old-control-dir>] [--dry-run]`，显式迁移旧本机控制目录；需先自行准备新协议 source，不联网或改 tracked 内容，不自动 apply，并保留旧目录供回退。
- 移除独立加解密子命令。普通 dotfile 继续通过 `add --encrypt`、`edit`、`apply` 管理，GPG 密钥继续使用原生备份/恢复；内部加密和 `keygen` 保留。
- `stool dotfile gpg backup` 默认备份全部可导出的本地私钥、对应公钥和 ownertrust；`--fingerprint` 改为可选的完整主指纹筛选。包含离线主密钥下的本机私有子密钥及过期/撤销本地密钥，提示缺失/硬件私钥的恢复限制；无可导出私钥或显式选择不可导出组时失败。覆盖保护要求保留既有主/子指纹，既有组含不可导出组件时保守拒绝覆盖，可保留旧备份并使用全新 `--output` 目录。四文件格式及密码/独立 age 身份文件恢复兼容保持不变。
- `install.sh` / `newinstall.sh` 同源，只安装 stool v0.6.0 起的稳定版本，保留校验、原子替换及错误时保护已有程序；历史版本继续使用其固定 tag 的安装器。

v0.6.0 发布后使用（当前尚不可下载此版本）：

```bash
curl -fsSL https://raw.githubusercontent.com/szwathub/stool/v0.6.0/install.sh | VERSION=v0.6.0 bash
export PATH="$HOME/.local/bin:$PATH"
stool --version
```

当前先[从源码构建](README.md#从源码构建)，新机器的 HTTPS 认证、初始化和 GPG 恢复见[设计文档](docs/dotfile-design.md#3-新-mac从公开安装到私有仓库恢复)。下方保留历史版本事实，不将旧 binary 或旧资源伪改为 stool。

# xyrev v0.5.0

GPG 密钥备份、校验和恢复统一迁入 xyrev，不再依赖 dotfiles 仓库中的备份/恢复脚本。兼容已提交的个人和 Work 密码加密备份，现有密文无需重新生成。

- 新增 `xyrev dotfile gpg backup` 和 `restore`，支持多主密钥、隐藏密码输入、外部密码文件及独立 age 身份文件。
- 原生校验摘要、公私钥完整集合及 ownertrust，防止遗漏既有密钥或导入不完整备份。密码和私钥明文不写入仓库、命令参数、环境变量或临时文件。
- `.xyrevgpg.toml[.tmpl]` 声明自动恢复；部署顺序为 before 钩子、普通配置、GPG 恢复、after 钩子。成功后记录摘要，失败可重试，重复部署保持幂等。
- 保留历史单/多密钥 Shell 清单读取兼容，清单仅作为数据解析。预览不运行 GPG 或询问密码；独立 destination 跳过自动恢复。

`Xyrev/xyrev` 和发行包保持公开；示例 `szwathub/dotfiles` 为私有仓库。首次初始化前先完成 SSH 或 HTTPS 凭据助手认证，并用 `git ls-remote` 确认仓库读取权限。Git 认证与后续 GPG 备份密码独立；不能依赖 `apply` 才安装的软件完成首次认证。完整新 Mac 步骤见 [README](README.md#新-mac-初始化私有-dotfiles)。

macOS 安装与初始化（以下使用已认证的 SSH）：

```bash
curl -fsSL https://raw.githubusercontent.com/Xyrev/xyrev/v0.5.0/install.sh | VERSION=v0.5.0 bash
export PATH="$HOME/.local/bin:$PATH"
xyrev --version
git ls-remote git@github.com:szwathub/dotfiles.git HEAD
xyrev dotfile init git@github.com:szwathub/dotfiles.git --recurse-submodules \
  --data git_signing_key=2BF609784E75AAC78CB67A7E465799B005224003
xyrev dotfile diff
sudo -v
xyrev dotfile apply
xyrev dotfile verify
```

现有机器升级 xyrev 后运行 `xyrev dotfile update`。启用恢复时在本机控制终端输入原备份密码；原 GPG 私钥如有口令，系统 pinentry 可能另行提示。只同步配置可使用 `--data restore_gpg=false`。安装 xyrev 无需 Rust；首次克隆仍需要 Git，GPG 操作需要本机 GPG（macOS 可由 dotfiles 安装 GPG Suite）。

# xyrev v0.4.1

支持在新机器上用密码恢复 GPG 备份，无需单独保存 age 身份文件。私钥和 ownertrust 通过 age 密码加密，恢复时经管道导入 GPG；密码不写入参数、环境变量或仓库。

- v0.4.1 当时新增独立的密码加解密入口，支持终端隐藏输入和二次确认，以及外部密码文件或管道描述符；当前 develop 已移除这些入口，迁移方式见本文开头。
- 修复部署钩子内独立加解密的锁冲突，保留其他写操作的互斥保护。
- 钩子获得当前运行程序的绝对路径 `XYREV_EXECUTABLE`，避免误调用 PATH 中旧版 xyrev。
- dotfiles 恢复钩子校验密文摘要和私钥内容，密码错误、备份损坏或导入失败时可修复后重试；重复部署不重复导入。
- 保留独立 age 身份文件模式，并提供 macOS、Linux、Windows 共六个发布包及 SHA256SUMS.txt。

macOS / Linux 安装：

```bash
curl -fsSL https://raw.githubusercontent.com/Xyrev/xyrev/v0.4.1/install.sh | VERSION=v0.4.1 bash
export PATH="$HOME/.local/bin:$PATH"
xyrev --version
xyrev dotfile init https://github.com/szwathub/dotfiles --recurse-submodules
xyrev dotfile diff
xyrev dotfile apply
xyrev dotfile verify
```

启用 GPG 恢复时，`apply` 会在本机终端提示输入备份密码；GPG 原密钥如有口令，GPG Suite 可能另行通过 pinentry 提示解锁。首次设置的备份密码请保存在密码管理器中，不要放进 Git、命令参数或聊天。只同步配置可添加 `--data restore_gpg=false`。

首次克隆前若提示缺少 Git，macOS 请先完成 `xcode-select --install`。Homebrew 和 GPG Suite 安装可能需要预先准备管理员权限。Windows 使用 ZIP 中的 xyrev.exe，dotfiles 的 Bash GPG 恢复钩子在 Windows 跳过。

# xyrev v0.4.0

首次发布完整的 Git 驱动 dotfiles 管理流程：从 Git URL 初始化仓库，预览、部署、校验和更新配置，并在缺少 Git 时提供安装引导。

- 支持 xyrev 路径属性、MiniJinja 模板、本机数据和初始化提示。
- 支持 before / after、once / onchange 脚本、外部资源与 SHA-256、子模块、内置 Git、备份和冲突处理。
- 提供 add / edit、age / GPG 加密、秘密提供器、归档及诊断命令。
- 修复安装脚本的仓库与包名，安装 xyrev 无需预装 Git、Rust 或 Homebrew；发布包校验成功后才替换已有程序。
- 发布 macOS Intel / Apple Silicon、Linux x86_64 glibc / musl 与 ARM64、Windows x86_64 二进制及 SHA256SUMS.txt。

macOS / Linux 安装：

```bash
curl -fsSL https://raw.githubusercontent.com/Xyrev/xyrev/v0.4.0/install.sh | VERSION=v0.4.0 bash
export PATH="$HOME/.local/bin:$PATH"
xyrev --version
xyrev dotfile init https://github.com/szwathub/dotfiles --recurse-submodules
xyrev dotfile diff
```

确认预览后运行 `xyrev dotfile apply`，然后使用 `xyrev dotfile verify` 校验结果。macOS 首次克隆若提示缺少 Git，请先完成 `xcode-select --install` 的系统安装，再重试初始化。Homebrew 和 GPG Suite 的安装可能需要预先准备管理员权限。

Windows 使用 `xyrev-v0.4.0-x86_64-pc-windows-msvc.zip` 中的 `xyrev.exe`。各平台发布包均可用 SHA256SUMS.txt 校验。
