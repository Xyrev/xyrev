# 待发布

## 版本变更

- 统一安装入口为 `install.sh`，移除重复安装脚本。
- 新增 macOS 的 `stool apple init`，独立设置 Dock 最小化动画，支持只读预览、按需写入和回读验证。
- 新增 `stool apple brew init/install`，独立初始化 Homebrew 并安装默认 `~/.Brewfile` 或指定清单，支持 prefix、只读预览和显式无人值守模式。
- 新增独立 `stool key backup/restore`：默认备份全部 GPG 与 SSH 密钥到仓库外的单个密码加密 `tar.age` 文件，手动恢复默认以 HOME 为目标；SSH 恢复无需 Git 或 GPG。

## 兼容性与升级注意事项

- 以上功能尚未发布，需[从源码构建](https://github.com/szwathub/stool/blob/develop/README.md#从源码构建)；历史 tag 和发行包保持原样。
- 删除旧 `dotfile gpg` 命令、自动恢复声明/状态及旧 GPG 备份格式兼容；普通 dotfile 加密保留。升级前先创建并隔离验证新的离线包，再清理旧仓库备份，见[离线密钥说明](https://github.com/szwathub/stool/blob/develop/README.md#密钥离线备份与恢复)。
- 示例 dotfiles 仅管理配置、签名指纹和 macOS 的 `~/.Brewfile`；移除原 Homebrew、软件包和 macOS 偏好钩子及对应开关。`apply/update` 不再安装软件、修改 Dock 或恢复密钥，需显式调用各独立命令；新机器流程统一见 [README](https://github.com/szwathub/stool/blob/develop/README.md#新-mac-初始化私有-dotfiles)。

# stool v0.6.0

## 版本变更

- 项目、仓库、Cargo package/bin、CLI、补全、man 和 Debian 元数据统一为 `stool` / `szwathub/stool`；六平台资源使用 `stool-v0.6.0-<target>`，包内程序为 `stool` / `stool.exe`。
- 移除独立 encrypt/decrypt 命令，保留普通 dotfile 的 `add --encrypt`、`edit`、`apply`、内部加密与 `keygen`。
- `stool dotfile gpg backup` 默认导出全部可导出的本机私钥、对应公钥和 ownertrust，`--fingerprint` 改为可选完整主指纹筛选；包含本机可用的私有子密钥及过期、撤销密钥。

## 兼容性与升级注意事项

- 控制项、模板、环境变量和默认目录改为 `.stool*`、`stool.*` / `stool:template:`、`STOOL_*` 和 `~/.stool`。不提供旧状态迁移命令；保留旧目录，以新协议 source 重新初始化并检查 diff 后部署，不隐式改写仓库或重加密备份。
- 本版本的 GPG 四文件格式兼容密码和独立 age 身份文件恢复；无可导出私钥或显式筛选不可导出密钥时失败。覆盖须保留既有主/子指纹，既有组含不可导出组件时拒绝覆盖，可另选新输出目录；硬件或缺失私钥仍有恢复限制。后续离线格式变更见[待发布说明](https://github.com/szwathub/stool/blob/develop/RELEASE_NOTES.md#待发布)。
- `install.sh` 仅接受 stool v0.6.0 起的稳定 Release，保留 SHA-256 校验和原子替换；历史 xyrev 的 tag、资源与校验和保持原样。安装入口统一见 [README](https://github.com/szwathub/stool/blob/develop/README.md#安装与版本边界)。

# xyrev v0.5.0

## 版本变更

- GPG 备份与恢复迁入 `xyrev dotfile gpg backup/restore`，支持多主密钥、隐藏密码输入、外部密码文件及独立 age 身份文件，移除 dotfiles 中对应 Shell 脚本。
- 原生校验摘要、公私钥完整集合和 ownertrust；密码和私钥明文不写入仓库、命令参数、环境变量或临时文件。
- 新增 `.xyrevgpg.toml[.tmpl]` 自动恢复声明，按 before、配置文件、GPG 恢复、after 顺序部署；成功记录摘要，失败可重试。预览不运行 GPG，独立 destination 跳过恢复。

## 兼容性与升级注意事项

- 兼容既有个人/Work 密码备份及历史单、多密钥 Shell 清单，清单仅作为数据解析，无需重新加密；原私钥口令仍由 pinentry 解锁。
- 历史安装与当前协议边界见 [README](https://github.com/szwathub/stool/blob/develop/README.md#历史版本安装)。

# xyrev v0.4.1

## 版本变更

- 新增密码加解密入口，以 age 密码加密 GPG 私钥和 ownertrust，支持隐藏输入与二次确认、外部密码文件或管道描述符；密码不进入参数、环境变量或仓库。
- 修复部署钩子内独立加解密的锁冲突，保留其他写操作互斥；新增 `XYREV_EXECUTABLE`，指向当前运行程序的绝对路径。
- dotfiles 恢复钩子新增摘要和私钥内容校验，失败可重试，重复部署不重复导入。

## 兼容性与升级注意事项

- 密码恢复无需独立 age 身份文件，原身份文件模式仍保留；原私钥口令不变。Windows 跳过 dotfiles 的 Bash GPG 恢复钩子。
- 这些独立加解密入口在 stool v0.6.0 移除，当前源码也已移除自动 GPG 恢复；旧版安装见 [README](https://github.com/szwathub/stool/blob/develop/README.md#历史版本安装)。

# xyrev v0.4.0

## 版本变更

- 首次提供 Git URL 初始化、预览、部署、校验和更新 dotfiles 的完整流程；缺少 Git 时提供安装引导。
- 支持路径属性、MiniJinja 模板、本机数据与初始化提示，before/after、once/onchange 脚本，外部资源与摘要校验、子模块、内置 Git、备份和冲突处理。
- 提供 add/edit、age/GPG 加密、秘密提供器、归档和诊断；修复安装器仓库与包名，发布六平台程序和 `SHA256SUMS.txt`。

## 兼容性与升级注意事项

- 安装程序无需 Git、Rust 或 Homebrew，校验成功后才替换已有程序；首次克隆需要 Git，GPG 操作需要本机 GPG。
- Windows 使用 ZIP 内的 `xyrev.exe`；历史安装见 [README](https://github.com/szwathub/stool/blob/develop/README.md#历史版本安装)。
