# xyrev

开发环境命令行工具。`xyrev dotfile` 通过 Git URL 初始化、预览、部署、维护和更新 dotfiles，采用 chezmoi 风格路径属性、MiniJinja 模板及统一变更计划。

## 安装发布版本

macOS / Linux 无需先安装 Git、Rust 或 Homebrew。安装器按系统架构下载发布包，验证 SHA-256 后安装到 `~/.local/bin`：

```bash
curl -fsSL https://raw.githubusercontent.com/Xyrev/xyrev/v0.4.1/install.sh | VERSION=v0.4.1 bash
export PATH="$HOME/.local/bin:$PATH"
xyrev --version
```

可设置 `PREFIX` 选择安装目录（最终位置为 `$PREFIX/bin/xyrev`）。`install.sh` 和兼容入口 `newinstall.sh` 行为一致；不指定 `VERSION` 时选择最新稳定 Release。Windows 可从 [Release 下载页](https://github.com/Xyrev/xyrev/releases/tag/v0.4.1) 下载 `xyrev-v0.4.1-x86_64-pc-windows-msvc.zip`，解压后使用其中的 `xyrev.exe`。

在新机器上初始化 dotfiles：

```bash
xyrev dotfile init https://github.com/szwathub/dotfiles --recurse-submodules
xyrev dotfile diff
xyrev dotfile apply
xyrev dotfile verify
```

Git 不可用时，`init` 会在克隆前显示安装指引；macOS 按提示完成 Command Line Tools 安装后重试原命令。`apply` 时仓库先检查或初始化 Homebrew，再安装 Brewfile 中的软件并部署配置。先查看 `diff` 确认变更；完整部署到 HOME 会执行仓库中的系统设置脚本。

## 从源码构建

```bash
cargo build --locked --release
xyrev dotfile init https://github.com/YOUR_NAME/dotfiles
xyrev dotfile diff
xyrev dotfile apply
xyrev dotfile update
```

Rust 最低版本为 1.74。默认使用系统 Git；`init --builtin` 使用内置 gix。HTTPS、`ssh://`、`git@host:path`、绝对 `file://` URL 均可使用。私有仓库复用凭据助手或 SSH 配置，URL 不接受嵌入密码。`--branch` 选择分支，`--recurse-submodules` 初始化并持续更新子模块。内置后端通过 gix 克隆、校验和快进更新普通仓库；SSH 需要 SSH 客户端，子模块及 Git LFS 使用系统 Git 工具。

`init` 在克隆前检查系统 Git 是否可用。缺失时停止并显示当前系统的安装命令：macOS 使用 `xcode-select --install` 安装包含 Git 的 Command Line Tools，Windows 提示 Git for Windows / winget，Linux 提示发行版包管理器。安装完成后运行 `git --version`，再重试原命令；检查失败不会创建 source、保存初始化配置或部署文件。macOS 的 `/usr/bin/git` 占位程序不会被当作已安装的 Git，也不会自动弹出安装窗口。普通仓库使用 `--builtin` 时无需这个检查；`--builtin --recurse-submodules` 仍需要系统 Git。仓库中的 Homebrew / 软件包脚本在后续 apply 阶段执行，不能替代首次克隆前的 Git 安装。

## 日常命令

```bash
xyrev dotfile init <git-url> --apply
xyrev dotfile status
xyrev dotfile managed
xyrev dotfile unmanaged
xyrev dotfile apply --dry-run
xyrev dotfile verify
xyrev dotfile doctor
xyrev dotfile add ~/.gitconfig ~/.config/fish
xyrev dotfile add --template ~/.gitconfig
xyrev dotfile edit ~/.gitconfig --apply
xyrev dotfile cd                 # 在 source 中启动 shell
xyrev dotfile cd --print
xyrev dotfile archive --format tgz --output dotfiles.tar.gz
xyrev dotfile execute-template '{{ xyrev.os }} / {{ xyrev.arch }}'
xyrev dotfile data
xyrev completions zsh
xyrev man --output ./man-page
```

`add` 路径可为目标目录内的绝对路径或目标相对路径，会递归导入并推断属性；导入时记录本机基线，支持随后渲染模板。`external_` 字面子树和下载资源直接编辑 source / 资源定义，不用 add 重新编码。`edit` 使用本机 `editor` 配置、`VISUAL`、`EDITOR`，最后使用 vi / notepad。`verify` 在受管理文件的内容、类型或权限不一致时返回非零；脚本待执行项通过 `status` 查看。

用隔离目录试运行：

```bash
xyrev dotfile --source /tmp/dotfiles-source \
  --destination /tmp/dotfiles-home init <git-url>
xyrev dotfile diff
xyrev dotfile apply
```

`init` 保存目录，后续可直接使用。命令行覆盖仅对本次有效。控制目录为 `~/.xyrev/dotfile/`：`source/`、`config.toml`、`state/`、`backups/`、`cache/`。文件基线按 source/destination 配对隔离，脚本成功状态保存在 redb。并发写操作互斥。

## 仓库路径协议

```text
dotfiles/
├── .xyrevignore
├── .xyrevdata.toml
├── .xyrev.toml.tmpl
├── .xyrevtemplates/shared
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

普通隐藏 source 条目和 Git 元数据跳过。受支持的控制文件统一使用 `.xyrev*`，同时接受对应的 `.chezmoi*` 迁移别名：

- `ignore[.tmpl]`：按目标相对路径匹配 `*`、`?`、`**`、字符组、`!` 例外；所有例外优先于忽略项。支持目录内局部规则，`folder/` 仅匹配目录条目。
- `remove[.tmpl]`：每行一个目标相对 glob，显式递归删除匹配项并备份；拒绝越界及控制目录。
- `root`：选择仓库内的 source 子目录；`version`：最低 xyrev 版本或 semver 范围。
- `data.{toml,json,yaml,yml}[.tmpl]` / `data/`：合并模板数据。
- `templates/`：模板 include 目录；`scripts/`：额外脚本目录。
- `.xyrev.toml.tmpl`：初始化本机配置。
- `external.{toml,json,yaml,yml}[.tmpl]` / `externals/`：外部资源定义。

现有模块目录和 `*.symlink` 仓库需先迁移。原 `szwathub/dotfiles` 的迁移示例见[设计文档](chezmoi-rust-implementation.md)；本项目不会自动修改远端仓库。

## 模板与本机配置

模板使用 **Jinja 语法**，Go Template 需要迁移。例如 `{{ .chezmoi.os }}` 改为 `{{ xyrev.os }}`；条件使用 `{% if xyrev.os == 'macos' %}...{% endif %}`，include 使用 `{% include 'shared' %}`。默认缺失变量报错。

`xyrev` 内置数据包括 `os`、`arch`、`hostname`、`username`、`home_dir`、`source_dir`、`dest_dir`、`cache_dir`、`config_file`、`version`、`path_separator`。数据覆盖顺序为仓库数据、本机 `[data]`、`--data key=value`（值可为 JSON，点分路径支持嵌套）；`xyrev` 命名空间保留。

支持 Jinja 内置过滤器、`env()`、`join_path()`、`from_json` / `from_yaml` / `to_yaml` / `to_toml` 过滤器。模板指令行会从结果中移除，例如：

```jinja
# xyrev:template: line-ending=crlf encoding=utf-16le missing-key=error
{{ xyrev.version }}
```

指令还支持 `left-delimiter` / `right-delimiter`、LF/CRLF/preserve、其他 encoding_rs 编码和 `missing-key=zero`。模板文件本身为 UTF-8。

初始化配置示例 `.xyrev.toml.tmpl`：

```toml
[data]
name = {{ prompt_string('Git name', 'Your Name') | tojson }}

[git]
builtin = false
lfs = false

[encryption]
backend = 'age'
recipients = ['age1...']
identities = ['~/.config/xyrev/identity.txt']
```

初始化还支持 `prompt_bool`、`prompt_int`、`prompt_choice`；非终端必须提供默认值。日常模板不开放提示函数。编辑器可在本机配置顶层设置 `editor = 'code --wait'`。初始化模板只能设置 `data`、`git`、`encryption`、`editor`，保留 URL、分支和目录。

## 加密、秘密与脚本

```bash
xyrev dotfile keygen --output ~/.config/xyrev/identity.txt
xyrev dotfile add --encrypt ~/.ssh/config
xyrev dotfile encrypt input --recipient age1... --output output.age
xyrev dotfile decrypt output.age --identity ~/.config/xyrev/identity.txt
xyrev dotfile encrypt input --passphrase --output password-backup.age
xyrev dotfile decrypt password-backup.age --passphrase
```

内置 age 支持 X25519 身份、多个接收方、密码加密和 armored 密文。独立 `encrypt` / `decrypt --passphrase` 在本机终端隐藏输入，加密时二次确认；密码模式不需要 age 身份文件。脚本可用 `--passphrase-file /dev/fd/3` 从管道描述符读取密码，不要把密码放入命令参数、环境变量或仓库。密码不能为空；密码错误不会写出解密结果或覆盖输出文件。密码选项不能与 `--identity` / `--recipient` / GPG backend 混用。既有口令型 source 文件的 `XYREV_AGE_PASSPHRASE` 兼容方式保留。

GPG 使用本机 `gpg`，配置 `backend='gpg'`、`gpg_recipient` 或使用 `--backend gpg --recipient <id>`。`encrypted_` 文件支持 `.age` / `.asc`，可叠加 `.tmpl`。加密编辑仅在私有临时目录中解密，编辑成功后重新加密。

模板秘密函数有 `onepassword`、`bitwarden`、`pass`、`gopass`、`vault`、`keyring`、`keepassxc`、`lastpass`、`doppler`、`aws`、`azure`，另有通用 `secret(provider, ...args)`。示例 `{{ onepassword('op://vault/item/password') }}`、`{{ bitwarden('item-id').login.password }}`。各服务复用已认证的官方 CLI；同一进程内相同请求只调用一次，不将秘密缓存落盘。Bitwarden 可使用 `BW_SESSION`，或以 `BW_PASSWORD` 解锁一次；KeePassXC 从 `XYREV_KEEPASS_PASSWORD` 读取口令。服务失败时隐藏其输出。

脚本支持 `run_`、`run_once_`、`run_onchange_`，可叠加 `before_` / `after_` 和 `.tmpl`。before 阶段先执行，普通脚本与文件按解码目标路径穿插执行，after 阶段最后执行。只在成功后记录 once/onchange 状态；失败可重试，已完成文件和成功脚本保留。onchange 记录最近成功内容，A→B→A 会重新执行。提供 `XYREV_SOURCE_DIR` / `XYREV_DEST_DIR` / `XYREV_HOME` / `XYREV_OS` / `XYREV_ARCH`，以及当前运行程序绝对路径 `XYREV_EXECUTABLE`。

钩子可以调用 `"$XYREV_EXECUTABLE" dotfile decrypt backup.age --passphrase-file /dev/fd/3` 或 `--identity /path/to/identity`，将解密结果通过管道交给导入工具；独立 `encrypt` / `decrypt` 不争用部署锁，`apply` / `add` 等写操作仍受锁保护。需流式恢复的密钥备份可放在 `.xyrevtemplates/`，该目录不会作为目标文件部署或纳入 archive。将密文摘要包含在 onchange 模板中，备份变更才会触发重新导入。密码保存在密码管理器；采用身份文件模式时，age 私钥必须在仓库外单独保管并预先恢复。不要将 GPG 私钥只加密给自身，否则新机器缺少用于解密的同一把私钥。钩子没有交互 stdin，但可显式通过控制终端隐藏读取密码；受口令保护的 GPG 密钥可能通过 pinentry 提示解锁。

预览不执行 run hooks、不更新目标、部署状态或缓存；渲染模板仍会调用秘密提供器，`modify_` 转换脚本会在私有临时工作目录中执行。部署和预览都应使用可信仓库。diff 和 archive 包含渲染后的明文，请按目标文件的敏感程度保存。

## 外部资源与归档

`.xyrevexternal.toml` 示例：

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
