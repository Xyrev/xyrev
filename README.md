# stool

开发环境命令行工具。macOS 的 `stool apple init` 独立管理系统偏好；`stool dotfile` 从 Git URL 初始化、预览、部署和更新配置；`stool key` 独立管理仓库外的 GPG 与 SSH 加密离线备份和手动恢复。

## 当前源码与版本边界

项目、Cargo package/bin 和仓库统一为 `stool` / `szwathub/stool`，当前 Cargo 版本为 0.6.0。本次 `stool key` 和 macOS 的 `stool apple init` / `stool apple brew` 属于待发布改动，历史 tag 与发行包不因此改变；先确认 `stool key --help`，需要时[从源码构建](#从源码构建)。

控制项、模板数据/指令、环境变量和默认目录使用 `.stool*`、`stool.*` / `stool:template:`、`STOOL_*` 和 `~/.stool`。不自动读取旧命名，也不复制旧状态。旧环境需重新初始化并检查 source、资源引用、待执行脚本和目标冲突。

旧 `dotfile gpg` 命令、自动恢复声明和旧 GPG 备份格式已移除。先创建并验证新的离线包，再清理仓库；不保留旧备份兼容。普通配置的 `add --encrypt`、`edit`、`apply`、内部 age/GPG 和 `dotfile keygen` 保留，独立 encrypt/decrypt 命令仍不提供。

## 安装与版本边界

发布包安装器 `install.sh` 按平台下载已编译 binary 并校验 SHA-256，无需先安装 Rust、Git 或 Homebrew。`VERSION` 选择已发布版本，`PREFIX` 选择安装前缀（binary 在 `$PREFIX/bin/stool`），新安装器只接受 stool v0.6.0 起的稳定 Release；`latest` 仍指向历史包时会停止。源码更新不等于新包已经发布。

macOS / Linux 从 [Releases](https://github.com/szwathub/stool/releases) 选择已发布且资产完整的稳定版本，将下方 `vX.Y.Z` 替换为该版本。固定同一 tag 的安装器和 `VERSION`，避免源码与发行包混用；`PREFIX` 可在运行安装器时另行指定。

```bash
stool_version=vX.Y.Z
curl -fsSL "https://raw.githubusercontent.com/szwathub/stool/$stool_version/install.sh" \
  | VERSION="$stool_version" bash
export PATH="$HOME/.local/bin:$PATH"
stool --version
```

PATH 设置只影响当前终端；zsh 可将 export 加入 `~/.zprofile`，Fish 可用 `fish_add_path ~/.local/bin`。指定其他 `PREFIX` 时将对应的 `bin` 目录加入 PATH。Windows 下载对应 ZIP 和 `SHA256SUMS.txt`，校验后解压使用其中的 `stool.exe`。

使用离线密钥功能前运行 `stool key --help` 确认命令可用；旧发行包不包含当前待发布功能，需要时[从源码构建](#从源码构建)。也可在旧机器构建后将适配新机架构的 binary 转入新机，无需在新机安装 Rust。安装完成后的 Git 认证、dotfiles 部署和 GPG/SSH 恢复统一见[新 Mac 流程](#新-mac-初始化私有-dotfiles)，各版本差异见[版本说明](RELEASE_NOTES.md)。

### 历史版本安装

xyrev v0.4.0、v0.4.1、v0.5.0 使用各自固定 tag 的 `install.sh`，程序名为 `xyrev`，不能用当前 stool 安装器选择这些版本。macOS / Linux 示例：

```bash
xyrev_version=v0.5.0  # 也可选择 v0.4.1 或 v0.4.0
curl -fsSL "https://raw.githubusercontent.com/szwathub/stool/$xyrev_version/install.sh" \
  | VERSION="$xyrev_version" bash
export PATH="$HOME/.local/bin:$PATH"
xyrev --version
```

Windows 从相应历史 Release 下载 ZIP，校验后使用其中的 `xyrev.exe`。历史资源、校验和与安装器保持原样；旧安装器内记录的原仓库地址依赖 GitHub 重定向。旧命令、自动 GPG 恢复和 dotfiles 格式以对应 tag 的 [v0.5.0 文档](https://github.com/szwathub/stool/blob/v0.5.0/README.md)、[v0.4.1 文档](https://github.com/szwathub/stool/blob/v0.4.1/README.md)、[v0.4.0 文档](https://github.com/szwathub/stool/blob/v0.4.0/README.md)为准；当前 `.stool*` 仓库和 `stool key` 流程不适用于旧程序。新机器使用当前流程时先选择包含所需命令的 stool 构建。

## macOS 偏好初始化

此命令仅在 macOS 提供，使用包含 `apple init` 的当前源码构建；旧发行包不会自动增加此入口。安装 stool 后即可独立执行，无需 Git、Homebrew 或 dotfiles：

```bash
stool apple init --help
stool apple init --dry-run
stool apple init
```

当前只设置当前用户的 Dock 最小化动画：`com.apple.dock mineffect` 为字符串 `suck`，沿用原 dotfiles 脚本的值。预览读取当前值并展示计划，不写入；实际运行仅在值不匹配或尚未设置时写入，并回读确认成功。重复运行重新检查真实设置，手动改动后可再次应用，不依赖 dotfile 脚本成功记录。

通过 macOS 自带的 `/usr/bin/defaults` 执行，无需 sudo；命令不会自动重启 Dock，必要时自行重启 Dock 或退出登录后重新进入。读取、写入或回读验证失败返回非零，错误包含具体偏好及系统诊断。

原 `run_onchange_after_20-macos.sh.tmpl` 和 `configure_macos` 已从示例 dotfiles 移除；旧本机 `[data].configure_macos` 不再生效，可手动删除。移除钩子不撤销已有 Dock 设置，后续 `stool dotfile apply/update` 也不自动调用 `apple init`。Homebrew/软件包安装及离线密钥操作均显式调用独立命令。

## Homebrew 与软件包

仅 macOS 提供，需使用包含 `apple brew` 的当前源码构建；本次只提交推送，不改变旧发行包。

```bash
stool apple brew init --dry-run
stool apple brew init
# 按输出执行 shellenv，使当前终端可以找到 brew、gpg 等程序
stool apple brew install --dry-run
stool apple brew install
# 也可选择其他可信清单
stool apple brew install --file /path/to/Brewfile
```

`init` 不依赖 dotfiles。发现已有 Homebrew 时检查版本和实际 prefix 并复用；缺失时先检查 Xcode/Command Line Tools、系统 Git 和 curl，再通过 HTTPS 下载官方安装器到私有临时文件，以 `/bin/bash` 执行，完成后验证可执行文件、版本与 prefix。缺失前置会停止并给出修复指引，不自动弹出 CLT 安装器；`xcode-select --install` 完成后重试。下载、安装或验证失败均返回非零，临时安装器始终清理；已有损坏的 Homebrew 报错后需先修复。

选择顺序为 `--prefix <绝对目录>`、`HOMEBREW_PREFIX`、PATH 中的 brew、当前二进制架构的默认目录（Apple Silicon `/opt/homebrew`，Intel `/usr/local`）。例如 `stool apple brew init --prefix /custom/homebrew`；install 同样支持该选项。安装后输出实际路径对应的 zsh/bash shellenv 命令，由用户在当前终端执行；工具不改写 shell 配置。dotfiles 的 `brew_prefix` 仍供 Fish 配置使用，应与实际安装一致。

默认继承终端输入，官方安装器和 cask 可正常请求授权。没有终端时实际安装须显式加 `--non-interactive`，并事先准备所需权限；不使用 sudo 启动整个 stool。详情见 [官方安装说明](https://docs.brew.sh/Installation)。

`install` 默认读取 `~/.Brewfile`，也可用 `--file` 指定。文件缺失、不可读或不是普通文件时，在任何安装动作前失败；成功读取后确保 Homebrew 可用，再执行 `brew bundle install --no-upgrade --file <文件>`。保留禁用自动更新、常规升级与自动清理的行为；依赖安装仍由 Homebrew 决定，不提供版本锁定。失败保留已完成安装，修复后重新运行。每次显式调用都检查真实状态，不以清单摘要或 dotfile 脚本记录跳过，所以清单新增或手动卸载软件后可重新安装。

严格预览仅检查 Homebrew/前置、读取并展示清单和计划，不下载、不创建安装器、不安装，也不执行 `brew bundle check`。Brewfile 可包含 Ruby；实际安装只用于可信清单，预览不会执行其中的代码，也不承诺准确列出所有缺失包。见 [Homebrew Bundle 说明](https://docs.brew.sh/Brew-Bundle-and-Brewfile)。

示例 dotfiles 将 `dot_Brewfile` 部署到 macOS 的 `~/.Brewfile`，保留 GPG Suite，移除两个 Homebrew/软件包钩子及共享 `macos-hook.sh`。旧 `install_homebrew` / `install_packages` 本机数据可手动删除；配置 apply/update/verify 不再安装软件，清单更新后需显式运行 `stool apple brew install`。Git/Fish 仍使用 `quote` 宏，Fish 的 `brew_prefix` 保留。

## 从旧环境迁移

先在旧机器运行 `stool key backup` 到仓库外并隔离验证，保存旧 source 的未提交改动与本机配置。私库按下文完成认证后重新 init、查看 status/diff，再部署；不直接拷贝旧文件基线和脚本成功记录。旧控制目录和现有密钥不会自动删除。已初始化的 stool 环境可删除本机 `[data]` 中旧的恢复开关与凭据路径；配置 update 不再恢复密钥。

## 新 Mac 初始化私有 dotfiles

`szwathub/stool` 公开，`szwathub/dotfiles` 私有。Git 认证、外层离线备份密码、各 GPG/SSH 私钥原口令和管理员密码分别管理。

以下命令在 macOS 的 zsh 中执行。先安装包含 `stool key` 的构建，并确认 `stool key --help` 可用；本次离线密钥和 Apple 命令属于待发布源码，旧 tag 的安装器和 binary 不会自动获得这些命令。当前可[从源码构建](#从源码构建)，正式发布包安装不需要 Rust。

### 先转入离线备份并恢复

安装 stool 后即可通过移动硬盘等方式转入加密包，不需要先下载 dotfiles。没有 GPG 时先恢复 SSH：

```bash
stool key restore --ssh --input /Volumes/Backup/my-keys.tar.age
```

将示例路径替换为实际文件；终端隐藏输入备份密码。目标默认为当前 HOME，无需 `--output "$HOME"`；只恢复 SSH 不需要 Git 或 GPG。已有 GPG 时可省略 `--ssh`，一次恢复包内全部 GPG 和 SSH 密钥。恢复保留原私钥口令，遇到同名不同内容的 SSH 文件会停止，先自行解决冲突。

### 准备 Git 和私库认证

`git --version` 不可用时，运行 `xcode-select --install`，在系统窗口完成 Command Line Tools 安装，再检查 Git。恢复的 `github_rsa` 在配置部署前需显式选取：

```bash
[ -n "${SSH_AUTH_SOCK:-}" ] || eval "$(/usr/bin/ssh-agent -s)"
/usr/bin/ssh-add "$HOME/.ssh/github_rsa"
git -c core.sshCommand='ssh -i ~/.ssh/github_rsa -o IdentitiesOnly=yes' \
  ls-remote git@github.com:szwathub/dotfiles.git HEAD
```

原公钥必须仍获有仓库访问权的 GitHub 账号授权；恢复私钥不会增加账号权限。首次连接对照 [GitHub 主机指纹](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/githubs-ssh-key-fingerprints)，不要关闭主机校验。私钥原口令与备份密码独立；macOS 如需保存在 Keychain，可使用 `/usr/bin/ssh-add --apple-use-keychain`。

也可完全使用 HTTPS。在 GitHub 创建仅授权目标私库、具有 Contents 读取权限的 [fine-grained PAT](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)，再于普通交互终端缓存凭据：

```bash
git config --global credential.https://github.com.helper osxkeychain
git ls-remote https://github.com/szwathub/dotfiles.git HEAD
```

Username 输入 GitHub 用户名，Password 输入 PAT。GitHub 账户密码不能替代 PAT；凭据不放在 URL、参数、环境变量、仓库或聊天中。浏览器登录不会自动认证 Git。系统 Git 后端关闭终端凭据提示，须先完成 `ls-remote` 预检；推送配置另需写权限。SSH 和 HTTPS 二选一即可。

### 初始化、部署配置，再按需恢复 GPG

下面使用 HTTPS；已选择 SSH 时，换成 `git@github.com:szwathub/dotfiles.git`，且首次 init 前保持上面加载的 agent 可用：

```bash
stool dotfile init https://github.com/szwathub/dotfiles.git --recurse-submodules
stool dotfile diff
stool dotfile apply
stool dotfile verify
```

init 核对 Git 姓名、邮箱、签名指纹和 Homebrew prefix。`--data git_signing_key=...` 仅覆盖个人 Git 签名默认值，不是下载或恢复前置。个人与 Work 身份仍按 clone URL 规则选择；签名私钥需手动恢复。

这份 dotfiles 只部署配置和 macOS 的 `~/.Brewfile`。配置部署完成后显式运行 `stool apple brew install --dry-run` 预览，再运行 `stool apple brew install` 安装清单内软件（含 GPG Suite）；该命令会先确保 Homebrew 可用。Homebrew 初始化也可在克隆前独立运行 `stool apple brew init`。安装完成后按输出执行 shellenv 指引，再检查 `gpg`。macOS 偏好单独运行 `stool apple init`；apply/update 不再安装软件或设置 Dock。`apply`、`update`、`diff` 和 `verify` 均不恢复密钥、不询问离线备份密码；`verify` 只检查配置。已有 `~/.gitconfig` 可能冲突，先看 diff，再选择备份覆盖；非交互时检查后才使用 `apply --force`。

若第一步仅恢复了 SSH，先显式安装软件包，再按 Homebrew 输出配置当前终端 PATH，确认 `gpg` 可用后手动导入：

```bash
stool apple brew install --dry-run
stool apple brew install
# 按上述输出执行当前 Homebrew prefix 对应的 shellenv 命令
command -v gpg
gpg --version
stool key restore --gpg --input /Volumes/Backup/my-keys.tar.age
gpg --list-secret-keys --fingerprint \
  2BF609784E75AAC78CB67A7E465799B005224003 \
  98A56F40C2A502230AC6635435DF2F117D3233C5
```

GPG Suite/pinentry 可能另行要求私钥原口令。已在最开始恢复全部密钥时无需再次恢复。实际签名检查、备份参数及失败重试见 [离线密钥说明](docs/key-backup.md)。后续配置更新运行 `stool dotfile update`；密钥变化时另行创建新的离线备份。

## 从源码构建

```bash
cargo build --locked --release
export PATH="$PWD/target/release:$PATH"
stool --version
stool dotfile init https://github.com/YOUR_NAME/dotfiles
stool dotfile diff
stool dotfile apply
stool dotfile update
```

Rust 最低版本为 1.74。默认使用系统 Git；`init --builtin` 使用内置 gix。HTTPS、`ssh://`、`git@host:path`、绝对 `file://` URL 均可使用。私有仓库复用凭据助手或 SSH 配置，URL 不接受嵌入密码。`--branch` 选择分支，`--recurse-submodules` 初始化并持续更新子模块。内置后端通过 gix 克隆、校验和快进更新普通仓库；SSH 需要 SSH 客户端，子模块及 Git LFS 使用系统 Git 工具。

`init` 在克隆前检查系统 Git 是否可用。缺失时停止并显示当前系统的安装命令：macOS 使用 `xcode-select --install` 安装包含 Git 的 Command Line Tools，Windows 提示 Git for Windows / winget，Linux 提示发行版包管理器。安装完成后运行 `git --version`，再重试原命令；检查失败不会创建 source、保存初始化配置或部署文件。macOS 的 `/usr/bin/git` 占位程序不会被当作已安装的 Git，也不会自动弹出安装窗口。普通仓库使用 `--builtin` 时无需这个检查；`--builtin --recurse-submodules` 仍需要系统 Git。Homebrew / 软件包安装由独立 `stool apple brew` 管理，不由 apply 执行，也不能替代首次克隆前的 Git 安装。

## 日常命令

详细说明见[产品概览](docs/dotfile-overview.md)、[实现说明](docs/dotfile-internals.md)和[设计文档](docs/dotfile-design.md)。

```bash
stool dotfile init <git-url> --apply
stool dotfile status
stool dotfile managed
stool dotfile unmanaged
stool dotfile apply --dry-run
stool dotfile verify
stool dotfile doctor
stool dotfile add ~/.gitconfig ~/.config/fish
stool dotfile add --template ~/.gitconfig
stool dotfile edit ~/.gitconfig --apply
stool dotfile cd                 # 在 source 中启动 shell
stool dotfile cd --print
stool dotfile archive --format tgz --output dotfiles.tar.gz
stool dotfile execute-template '{{ stool.os }} / {{ stool.arch }}'
stool dotfile data
stool completions zsh
stool man --output ./man-page
```

`add` 路径可为目标目录内的绝对路径或目标相对路径，会递归导入并推断属性；导入时记录本机基线，支持随后渲染模板。`external_` 字面子树和下载资源直接编辑 source / 资源定义，不用 add 重新编码。`edit` 使用本机 `editor` 配置、`VISUAL`、`EDITOR`，最后使用 vi / notepad。`verify` 在受管理文件的内容、类型或权限不一致时返回非零；脚本待执行项通过 `status` 查看。

用隔离目录试运行：

```bash
stool dotfile --source /tmp/dotfiles-source \
  --destination /tmp/dotfiles-home init <git-url>
stool dotfile diff
stool dotfile apply
```

`init` 保存目录，后续可直接使用。命令行覆盖仅对本次有效。控制目录为 `~/.stool/dotfile/`：`source/`、`config.toml`、`state/`、`backups/`、`cache/`。文件基线按 source/destination 配对隔离，脚本成功状态保存在 redb。并发写操作互斥。

## 仓库路径协议

```text
dotfiles/
├── .stoolignore
├── .stooldata.toml
├── .stool.toml.tmpl
├── .stooltemplates/shared
├── dot_gitconfig.tmpl
├── dot_config/fish/config.fish
├── private_dot_ssh/private_config
├── dot_local/bin/executable_tool
└── run_once_after_setup.sh
```

普通 source 文件写入独立目标文件。每个目录分量单独解析属性；文件属性依次为类型前缀（`create_` / `modify_` / `remove_` / `symlink_`）、`encrypted_`、`private_`、`readonly_`、`empty_`、`executable_`、`dot_`、`literal_`；目录支持 `external_`、`exact_`、`private_`、`readonly_`、`dot_`、`literal_`。不适用或顺序错误会停止部署。

| 属性 | 行为 |
|---|---|
| `dot_gitconfig` | `~/.gitconfig` |
| `private_dot_ssh/private_config` | Unix 目录 0700、文件 0600 |
| `readonly_` / `executable_` | 去掉写权限 / 增加执行权限 |
| `empty_` | 保留空文件；未带此属性的空普通 source 文件表示目标应不存在 |
| `literal_` / `.literal` | 将保留前缀 / 后缀当作字面文件名 |
| `.tmpl` | 渲染 MiniJinja 模板 |
| `symlink_` | source 文件内容为符号链接目标 |
| `create_` | 目标存在时完整保留，只创建缺失项 |
| `modify_` | 脚本从 stdin 读取原内容，将 stdout 作为期望内容；应设计为幂等转换 |
| `remove_` | 删除文件、链接或空目录；非空目录会报错 |
| `exact_` | 递归移除目录中未声明的项，先备份 |
| `external_` | 子树文件名按字面处理，不解析其属性 |

Unix 默认文件 0644、目录/可执行文件 0755。Windows 使用 DACL 实现私有和只读属性，文件执行位归一化；创建符号链接需要开发者模式或相应权限。无 shebang 的 Windows 脚本由 PowerShell 执行，Unix 使用 sh。

普通隐藏 source 条目和 Git 元数据跳过。当前 develop 的受支持控制文件仅使用 `.stool*`，不自动读取旧命名：

- `ignore[.tmpl]`：按目标相对路径匹配 `*`、`?`、`**`、字符组、`!` 例外；所有例外优先于忽略项。支持目录内局部规则，`folder/` 仅匹配目录条目。
- `remove[.tmpl]`：每行一个目标相对 glob，显式递归删除匹配项并备份；拒绝越界及控制目录。
- `root`：选择仓库内的 source 子目录；`version`：最低 stool 版本或 semver 范围。
- `data.{toml,json,yaml,yml}[.tmpl]` / `data/`：合并模板数据。
- `templates/`：模板 include 目录；`scripts/`：额外脚本目录。
- `.stool.toml.tmpl`：初始化本机配置。
- `external.{toml,json,yaml,yml}[.tmpl]` / `externals/`：外部资源定义。

现有模块目录和 `*.symlink` 仓库需先迁移。`szwathub/dotfiles` 的当前配置见[设计文档](docs/dotfile-design.md)；本项目不会自动修改远端仓库。

## 模板与本机配置

模板使用 **MiniJinja 语法**，机器数据使用 `{{ stool.os }}` 等字段；条件使用 `{% if stool.os == 'macos' %}...{% endif %}`，include 使用 `{% include 'shared' %}`。默认缺失变量报错。

`stool` 内置数据包括 `os`、`arch`、`hostname`、`username`、`home_dir`、`source_dir`、`dest_dir`、`cache_dir`、`config_file`、`version`、`path_separator`。数据覆盖顺序为仓库数据、本机 `[data]`、`--data key=value`（值可为 JSON，点分路径支持嵌套）；`stool` 命名空间保留。

支持 Jinja 内置过滤器、`env()`、`join_path()`、`from_json` / `from_yaml` / `to_yaml` / `to_toml` 过滤器。模板指令行会从结果中移除，例如：

```jinja
# stool:template: line-ending=crlf encoding=utf-16le missing-key=error
{{ stool.version }}
```

指令还支持 `left-delimiter` / `right-delimiter`、LF/CRLF/preserve、其他 encoding_rs 编码和 `missing-key=zero`。模板文件本身为 UTF-8。

初始化配置示例 `.stool.toml.tmpl`：

```toml
[data]
name = {{ prompt_string('Git name', 'Your Name') | tojson }}

[git]
builtin = false
lfs = false

[encryption]
backend = 'age'
recipients = ['age1...']
identities = ['~/.config/stool/identity.txt']
```

初始化还支持 `prompt_bool`、`prompt_int`、`prompt_choice`；非终端必须提供默认值。日常模板不开放提示函数。编辑器可在本机配置顶层设置 `editor = 'code --wait'`。初始化模板只能设置 `data`、`git`、`encryption`、`editor`，保留 URL、分支和目录。

## 加密、秘密与脚本

```bash
stool dotfile keygen --output ~/.config/stool/identity.txt
stool dotfile add --encrypt ~/.ssh/config
stool dotfile edit ~/.ssh/config
stool dotfile apply
```

普通 dotfile 内容由 `add --encrypt` 导入为密文，`edit` 编辑后重新加密，`apply` 解密并部署。age 使用本机 `[encryption]` 中的 `recipients` 和 `identities` 配置，支持 X25519、多接收方及 armored 密文；既有口令型 source 文件的 `STOOL_AGE_PASSPHRASE` 兼容方式保留。离线密钥包的 age 密码模式和外部密码文件另见下节，不依赖已移除的独立子命令。

普通 GPG 加密文件使用本机 `gpg`，在本机 `[encryption]` 中配置 `backend='gpg'` 和 `gpg_recipient`。`encrypted_` 文件支持 `.age` / `.asc`，可叠加 `.tmpl`。加密编辑仅在私有临时目录中解密，编辑成功后重新加密。

模板秘密函数有 `onepassword`、`bitwarden`、`pass`、`gopass`、`vault`、`keyring`、`keepassxc`、`lastpass`、`doppler`、`aws`、`azure`，另有通用 `secret(provider, ...args)`。示例 `{{ onepassword('op://vault/item/password') }}`、`{{ bitwarden('item-id').login.password }}`。各服务复用已认证的官方 CLI；同一进程内相同请求只调用一次，不将秘密缓存落盘。Bitwarden 可使用 `BW_SESSION`，或以 `BW_PASSWORD` 解锁一次；KeePassXC 从 `STOOL_KEEPASS_PASSWORD` 读取口令。服务失败时隐藏其输出。

脚本支持 `run_`、`run_once_`、`run_onchange_`，可叠加 `before_` / `after_` 和 `.tmpl`。before 阶段先执行，普通脚本与文件按解码目标路径穿插执行，after 阶段最后执行。只在成功后记录 once/onchange 状态；失败可重试，已完成文件和成功脚本保留。onchange 记录最近成功内容，A→B→A 会重新执行。提供 `STOOL_SOURCE_DIR` / `STOOL_DEST_DIR` / `STOOL_HOME` / `STOOL_OS` / `STOOL_ARCH`，以及当前运行程序绝对路径 `STOOL_EXECUTABLE`。

GPG/SSH 私钥由独立 `stool key` 命令离线备份与恢复，不接入部署钩子；普通 dotfile 的加密由导入、编辑和部署流程处理。

预览时 stool 不执行 run hooks、不写目标、部署状态或持久缓存；渲染模板仍会调用秘密提供器，`modify_` 转换脚本会在私有临时工作目录中执行。这些外部程序没有被沙箱限制，仍可能有自己的副作用。部署和预览都应使用可信仓库。diff 和 archive 包含渲染后的明文，请按目标文件的敏感程度保存。

## 密钥离线备份与恢复

```bash
# 先进入仓库外目录，输出默认位于当前目录，文件名自动带时间戳
mkdir -p "$HOME/Backups/stool"
cd "$HOME/Backups/stool"
stool key backup

# 默认恢复包内全部 GPG 和 SSH 密钥到 HOME
stool key restore --input /Volumes/Backup/my-keys.tar.age
```

`--gpg`、`--ssh`、`--all` 互斥，无开关默认全部。backup 的 `--input` 是可选源用户目录，`--output` 是可选新文件；restore 的 `--input` 是必需加密文件，`--output` 是可选目标用户目录，默认 HOME。SSH 单独备份/恢复可在没有 Git/GPG 的机器运行。

一个密码加密的 tar.age 文件包含完整 GPG 私钥、公钥、ownertrust、已有撤销证书、SSH 私钥及配对公钥和清单；GPG 缺失/硬件材料、SSH agent-only/硬件及 `.ssh` 之外文件有明确范围限制。备份隐藏输入并确认密码，恢复隐藏输入一次，原私钥口令不改变。密码与完整包均留在 Git 仓库外，已有同名输出或不同内容的目标文件不覆盖。

备份范围、路径和权限防护、密码文件/管道、格式、验证步骤和失败重试见 [离线密钥说明](docs/key-backup.md)。dotfiles 只管理配置和签名指纹，不包含公钥、私钥密文或自动恢复声明。`dotfile verify` 不检查密钥是否已手动恢复。

## 外部资源与归档

`.stoolexternal.toml` 示例：

```toml
['.local/bin/tool']
type = 'file'
url = 'https://example.org/tool'
executable = true
refreshPeriod = '24h'
# checksum = { sha256 = '...' }

['.config/vendor']
type = 'archive'
url = 'https://example.org/package.tar.gz'
stripComponents = 1
exact = true
exclude = ['README.md']
```

支持 HTTP(S)、`file://`、`file` / `archive` / `archive-file` / `git-repo`；归档自动识别 tar、gzip tar、zip。`archive-file` 用 `path` 选择一个文件；Git 资源可指定 `branch` 和 `recurseSubmodules`。还支持 `include`、`exclude`、`private`、SHA-256 校验；`refreshPeriod` 支持 s/m/h/d/w，`0s` 每次刷新。资源缓存只在部署时持久化，预览需要时临时下载。拒绝归档越界、重复路径、链接父目录及特殊设备文件。`git.lfs=true` 在 init/update 中运行 git-lfs，未检出的 LFS 指针会阻止部署。

`archive --format tar|tgz|zip` 使用相同渲染计划导出内容、权限和链接，不执行 run hooks、不部署目标，输出文件设为私有权限。

## 冲突、备份和验证

首次遇到不同的已有文件，或上次部署后本机发生修改时，先报告全部冲突。终端中可交互选择备份覆盖；自动化中默认返回错误。检查后使用 `apply --force`；`--prompt never` 禁用交互，`--prompt always` 要求终端。`create_` 保留已有项；首次显式 `modify_` / remove / exact 操作可以更新对应目标，之后继续检查本地修改基线。

已有父目录是符号链接、目标与 source/control 重叠、重复或大小写碰撞目标等情况会停止部署。覆盖和删除前备份至 `backups/apply-*/files/`，元数据记录于 `metadata.json`。目录权限修改只记录元数据。文件同目录原子替换；多文件部署发生 IO 或脚本错误时保留已完成项和备份，修复后重试。

重复部署不会重写相同内容。移除 source 条目会解除管理并保留目标；`remove_`、exact 和 remove 控制文件用于显式删除。update 拒绝未提交、未跟踪修改和分支分叉；仅快进后重新部署，目标冲突时保留已拉取 source。

```bash
cargo test --locked --all-targets
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo +1.74.0 test --locked --all-targets
```

测试使用临时 Git 远端、HTTP 服务、模拟秘密 CLI、隔离 HOME 和目标目录。age 使用真实加解密；第三方认证服务不使用真实凭据。macOS 本机验证，Linux/Windows 和最低 Rust 版本由 CI 矩阵持续验证。

MIT · Zhiwei Sun (@szwathub) · szwathub@gmail.com
