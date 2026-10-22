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
