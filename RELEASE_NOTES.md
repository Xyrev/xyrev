# xyrev v0.4.1

支持在新机器上用密码恢复 GPG 备份，无需单独保存 age 身份文件。私钥和 ownertrust 通过 age 密码加密，恢复时经管道导入 GPG；密码不写入参数、环境变量或仓库。

- 新增 `dotfile encrypt/decrypt --passphrase`，在本机终端隐藏输入；加密时二次确认。脚本可使用 `--passphrase-file` 读取文件或管道描述符。
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

- 支持 chezmoi 风格路径属性、MiniJinja 模板、本机数据和初始化提示。
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
