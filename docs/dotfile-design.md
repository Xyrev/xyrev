# stool dotfile：实现规范、兼容边界与新机器流程

> 校订日期：2026-10-24。当前版本为 stool v0.6.0（待发布），命令与协议按当前源码描述。历史 xyrev v0.5.0 的 tag、资源、固定安装地址与二进制行为不变。
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
| 维护与诊断 | 显式 migrate、add/edit/unmanaged、递归导入、编辑器、root/version、verify/doctor/data/execute-template/cd | [maintenance.rs](../src/dotfile/maintenance.rs)、[tools.rs](../src/dotfile/tools.rs) |
| 脚本 | run/once/onchange、before/during/after 顺序；成功才持久化，失败可重试，onchange 的 A→B→A 再执行 | [scripts.rs](../src/dotfile/scripts.rs)、[钩子测试](../tests/dotfile_crypto_hooks.rs) |
| 普通文件加密 | 内置 age；本机 GPG 子进程；通过 add --encrypt 导入、edit 编辑、apply 解密部署；保留内部 age 密码处理 | [crypto.rs](../src/dotfile/crypto.rs)、[加密文件测试](../tests/dotfile_advanced.rs) |
| GPG 密钥备份与恢复 | 默认备份全部可导出本地私钥、可选完整主指纹筛选、age 密码/身份文件保护、清单校验、原生恢复与失败重试 | [gpg.rs](../src/dotfile/gpg.rs)、[命令测试](../tests/dotfile_gpg.rs)、[真实 GPG 测试](../tests/test_gpg_native.py) |
| 秘密提供器 | 1Password、Bitwarden、pass/gopass、Vault、Keyring、KeePassXC、LastPass、Doppler、AWS、Azure 及通用命令适配；进程内缓存 | [secrets.rs](../src/dotfile/secrets.rs)、[高级测试](../tests/dotfile_advanced.rs) |
| 外部资源与归档 | 文件、归档、归档中单文件、Git 资源；TTL、SHA-256、过滤、路径保护；tar/tgz/zip 导出 | [external.rs](../src/dotfile/external.rs)、[archive.rs](../src/dotfile/archive.rs) |
| 终端和命令文档 | ratatui 冲突确认；由同一 clap 定义生成 shell 补全和递归 man pages | [ui.rs](../src/dotfile/ui.rs)、[CLI 定义](../src/cli/dotfile/mod.rs)、[生成脚本](../scripts/generate-cli-assets.sh) |

源码组织与状态存储见第 5 节；运行环境和验收范围见第 6 节。产品命名统一为 `stool`，模块入口统一为 `dotfile`。

### 1.2 常用命令

```text
stool dotfile migrate [--from <old-control-dir>] [--dry-run]
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
stool dotfile gpg backup [--fingerprint <primary-fingerprint>] [options]
stool dotfile gpg restore [options]
```

`--source`、`--destination` 和可重复的 `--data key=value` 是 dotfile 全局选项。覆盖值支持 JSON 和点分嵌套路径；普通调用中的覆盖仅对本次有效。init 的配置模板会保存用户确认的本机数据。

默认控制目录是 `~/.stool/dotfile/`：

| 路径 | 用途 |
|---|---|
| `source/` | 默认 Git 工作树 |
| `config.toml` | 仓库 URL、分支、目录、选项及本机数据 |
| `state/<namespace>.json` | 按 source/destination 配对隔离的文件基线及 GPG 恢复摘要 |
| `state/scripts.redb` | 按同一配对隔离的脚本成功状态 |
| `backups/` | 覆盖和删除前的备份、元数据 |
| `cache/` | 部署时的外部资源缓存 |

### 1.3 待发布兼容性变更

