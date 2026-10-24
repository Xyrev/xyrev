# stool dotfile：实现规范、兼容边界与新机器流程

> 校订日期：2026-10-24。Cargo 当前版本为 0.6.0，本文按待发布源码描述，新增离线命令不回填旧 tag。历史 xyrev v0.5.0 的 tag、资源、固定安装地址与二进制行为不变。
>
> 本文记录 stool 的产品协议、新机器流程、实现证据与长期规划。[产品概览](dotfile-overview.md) 说明能力和边界，[实现说明](dotfile-internals.md) 说明当前 Rust 模块和状态处理。当前使用入口另见 [README](../README.md)，命令选项以 `stool dotfile --help` 及子命令帮助为准。
>
> 原设计中的 workspace、Vfs trait、类型化路径和异步实现草图已归入第 7 节。它们不是现有源码结构，也不是使用本版本的前置条件。

## 1. 产品目标与已实现功能

stool 在现有单一 Rust crate 中提供 `stool dotfile`：通过完整 Git URL 获取 source，按 stool 路径属性和 MiniJinja 模板计算目标，与 destination 及上次成功部署的基线比较，再执行变更。默认 destination 为当前用户 HOME。

### 1.1 已实现能力与证据入口

下表表示已经接入的产品能力，不表示所有第三方账户、配置组合或未来扩展都已完成真实服务验收。

| 能力 | 当前行为 | 实现与测试 |
|---|---|---|
| Git 初始化与更新 | 完整 URL、分支、子模块选项保存；重复 init 校验来源；update 拒绝脏工作树和分叉，仅允许快进 | [git.rs](../src/dotfile/git.rs)、[prerequisites.rs](../src/dotfile/prerequisites.rs)、[CLI 测试](../tests/dotfile_cli.rs) |
| 内置 Git | `--builtin` 使用 gix 克隆、选分支、检查状态和快进更新；子模块、LFS 按需回退系统工具 | [native_git.rs](../src/dotfile/native_git.rs)、[高级测试](../tests/dotfile_advanced.rs) |
| 路径与文件属性 | 文件、目录、权限、模板、加密、显式链接、创建/转换/删除、exact、literal 与 external 子树 | [source.rs](../src/dotfile/source.rs)、[属性与部署测试](../tests/dotfile_advanced.rs) |
| 统一变更计划 | source/target/actual 与上次基线比较；预检冲突；覆盖前备份；单文件原子替换、部分成功记录和重试 | [plan.rs](../src/dotfile/plan.rs)、[CLI 测试](../tests/dotfile_cli.rs) |
| 模板和本机数据 | MiniJinja、include、严格缺失变量、嵌套数据覆盖、模板指令、初始化提示 | [template.rs](../src/dotfile/template.rs)、[高级测试](../tests/dotfile_advanced.rs) |
| 维护与诊断 | add/edit/unmanaged、递归导入、编辑器、root/version、verify/doctor/data/execute-template/cd | [maintenance.rs](../src/dotfile/maintenance.rs)、[tools.rs](../src/dotfile/tools.rs) |
| 脚本 | run/once/onchange、before/during/after 顺序；成功才持久化，失败可重试，onchange 的 A→B→A 再执行 | [scripts.rs](../src/dotfile/scripts.rs)、[钩子测试](../tests/dotfile_crypto_hooks.rs) |
| 普通文件加密 | 内置 age；本机 GPG 子进程；通过 add --encrypt 导入、edit 编辑、apply 解密部署；保留内部 age 密码处理 | [crypto.rs](../src/dotfile/crypto.rs)、[加密文件测试](../tests/dotfile_advanced.rs) |
| 离线 GPG/SSH 密钥 | 独立 key backup/restore、默认全部、单文件密码加密、仓库外保存和手动恢复 | [src/key/](../src/key/mod.rs)、[命令测试](../tests/key_backup.rs)、[真实密钥测试](../tests/test_keys_native.py) |
| 秘密提供器 | 1Password、Bitwarden、pass/gopass、Vault、Keyring、KeePassXC、LastPass、Doppler、AWS、Azure 及通用命令适配；进程内缓存 | [secrets.rs](../src/dotfile/secrets.rs)、[高级测试](../tests/dotfile_advanced.rs) |
| 外部资源与归档 | 文件、归档、归档中单文件、Git 资源；TTL、SHA-256、过滤、路径保护；tar/tgz/zip 导出 | [external.rs](../src/dotfile/external.rs)、[archive.rs](../src/dotfile/archive.rs) |
| 终端和命令文档 | ratatui 冲突确认；由同一 clap 定义生成 shell 补全和递归 man pages | [ui.rs](../src/dotfile/ui.rs)、[CLI 定义](../src/cli/dotfile/mod.rs)、[生成脚本](../scripts/generate-cli-assets.sh) |

