# 未发布：当前 develop（v0.5.0 之后）

移除独立子命令 `xyrev dotfile encrypt` 和 `xyrev dotfile decrypt`，并同步帮助、补全和 man pages。这是 v0.5.0 之后的兼容性变更，尚未发布新版本；已发布版本的二进制、tag 和安装命令保持原样。

- 控制项和模板指令统一使用 `.xyrev*` 与 `xyrev:template:`，不再自动读取旧命名。升级前需手动改名并检查引用，具体协议见 [设计文档](docs/dotfile-design.md#21-路径和控制文件)。这是待发布变更，已发布 v0.5.0 的安装包和行为不变。
- 三份 dotfile 文档统一放入 `docs/`：产品概览、实现说明与设计文档；内容以当前 Rust 实现为准。
- 旧脚本若调用这两个子命令，需要在升级前迁移。普通 dotfile 继续使用 `add --encrypt`、`edit` 和 `apply`；GPG 密钥继续使用原生 `gpg backup` / `gpg restore` 或自动恢复声明；其他文件加解密交给相应工具。
- 保留内部 age/GPG 加密、密码处理、`keygen`、既有密文及 GPG 备份兼容。删除 CLI 入口不要求重新备份或重新加密。
- 下方 v0.5.0 及更早版本是已发布历史，不能据其历史说明推断当前 develop 仍提供独立加解密入口。旧 GPG Shell 钩子应迁移到 v0.5.0 的原生恢复声明，不再调用独立加解密子命令。

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