v0.6.0 将项目、Cargo package/bin、命令和发布资源统一为 stool，仓库为 `szwathub/stool`。本版本尚未发布，旧 tag 和资源不重命名。当前可从源码构建；本文的新安装 URL 仅在 v0.6.0 发布后可用，历史 xyrev 安装另见 [README](../README.md#安装与版本边界)。

source 控制项、模板数据/指令、环境变量和默认目录使用 `.stool*`、`stool.*` / `stool:template:`、`STOOL_*` 与 `~/.stool`。不自动加载旧协议，旧环境应通过显式迁移流程处理，并核对本机配置、状态与 source 引用；不因启动新程序而隐式改写远端仓库。

独立加解密子命令已移除。普通 dotfile 用 `add --encrypt`、`edit`、`apply` 管理；GPG 密钥使用 `gpg backup` / `gpg restore` 与 `.stoolgpg.toml[.tmpl]`。内部 age/GPG、密码处理、既有密文、`keygen` 和 GPG 备份格式保留，无需重加密。

### 1.4 旧控制目录迁移

`stool dotfile migrate [--from <old-control-dir>] [--dry-run]` 默认读取旧 `~/.xyrev/dotfile`，目标为 `~/.stool/dotfile`。使用保存的来源、目标和状态，不接受 source/destination/data 全局覆盖。

先由用户用正常 Git 操作准备好 `.stool*` 协议的 source，再迁移；命令不联网、不改 tracked 仓库内容、不自动 apply。`--dry-run` 只检查，不写目录、不执行 hooks/GPG/秘密 CLI，不求值 env()/prompt 模板函数。

实际迁移复制旧控制数据到新根的暂存目录，转换本机配置与可证明等价的状态后原子发布；旧目录保留回退，新目标只要已存在就停止，同一收据重复调用报告已迁移除外。未知历史 namespace 保留；未能证明纯改名等价的脚本/GPG 成功记录不继承，继续显示为待执行。完整命令顺序见 [README](../README.md#从旧环境迁移)。

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

原生 GPG 声明只接受 `.stoolgpg.toml` 或 `.stoolgpg.toml.tmpl`，两者不能并存。未知的保留控制项报错。Git 元数据不部署，普通非字面隐藏 source 项跳过。具体字段见 [README](../README.md#仓库路径协议)。

### 2.2 模板、来源与部署边界

模板引擎为 MiniJinja，机器数据位于 `stool.*`，例如 `{{ stool.os }}`；条件使用 `{% if ... %}`，include 从 `.stooltemplates/` 加载。注册函数、字段与指令以 README 为准，不对其他模板语言提供自动转换。

Git 来源要求完整 HTTPS、`ssh://`、`git@host:path` 或绝对 `file://` URL，不接受用户名或 `user/repo` 猜测。公开发布包安装与仓库初始化相互独立，私有仓库必须先具备有效凭据。

普通加密文件和 GPG 密钥备份分别管理；内部 age/GPG 供导入、编辑、部署及原生密钥命令调用。部署只承诺单文件原子替换、覆盖前备份和逐项成功记录，不承诺多文件部署或整个 keyring 导入的事务回滚。平台及工具限制见第 6 节。

## 3. 新 Mac：从公开安装到私有仓库恢复

以下命令用于 macOS 系统 zsh；Fish 用户可先进入 `/bin/zsh`。示例私库 `szwathub/dotfiles` 已完成命名迁移并设为私有，`szwathub/stool` 仓库保持公开，v0.6.0 尚未发布；本节以已构建的 stool 为起点。首次引导采用 **HTTPS + 独立 GitHub 凭据**，不依赖尚未下载的 SSH/GPG 密钥；已有 SSH 认证仍可使用 SSH URL。

### 3.1 安装 stool 并准备 Git

当前先[从源码构建](../README.md#从源码构建) stool。v0.6.0 发布后可用以下安装命令；安装器按平台下载已编译包并校验 SHA-256，不下载 Rust，不要求先有 Git 或 Homebrew：

```bash
curl -fsSL https://raw.githubusercontent.com/szwathub/stool/v0.6.0/install.sh | VERSION=v0.6.0 bash
export PATH="$HOME/.local/bin:$PATH"
stool --version
```

期望版本为 `stool 0.6.0`。这不是已发布版本的下载承诺；新安装器不会把历史 xyrev 包当作 stool 安装。上面的 PATH 仅影响当前终端；zsh 用户可将该 export 行加入 `~/.zprofile`，Fish 用户可用 `fish_add_path ~/.local/bin` 持久添加。

检查 Git：

```bash
git --version
```

Git 不可用时执行下面命令，并在系统窗口完成 Command Line Tools 安装：

```bash
xcode-select --install
```

安装完成后重新运行 `git --version`。默认后端及 `--recurse-submodules` 在 clone 前检查 Git；macOS 会识别开发工具占位程序并给出指引，stool 不会代替用户完成系统安装。

### 3.2 独立完成 HTTPS 认证

在 GitHub 创建仅授权目标仓库、具有 Contents 读取权限的 [fine-grained PAT](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)。只下载和更新 source 使用读取权限；需要推送备份或配置时另需写权限。PAT 放在密码管理器或系统凭据存储，不放在 dotfiles、自定义 URL、命令参数、环境变量或聊天中。

先在系统 Git 的交互终端完成认证并缓存到 macOS Keychain：

```bash
git config --global credential.https://github.com.helper osxkeychain
git ls-remote https://github.com/szwathub/dotfiles.git HEAD
```

Username 输入有仓库权限的 GitHub 用户名，Password 输入 PAT；GitHub 账户登录密码不用于 HTTPS Git 认证。预检成功应返回 HEAD 提交号。默认系统 Git 后端设置 `GIT_TERMINAL_PROMPT=0`，因此必须先在 stool 外完成这一步；浏览器登录本身不会自动为 Git 提供凭据。官方说明见 [GitHub HTTPS 认证](https://docs.github.com/en/get-started/git-basics/about-remote-repositories)和 [Git 凭据助手](https://git-scm.com/docs/gitcredentials)。

已安装并配置 Git Credential Manager 或 GitHub CLI 的机器也可复用其 HTTPS 凭据；它们不是全新机器在首次 apply 之前必须安装的依赖。stool 本身没有 `github login` 或 OAuth/device-flow 登录命令。

### 3.3 初始化、预览和部署

```bash
stool dotfile init https://github.com/szwathub/dotfiles.git --recurse-submodules
stool dotfile diff
```

init 默认只保存来源、目录和用户确认的数据；未加 `--apply` 不部署。初始化时核对 Git 身份和 Homebrew prefix。`git_signing_key` 的覆盖参数不是下载或恢复前置；它是个人 Git 签名默认值，也可在初始化提示中选择。推荐使用个人完整主指纹，让 GPG 选择可用签名子密钥；Work 仓库身份由 dotfiles 的 URL 匹配规则另行配置。

HTTPS 凭据步骤会创建或改动 `~/.gitconfig`。首次部署若报告该文件冲突，先看 diff，再在终端选择备份覆盖；非交互时可检查后显式使用 `apply --force`。dotfiles 部署的 macOS Git 配置仍启用 `osxkeychain`，已缓存的凭据不依赖被替换的配置文本保存。

确认变更后，在同一 Terminal 准备管理员权限并部署：

```bash
sudo -v
stool dotfile apply
stool dotfile verify
```

这份 dotfiles 的完整 HOME 部署顺序为：

1. 复用或初始化 Homebrew，运行 before 软件包钩子；Brewfile 包含 GPG Suite，不另行重复安装 gnupg/pinentry-mac。
2. 部署普通配置。
3. 根据 `.stoolgpg.toml.tmpl` 执行原生 GPG 恢复。
4. 执行 after 阶段的 macOS 设置。

`sudo -v` 输入本机管理员密码；`GPG backup password:` 输入原备份密码，隐藏输入一次恢复个人和 Work 两把主密钥及其子密钥、备份内的 ownertrust。若 GPG Suite/pinentry 另有提示，输入对应私钥原来的口令。现有密码备份无需 agekey，也无需重新备份或加密。

| 凭据 | 用途和取得时机 |
|---|---|
| GitHub PAT / 其他独立 Git 凭据 | init 之前用于读取私库，由密码管理器、Keychain 或已配置助手提供 |
| GPG 备份密码 | 获取备份后，由 stool 解密 age 密文 |
| GPG 私钥原口令（若有） | GPG 导出或签名等私钥操作时由 pinentry 管理 |
| SSH 登录密钥（可选） | 只在选择 SSH 传输时需要；首次 HTTPS 下载不依赖它 |

当前私库只保存 SSH 配置及其中的 `IdentityFile` 路径，不包含 `github_rsa` 等 SSH 私钥。Git 的 GPG 提交签名与 SSH 登录认证互相独立。以后即使增加加密的 SSH 密钥备份，首次私库访问也应有独立的 HTTPS 凭据；按需为每台机器生成独立 SSH 登录密钥也是可行替代。

### 3.4 检查恢复与后续维护

`verify` 检查文件部署计划和已记录的 GPG 恢复摘要，不会执行恢复、测试签名或保证 keyring 中的密钥未被之后手动删除；待执行脚本看 `status`。另用 GPG 列出两把私钥：

```bash
/usr/local/MacGPG2/bin/gpg --list-secret-keys --keyid-format LONG --fingerprint \
  2BF609784E75AAC78CB67A7E465799B005224003 \
  98A56F40C2A502230AC6635435DF2F117D3233C5
```

| 身份 | 完整主密钥指纹 |
|---|---|
| 个人 `szwathub@gmail.com` | `2BF609784E75AAC78CB67A7E465799B005224003` |
| Work `cyrex@funmobig.com` | `98A56F40C2A502230AC6635435DF2F117D3233C5` |

需要验证私钥实际可用时，在临时文件上分别签名、验签；仅产生测试消息与公开签名，不导出私钥：

```bash
stool_gpg_check_dir=$(mktemp -d /tmp/stool-gpg-check.XXXXXX)
printf 'stool GPG restore check\n' > "$stool_gpg_check_dir/message.txt"
for stool_gpg_key in \
  2BF609784E75AAC78CB67A7E465799B005224003 \
  98A56F40C2A502230AC6635435DF2F117D3233C5
do
  /usr/local/MacGPG2/bin/gpg --local-user "$stool_gpg_key" --detach-sign \
    --output "$stool_gpg_check_dir/$stool_gpg_key.sig" "$stool_gpg_check_dir/message.txt" &&
  /usr/local/MacGPG2/bin/gpg --verify "$stool_gpg_check_dir/$stool_gpg_key.sig" "$stool_gpg_check_dir/message.txt"
done
```

两次都应出现对应身份的 `Good signature`；可能需要对应原 GPG 口令。GPG 命令依据 [GnuPG 操作说明](https://www.gnupg.org/documentation/manuals/gnupg/Operational-GPG-Commands.html)。

日常同步仍使用已保存的 HTTPS 来源：

```bash
stool dotfile update
```

Git 凭据过期时先重新完成交互预检；管理员权限过期时重新 `sudo -v` 后重试 `apply`。备份密码错误或 GPG 导入失败时修复原因后重试 `apply`，无需重新 init。显式重新恢复使用 `stool dotfile gpg restore`，只同步配置可单次加 `--data restore_gpg=false`。

独立 destination 用于隔离试运行，自动 GPG 恢复会跳过；示例私库的系统安装钩子也只在 macOS 且 destination 等于 HOME 时运行。不能用隔离部署成功来声称真实系统软件安装和密钥恢复已经完成。

## 4. 示例仓库当前配置

`szwathub/dotfiles` 作为私有仓库维护，使用 stool 路径协议和 MiniJinja；不包含 SSH 登录私钥。

| 当前 source | 部署或处理方式 |
|---|---|
| `dot_gitconfig.tmpl`、`dot_gitignore` | Git 配置与忽略规则，按本机数据渲染 |
| `dot_config/fish/config.fish.tmpl` | Fish 配置 |
| `dot_config/fish/external_functions/` | 固定提交的 HTTPS 子模块，初始化保留 `--recurse-submodules` |
| `private_dot_ssh/private_config` | SSH 连接配置及 IdentityFile 路径 |
| `dot_vimrc`、`dot_vim/colors/solarized.vim` | 基础 Vim 与配色，不安装插件管理器或插件 |
| `.stooltemplates/Brewfile` | 包含 GPG Suite，避免重复声明 gnupg/pinentry-mac |
| `.stoolgpg.toml.tmpl` | 声明原生 GPG 恢复 |
| `.stooltemplates/gpg/` | 个人和 Work 公钥及 age 密文，沿用现有备份密码 |

三个系统钩子是 `run_once_before_00-homebrew.sh.tmpl`、`run_onchange_before_10-packages.sh.tmpl`、`run_onchange_after_20-macos.sh.tmpl`。数字决定同阶段排序；GPG 恢复在文件部署后、after 钩子前由 stool 执行。

Git clone 的 `post-checkout` 钩子读取 `dot_git-clone-init.tmpl` 部署出的规则，设置仓库本地 `user.name`、`user.email`、`user.signingkey`。GitHub 使用个人数据，Gitee/工作域名使用 Work 配置；它不覆盖已有仓库身份，也不是每次操作实时重新选钥。HTTPS 和 SSH 的 URL 模式都在规则内。

将仓库设为私有不会撤回过去已被下载、缓存或 fork 的副本；加密备份继续使用强、独立的密码，私有访问控制不替代加密。[GitHub 可见性说明](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility)

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
| `crypto.rs`、`gpg.rs`、`secrets.rs` | 文件加密、GPG 密钥生命周期与秘密适配 |
| `external.rs`、`archive.rs` | 外部资源与归档 |
| `process.rs`、`tools.rs`、`ui.rs` | 子进程、辅助命令、终端交互 |

当前使用 `PathBuf` 和运行时路径校验、`anyhow::Result` 与上下文错误；未使用原草图中的 `AbsPath/RelPath` newtype 和 `thiserror::DotfileError`。外部 CLI 和渲染流程目前主要同步执行，未引入 Rayon/Tokio 并发层。

### 5.2 统一 Plan 与部分成功

1. 加载 source、配置与模板数据；扫描和渲染目标。
2. 加载 JSON 文件基线，构造 `Plan`；附加声明式 GPG 恢复计划。
3. 检查路径越界、source/control 重叠、重复目标、类型/权限和用户修改冲突。
4. apply 运行 before 钩子，按顺序部署文件和 during 脚本，执行原生 GPG 恢复，再运行 after 钩子。
5. 文件写入成功后保存相应基线；脚本只保存成功运行状态；完整 GPG 恢复成功后才保存摘要。

文件写入使用同目录临时文件和替换；已有父目录为符号链接时拒绝绕过目标边界。覆盖与显式删除前备份文件及元数据。IO 或钩子失败保留已完成项，重试重新计算实际状态；`--force` 不清理 Git 工作树，也不提供整批回滚。

文件基线和 GPG 摘要写入私有的 JSON 状态文件，脚本成功状态写入 redb。两种存储共用命令写锁，并按 source/destination 隔离；不能将其描述为“所有 EntryState 已进入 redb 事务”。脚本预览通过临时 redb 快照读取，避免对原数据库进行写打开。

### 5.3 预览、模板与外部副作用边界

`diff`、`status`、`apply --dry-run` 复用变更计划，stool 不运行部署 run 钩子、不执行目标写入或保存成功状态。它们仍可能为计算目标调用秘密提供器、解密普通加密文件、运行 `modify_` 转换脚本，或临时下载外部资源；不是任意仓库的零子进程、零联网模式；这些外部程序没有被沙箱限制，仍可能产生自己的副作用。diff/archive 可能包含渲染后的明文，应按文件敏感程度保存。

`managed` 不解密普通 encrypted source 文件、不准备原生 GPG 恢复，但数据、ignore/remove 和 external 模板仍会加载，不能保证所有情况下完全不调用外部工具。

原生 GPG 备份的预览仅校验公开清单与密文摘要、显示待恢复项，不询问备份密码、不为该恢复计划运行 GPG。此保障不能扩大为“任何使用 GPG 的 source 在 diff 时都不运行 GPG”。

### 5.4 加密、秘密与原生 GPG

普通文件由 `add --encrypt`、`edit` 和 `apply` 调用内部 age/GPG 实现，通过内存与进程管道处理；独立加解密子命令的待发布移除不影响这些流程。加密编辑为编辑器在私有临时目录中生成明文，结束后重新加密。内置 age 支持 X25519、多接收方、密码保护和 armor；普通加密 source 保留 `STOOL_AGE_PASSPHRASE` 兼容路径，秘密提供器也复用各自已认证 CLI 和进程凭据。不能把原生 GPG 的安全约束泛化为全部第三方后端。

原生 GPG 密钥备份格式保持四文件兼容：

| 文件 | 内容 |
|---|---|
| `public.asc` | 所选主密钥及子密钥的公钥 |
| `secret.asc.age` | age 加密的私钥导出 |
| `ownertrust.txt.age` | age 加密的所选密钥信任数据 |
| `backup-metadata.sh` | 模式、主指纹与 SHA-256；严格解析的字段数据，不执行 Shell |

`stool dotfile gpg backup` 默认发现并备份当前 keyring 中全部可导出的本地私钥、对应公钥和 ownertrust，按完整主指纹分组；只有公钥的条目不纳入。过期或撤销的本地密钥仍备份。离线主密钥配有本机私有子密钥时，也会纳入该组并导出可用私钥；缺失或硬件中的私钥只有公开部分可备份，并明确提示这些私钥无法由备份恢复。完全没有本地可导出私钥的组会跳过并提示；空 keyring 或全部不可导出时，在请求密码或改写既有备份前失败。

可选的 `--fingerprint` 只接受完整主指纹，可重复筛选多个主密钥组；不存在或完全不可导出的显式选择会失败。覆盖时拒绝遗漏已有清单中的主密钥及旧 `public.asc` 中的主/子指纹，筛选不能用于缩减既有备份。若既有组当前含有缺失或硬件私钥组件，旧格式未记录私钥可用性，无法证明原私钥材料不丢失，会保守拒绝覆盖。部分私钥组可首次备份到新目录；重复备份需保留旧备份并使用全新 `--output` 目录，不读取旧密码或合并旧私钥密文。

默认隐藏输入和确认 age 备份密码；原私钥口令由 GPG/pinentry 管理。支持仓库外的私有密码文件、独立 age 身份文件和 GPG 路径覆盖。备份也支持读取继承的管道描述符；恢复的密码文件要求为仓库外、非链接的私有普通文件。新密码模式不依赖 agekey，旧身份文件模式仍需其独立身份文件。

恢复先验证三份数据的 SHA-256、清单字段、公私钥完整主指纹集合及 ownertrust 的指纹/值范围，再导入真实 HOME keyring。支持历史单/多主指纹、密码模式、身份模式与缺省模式的旧清单。SHA-256 是一致性校验，不是独立的来源身份认证；仓库与配置仍须可信。

原生备份/恢复中，私钥明文和备份密码仅在受控内存及管道中处理，不把它们写入命令参数、环境变量、日志、普通状态或明文临时文件。GPG 声明不作为普通 dotfile 部署；默认 `.stooltemplates/gpg/` 内的支持文件位于控制目录，不部署或进入 archive。自定义备份目录也应放在控制目录或明确排除，普通 source 路径不会因用于 GPG 自动跳过。

恢复摘要只有成功后保存；备份变化会触发重新恢复，A→B→A 也会再运行。显式 `gpg restore` 可再次导入；后续相同计划的 apply 不重复导入。真实 GPG 导入失败可能留下已导入的部分，重试依赖 GPG 的导入语义，不提供 keyring 事务回滚。

备份在验证完整输出后写入暂存目录，最后发布清单；四文件替换不是跨文件事务。写入中断可能造成清单与数据不一致，后续恢复校验失败并停止，不应承诺任意 IO 失败下旧四文件始终完全不变。

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
| 示例仓库自动 GPG 恢复 | macOS / 已有 GPG 的 Linux 开启 | 示例默认关闭；安装 GPG 后可显式恢复 |

### 6.2 已有测试与边界

- Rust 测试覆盖属性、模板、计划、冲突、备份、失败重试、外部资源、CLI、密码输入、GPG 命令和 Git 前置；夹具使用临时 Git/HTTP、隔离 HOME 和模拟第三方 CLI。
- Python 安装器测试检查下载、校验、平台识别和已有程序保护；Unix 的真实 GPG 测试生成一次性密钥，验证多密钥、两种加密模式、历史兼容、隐藏密码和原私钥口令保护。
- Windows 原生命令、ACL 等由对应 Rust/CI 测试验证；Unix 真实 GPG 测试不等于真实 Windows GPG 的账户级验收。
- 秘密提供器与 LFS 的模拟适配测试不等于所有真实服务、账户和私有 LFS 端点已认证验收。
- 已发布 v0.5.0 的 [发布检查](https://github.com/Xyrev/xyrev/actions/runs/39821731785)完成三系统检查、MSRV 和六平台构建。后续本机隔离验证跑通公开安装器以及通过既有 SSH 凭据读取私库的 init/diff/apply/verify/update；该 SSH 验证关闭了系统安装和真实 GPG 恢复，不能宣称所有新机已自动安装或所有 HTTPS 账户已验证。
- 上述 v0.5.0 发布证据只对应历史 xyrev，不代表 stool v0.6.0 已经发布或完成远端 CI。当前改名须重新验证构建、安装打包与迁移；固定旧测试总数不充当持续验收结论。

当前测试策略使用 Rust 单元/集成测试及 Python 测试，不存在已经采用的 proptest/insta/txtar 框架。实际 CI 矩阵、工具链与条件见 [Check 工作流](../.github/workflows/ci.yml)和 [Release 工作流](../.github/workflows/release.yml)。

维护代码时使用：

```bash
cargo test --locked --all-targets
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo +1.74.0 test --locked --all-targets
python3 -m unittest discover -s tests -p 'test_installer.py' -v
STOOL_BIN=target/debug/stool python3 -m unittest discover -s tests -p 'test_gpg_native.py' -v
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
| 状态统一 | 文件/GPG JSON、脚本 redb | 有明确迁移需求时评估统一存储；保留恢复与兼容路径 |
| 属性/模板与后端扩展 | 第 2 节明确的有限协议 | GitHub 来源简写、更多路径能力或加密集成须单独定义需求、迁移与测试 |
| 测试工具 | 普通 Rust/Python 测试 | 有收益时增加 property、snapshot 或 txtar 驱动，不能将工具选型等同覆盖完成 |
| 更顺畅的 GitHub 登录 | 系统 Git/凭据助手和用户先完成的认证 | 可评估 OAuth/device-flow/系统钥匙串引导；当前没有原生登录命令 |

未列入当前能力的方向不应标为已完成，也不应阻塞已支持的 Git URL 初始化和部署。任何后续实现都应先明确兼容约束、状态迁移和可复现的验收条件。

## 8. 文档与维护约定

- [产品概览](dotfile-overview.md)：能力、运行边界及当前 develop 的待发布变化。
- [实现说明](dotfile-internals.md)：当前 Rust 模块、Plan、JSON/redb 和安全边界。
- [用户命令与配置](../README.md)、[发布说明](../RELEASE_NOTES.md)、[CLI 定义](../src/cli/dotfile/mod.rs)：区分当前源码与已发布行为。
- [私有示例仓库](https://github.com/szwathub/dotfiles)：登录授权后读取；匿名 404 属于预期。

修改功能后同步协议、迁移说明和测试证据；修改推荐引导路径时同步 README。将当前能力、待发布变化和长期规划分别记录，不用未落地的结构草图代替实现证明。