源码组织与状态存储见第 5 节；运行环境和验收范围见第 6 节。产品命名统一为 `stool`，配置入口为 `dotfile`，密钥入口为 `key`。

### 1.2 常用命令

```text
stool dotfile init <git-url> [--branch <branch>] [--recurse-submodules] [--apply] [--builtin]
stool dotfile diff
stool dotfile status
stool dotfile managed
stool dotfile unmanaged
stool dotfile apply [--dry-run] [--force] [--prompt auto|always|never]
stool dotfile update
stool dotfile verify
stool dotfile doctor
stool dotfile add <path> [--template] [--encrypt]
stool dotfile edit <path> [--apply]
stool dotfile archive --format tar|tgz|zip --output <file>
stool dotfile execute-template <template>
stool dotfile data
stool dotfile cd [--print]
stool dotfile keygen --output <identity-file>
stool key backup [--gpg|--ssh|--all] [--input <user-dir>] [--output <new-file>]
stool key restore [--gpg|--ssh|--all] --input <file> [--output <user-dir>]
```

`--source`、`--destination` 和可重复的 `--data key=value` 是 dotfile 全局选项。覆盖值支持 JSON 和点分嵌套路径；普通调用中的覆盖仅对本次有效。init 的配置模板会保存用户确认的本机数据。

默认控制目录是 `~/.stool/dotfile/`：

| 路径 | 用途 |
|---|---|
| `source/` | 默认 Git 工作树 |
| `config.toml` | 仓库 URL、分支、目录、选项及本机数据 |
| `state/<namespace>.json` | 按 source/destination 配对隔离的文件基线 |
| `state/scripts.redb` | 按同一配对隔离的脚本成功状态 |
| `backups/` | 覆盖和删除前的备份、元数据 |
| `cache/` | 部署时的外部资源缓存 |

### 1.3 本版兼容性变更

v0.6.0 将项目、Cargo package/bin、命令和发布资源统一为 stool，仓库为 `szwathub/stool`。旧 tag 和资源不重命名。可下载安装包或从源码构建；历史 xyrev 安装另见 [README](../README.md#安装与版本边界)。

source 控制项、模板数据/指令、环境变量和默认目录使用 `.stool*`、`stool.*` / `stool:template:`、`STOOL_*` 与 `~/.stool`。不自动加载旧协议，旧环境应按重新初始化流程处理，并核对本机配置、状态与 source 引用；不因启动新程序而隐式改写远端仓库。

独立加解密子命令已移除，普通 dotfile 用 `add --encrypt`、`edit`、`apply` 管理，内部 age/GPG 与 `dotfile keygen` 保留。

GPG/SSH 密钥统一使用 `stool key backup/restore`，默认全部，离线包和密码在 Git 仓库外。删除旧 `dotfile gpg`、自动恢复声明、四文件格式与身份文件加密备份兼容；先创建并隔离验证新包，再清理旧 source。dotfiles 不保存任何密钥备份或公钥。

### 1.4 旧控制目录迁移

不提供旧控制状态迁移命令。先保存旧 source 的未提交改动、本机配置及仓库外凭据，确认远端采用新协议后，使用普通 `init` 在 `~/.stool/dotfile` 建立独立 source 和状态。核对数据、资源路径、`status` 和 `diff` 后再部署；旧 source 的未提交修改需逐项合并。

不复制旧配置、文件基线或 once/onchange 成功记录，不自动改名业务目录。旧控制目录保持原样供核对，新的脚本可能重新执行。密钥备份先按 [离线说明](key-backup.md) 完成，之后配置初始化与密钥手动恢复分别进行；本次不改写历史或自动删除旧目录/已有密钥。

## 2. stool 路径协议与明确边界

### 2.1 路径和控制文件

普通 source 文件部署为独立目标文件，不默认创建链接。每个路径分量独立解析属性，组合顺序和适用类型受校验。

| source 示例或属性 | 当前语义 |
|---|---|
| `dot_gitconfig.tmpl` | 渲染到 `~/.gitconfig` |
| `private_dot_ssh/private_config` | 部署 SSH 配置；Unix 目录 0700、文件 0600；不代表仓库包含 SSH 私钥 |
| `dot_local/bin/executable_tool` | 写入可执行文件 |
| `symlink_dot_example` | 普通 source 文件的内容指定链接目标 |
| `encrypted_`、`.tmpl` | 解密与模板渲染；格式、引擎须符合当前实现 |
| `create_` | 已有目标保留，只创建缺失项 |
| `modify_` | 转换脚本从 stdin 读取现有内容，以 stdout 产生期望内容 |
| `remove_` | 删除文件、链接或空目录，非空目录报错 |
| `exact_`、remove 控制文件 | 明确声明递归删除未管理项；执行前备份 |
| `private_`、`readonly_`、`empty_`、`dot_` | 权限、空文件与点文件名属性 |
| `literal_`、`.literal`、`external_` | 按相应字面命名或外部子树规则处理 |

未带 `empty_` 的空普通 source 文件表示目标应不存在。移除 source 条目表示解除管理，保留目标。普通 source 原生符号链接被拒绝，应使用 `symlink_` 文件；literal 外部子树按其字面条目处理。旧 `*.symlink` 后缀没有特殊映射含义。

当前 develop 只读取以下 `.stool*` 控制项；旧命名不自动加载：

- `ignore[.tmpl]`、`remove[.tmpl]`：目标路径规则；ignore 支持局部规则、`**`、优先的 `!` 例外。
- `data.{toml,json,yaml,yml}[.tmpl]`、`data/`：数据文件和目录。
- `templates/`、`scripts/`、`externals/`、`external.{toml,json,yaml,yml}[.tmpl]`：模板、脚本与外部资源定义。
- `root`、`version`：选择 source 子目录、校验最低版本或 semver 范围。
- `.stool.toml.tmpl`：初始化本机配置，仅允许设置 data、git、encryption、editor。

旧 GPG 自动恢复声明已删除，未知的保留控制项报错。Git 元数据不部署，普通非字面隐藏 source 项跳过。具体字段见 [README](../README.md#仓库路径协议)。

### 2.2 模板、来源与部署边界

模板引擎为 MiniJinja，机器数据位于 `stool.*`，例如 `{{ stool.os }}`；条件使用 `{% if ... %}`，include 从 `.stooltemplates/` 加载。注册函数、字段与指令以 README 为准，不对其他模板语言提供自动转换。

Git 来源要求完整 HTTPS、`ssh://`、`git@host:path` 或绝对 `file://` URL，不接受用户名或 `user/repo` 猜测。公开发布包安装与仓库初始化相互独立，私有仓库必须先具备有效凭据。

普通加密文件与离线密钥命令独立；前者通过 dotfile 管理，后者不读取配置仓库。部署只承诺单文件原子替换、覆盖前备份和逐项成功记录，不承诺多文件部署或整个 keyring 导入的事务回滚。平台及工具限制见第 6 节。

## 3. 新 Mac：从公开安装到私有仓库恢复

以下命令在 macOS 的 zsh 中执行。先安装包含 `stool key` 的构建，并确认 `stool key --help` 可用；本次离线密钥功能属于待发布源码，旧 tag 的安装器和 binary 不会自动获得这些命令。当前可[从源码构建](../README.md#从源码构建)，正式发布包安装不需要 Rust。

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
sudo -v
stool dotfile apply
stool dotfile verify
```

init 核对 Git 姓名、邮箱、签名指纹和 Homebrew prefix。`--data git_signing_key=...` 仅覆盖个人 Git 签名默认值，不是下载或恢复前置。个人与 Work 身份仍按 clone URL 规则选择；签名私钥需手动恢复。

这份 dotfiles 的完整 HOME 部署依次初始化 Homebrew、安装 Brewfile（含 GPG Suite）、部署配置、设置 macOS 偏好。`apply`、`update`、`diff` 和 `verify` 均不恢复密钥、不询问离线备份密码；`verify` 只检查配置。已有 `~/.gitconfig` 可能冲突，先看 diff，再选择备份覆盖；非交互时检查后才使用 `apply --force`。

若第一步仅恢复了 SSH，软件包安装后确认 PATH 中有 `gpg`，再手动导入 GPG：

```bash
command -v gpg
gpg --version
stool key restore --gpg --input /Volumes/Backup/my-keys.tar.age
gpg --list-secret-keys --fingerprint \
  2BF609784E75AAC78CB67A7E465799B005224003 \
  98A56F40C2A502230AC6635435DF2F117D3233C5
```

GPG Suite/pinentry 可能另行要求私钥原口令。已在最开始恢复全部密钥时无需再次恢复。实际签名检查、备份参数及失败重试见 [离线密钥说明](key-backup.md)。后续配置更新运行 `stool dotfile update`；密钥变化时另行创建新的离线备份。

## 4. 示例仓库当前配置

`szwathub/dotfiles` 作为私有仓库维护，使用 stool 路径协议和 MiniJinja；只保存配置与签名指纹，不包含 GPG/SSH 公钥、私钥或加密备份。

| 当前 source | 部署或处理方式 |
|---|---|
| `dot_gitconfig.tmpl`、`dot_gitignore` | Git 配置与忽略规则，按本机数据渲染 |
| `dot_config/fish/config.fish.tmpl` | Fish 配置 |
| `dot_config/fish/external_functions/` | 固定提交的 HTTPS 子模块，初始化保留 `--recurse-submodules` |
| `private_dot_ssh/private_config` | SSH 连接配置及 IdentityFile 路径 |
| `dot_vimrc`、`dot_vim/colors/solarized.vim` | 基础 Vim 与配色，不安装插件管理器或插件 |
| `.stooltemplates/Brewfile` | 包含 GPG Suite，避免重复声明 gnupg/pinentry-mac |
| `private_dot_gnupg/` | GPG/agent 配置，不包含 keyring |

三个系统钩子是 `run_once_before_00-homebrew.sh.tmpl`、`run_onchange_before_10-packages.sh.tmpl`、`run_onchange_after_20-macos.sh.tmpl`。数字决定同阶段排序；配置部署不会自动恢复密钥。

Git clone 的 `post-checkout` 钩子读取 `dot_git-clone-init.tmpl` 部署出的规则，设置仓库本地 `user.name`、`user.email`、`user.signingkey`。GitHub 使用个人数据，Gitee/工作域名使用 Work 配置；它不覆盖已有仓库身份，也不是每次操作实时重新选钥。HTTPS 和 SSH 的 URL 模式都在规则内。

将仓库设为私有不会撤回过去已被下载、缓存或 fork 的副本；旧加密备份从当前树删除也不撤回历史副本；本次不改写历史。[GitHub 可见性说明](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility)

## 5. 当前实现结构与执行语义

### 5.1 单一 crate 与模块职责

当前仍是 `src/dotfile/` 下的模块，不存在已拆出的 `stool-dotfile-core` 等独立 crates：

| 模块 | 职责 |
|---|---|
| `src/cli/dotfile/mod.rs`、`mod.rs` | clap 命令定义、写锁、分发和统一流程 |
| `source.rs`、`template.rs` | 属性/控制项扫描、数据与模板渲染 |
| `plan.rs`、`maintenance.rs` | 变更计算、冲突、写入备份与导入/编辑 |
| `storage.rs`、`scripts.rs` | 路径保护、本机配置、JSON 基线与 redb 脚本状态 |
| `git.rs`、`native_git.rs`、`prerequisites.rs` | 系统 Git / gix、初始化更新、运行时工具检查 |
| `crypto.rs`、`secrets.rs` | 普通文件加密与秘密适配 |
| `src/key/{mod,gpg,ssh}.rs` | 独立离线密钥容器、GPG 与 SSH |
| `external.rs`、`archive.rs` | 外部资源与归档 |
| `process.rs`、`tools.rs`、`ui.rs` | 子进程、辅助命令、终端交互 |

当前使用 `PathBuf` 和运行时路径校验、`anyhow::Result` 与上下文错误；未使用原草图中的 `AbsPath/RelPath` newtype 和 `thiserror::DotfileError`。外部 CLI 和渲染流程目前主要同步执行，未引入 Rayon/Tokio 并发层。

### 5.2 统一 Plan 与部分成功

1. 加载 source、配置与模板数据；扫描和渲染目标。
2. 加载 JSON 文件基线，构造文件和脚本的 `Plan`。
3. 检查路径越界、source/control 重叠、重复目标、类型/权限和用户修改冲突。
4. apply 运行 before 钩子，按顺序部署文件和 during 脚本，再运行 after 钩子。
5. 文件写入成功后保存相应基线；脚本只保存成功运行状态。

文件写入使用同目录临时文件和替换；已有父目录为符号链接时拒绝绕过目标边界。覆盖与显式删除前备份文件及元数据。IO 或钩子失败保留已完成项，重试重新计算实际状态；`--force` 不清理 Git 工作树，也不提供整批回滚。

文件基线写入私有的 JSON 状态文件，脚本成功状态写入 redb。两种存储共用命令写锁，并按 source/destination 隔离；不能将其描述为“所有 EntryState 已进入 redb 事务”。脚本预览通过临时 redb 快照读取，避免对原数据库进行写打开。

### 5.3 预览、模板与外部副作用边界

`diff`、`status`、`apply --dry-run` 复用变更计划，stool 不运行部署 run 钩子、不执行目标写入或保存成功状态。它们仍可能为计算目标调用秘密提供器、解密普通加密文件、运行 `modify_` 转换脚本，或临时下载外部资源；不是任意仓库的零子进程、零联网模式；这些外部程序没有被沙箱限制，仍可能产生自己的副作用。diff/archive 可能包含渲染后的明文，应按文件敏感程度保存。

`managed` 不解密普通 encrypted source 文件，但数据、ignore/remove 和 external 模板仍会加载，不能保证所有情况下完全不调用外部工具。

dotfile 的预览、部署、更新、验证均不读取离线包，也不询问备份密码；普通加密 source 仍可能运行 GPG 解密。

### 5.4 普通文件加密与离线密钥

普通 `add --encrypt`、`edit`、`apply` 使用内部 age/GPG。加密编辑为编辑器在私有临时目录中生成明文，随后重加密；普通口令型 source 的 `STOOL_AGE_PASSPHRASE` 路径保留。秘密适配器使用各自已认证工具，不能把离线密钥的约束泛化为第三方服务。

离线密钥采用单文件、密码加密 tar.age，内部 JSON 清单、公钥、私钥、信任和已有撤销证书一并加密。GPG 默认收集全部可导出本地组，SSH 递归收集 `.ssh` 内真实私钥并保留原字节；缺失或硬件私钥不可恢复。备份输出和恢复目标须在 Git 外，拒绝覆盖不同文件、链接和越界路径。备份隐藏输入并确认密码，恢复一次输入；原私钥口令保留。

恢复先校验认证、清单、摘要、完整指纹和 SSH 配对，再检查目标冲突和 GPG keyring 的链接；GPG 导入后检查原本可导出私钥材料，成功才继续写 SSH/撤销证书。中间明文只在内存和管道，最终目标私钥受权限保护。GPG 导入和多文件写入可能部分完成，不提供整体回滚，也不写部署恢复记录。全部参数、范围、权限及失败重试见 [离线密钥说明](key-backup.md) 与 [实现说明](dotfile-internals.md#6-独立离线密钥生命周期)。

## 6. 平台、测试与验收证据

### 6.1 运行时依赖

| 功能 | 依赖或限制 |
|---|---|
| 安装发布包 | 系统下载、归档、SHA-256 工具；安装器不编译 Rust |
| 默认 Git 与子模块 | 系统 Git；私库先提供有效凭据 |
| `--builtin` | 普通仓库可免系统 Git；SSH 仍用 SSH 客户端，子模块/LFS 仍需系统工具 |
| GPG | 本机 GPG；macOS 示例由 dotfiles 安装 GPG Suite |
| 秘密提供器 | 对应官方 CLI、系统存储及其已完成的认证 |
| 安装软件/系统设置 | 仓库特定脚本、平台、网络与管理员权限；不是 stool 自身的零依赖承诺 |

| 平台能力 | macOS / Linux | Windows |
|---|---|---|
| 文件、模板、归档与公共命令 | Rust 实现 | 共享 Rust 实现 |
| private/readonly | Unix mode；只读目录写入时按需临时解锁 | PowerShell DACL 与只读属性 |
| executable | Unix execute bits | execute bits 归一化 |
| 符号链接 | 系统 symlink | 需要开发者模式或相应权限 |
| 脚本 | shebang 或 sh | shebang 或 PowerShell |
| 离线密钥 | 内置 age；GPG 操作需本机 gpg；OpenSSH 格式的 SSH-only 不需 Git/GPG | 共享 Rust/ACL；GPG 需自行安装，真实 Windows GPG 尚需账户级验收 |

### 6.2 已有测试与边界

- Rust 测试覆盖属性、模板、计划、冲突、备份、失败重试、外部资源、CLI、密码输入、GPG 命令和 Git 前置；夹具使用临时 Git/HTTP、隔离 HOME 和模拟第三方 CLI。
- Python 安装器测试检查下载、校验、平台识别和已有程序保护；Unix 的真实 GPG 测试生成一次性密钥，验证多密钥、子密钥、签名/解密、信任、撤销证书、SSH 配对与原口令保护。
- Windows 原生命令、ACL 等由对应 Rust/CI 测试验证；Unix 真实 GPG 测试不等于真实 Windows GPG 的账户级验收。
- 秘密提供器与 LFS 的模拟适配测试不等于所有真实服务、账户和私有 LFS 端点已认证验收。
- 已发布 v0.5.0 的 [发布检查](https://github.com/Xyrev/xyrev/actions/runs/39821731785)完成三系统检查、MSRV 和六平台构建。后续本机隔离验证跑通公开安装器以及通过既有 SSH 凭据读取私库的 init/diff/apply/verify/update；该 SSH 验证关闭了系统安装和真实 GPG 恢复，不能宣称所有新机已自动安装或所有 HTTPS 账户已验证。
- v0.6.0 的验收独立于历史版本：按本节命令检查新命名、新离线格式、安装打包和重新初始化流程，并以对应 tag 的 Release 工作流结果确认发布；固定旧测试总数不充当持续验收结论。

当前测试策略使用 Rust 单元/集成测试及 Python 测试，不存在已经采用的 proptest/insta/txtar 框架。实际 CI 矩阵、工具链与条件见 [Check 工作流](../.github/workflows/ci.yml)和 [Release 工作流](../.github/workflows/release.yml)。

维护代码时使用：

```bash
cargo test --locked --all-targets
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo +1.74.0 test --locked --all-targets
python3 -m unittest discover -s tests -p 'test_installer.py' -v
STOOL_BIN=target/debug/stool python3 -m unittest discover -s tests -p 'test_keys_native.py' -v
```

真实 GPG 测试需先构建 binary 并安装 GPG，使用一次性测试 keyring；不应拿用户私钥作为自动验收夹具。文档命令检查只做帮助/语法和链接校验，不执行用户登录、软件安装或恢复真实密钥。

## 7. 长期规划与未采用方案

下面是可单独评估的规划，不属于当前发布功能，也没有以草图形式承诺已经实现：

| 方向 | 当前方案 | 后续评估内容 |
|---|---|---|
| workspace 拆分 | 单 crate、按 dotfile 模块分工 | 需求明确时拆核心/CLI/适配层，再证明依赖边界与复用价值 |
| Vfs trait 与装饰器 | 统一扫描和 Plan 复用预览、应用、归档 | 是否需要内存 FS、更多后端；不为形式一致重写现有正确行为 |
| 类型化路径/错误 | PathBuf、运行时校验、anyhow 上下文 | AbsPath/RelPath newtype、thiserror 分类及迁移兼容性 |
| 并发渲染/秘密请求 | 同步执行、进程内缓存 | Rayon/Tokio 与限流；先定义顺序、提示、缓存和错误安全约束 |
| 状态统一 | 文件 JSON、脚本 redb | 有明确迁移需求时评估统一存储；保留恢复与兼容路径 |
| 属性/模板与后端扩展 | 第 2 节明确的有限协议 | GitHub 来源简写、更多路径能力或加密集成须单独定义需求、迁移与测试 |
| 测试工具 | 普通 Rust/Python 测试 | 有收益时增加 property、snapshot 或 txtar 驱动，不能将工具选型等同覆盖完成 |
| 更顺畅的 GitHub 登录 | 系统 Git/凭据助手和用户先完成的认证 | 可评估 OAuth/device-flow/系统钥匙串引导；当前没有原生登录命令 |

未列入当前能力的方向不应标为已完成，也不应阻塞已支持的 Git URL 初始化和部署。任何后续实现都应先明确兼容约束、状态迁移和可复现的验收条件。

## 8. 文档与维护约定

- [产品概览](dotfile-overview.md)：能力、运行边界及版本兼容性变化。
- [实现说明](dotfile-internals.md)：当前 Rust 模块、Plan、JSON/redb 和安全边界。
- [用户命令与配置](../README.md)、[发布说明](../RELEASE_NOTES.md)、[CLI 定义](../src/cli/dotfile/mod.rs)：区分当前源码与已发布行为。
- [私有示例仓库](https://github.com/szwathub/dotfiles)：登录授权后读取；匿名 404 属于预期。

修改功能后同步协议、迁移说明和测试证据；修改推荐引导路径时同步 README。将当前能力、待发布变化和长期规划分别记录，不用未落地的结构草图代替实现证明。
