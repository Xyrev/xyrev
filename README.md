# stool

开发环境命令行工具。`stool dotfile` 通过 Git URL 初始化、预览、部署、维护和更新 dotfiles，采用 stool 路径协议、MiniJinja 模板及统一变更计划。

## v0.6.0 开发状态与迁移

项目与命令已改名为 `stool`，仓库为 `szwathub/stool`，Cargo package/bin 和下一版本统一为 `stool 0.6.0`。**v0.6.0 尚未发布**；当前可[从源码构建](#从源码构建)，下方新安装地址仅在发布后可用。

控制项、模板数据/指令、环境变量和默认目录统一使用 `.stool*`、`stool.*` / `stool:template:`、`STOOL_*` 和 `~/.stool`。不自动读取旧命名；升级旧环境应使用[显式迁移流程](#从旧环境迁移)，并检查 source、模板和外部资源引用。工具不会隐式改写远端仓库。GPG 既有备份保持兼容，无需重新加密。

独立加解密子命令已移除；普通 dotfile 通过 `add --encrypt`、`edit`、`apply` 管理，GPG 密钥通过 `gpg backup` / `gpg restore` 或原生恢复声明管理，其他文件加解密使用相应工具。内部 age/GPG 和 `keygen` 保留。

## 安装与版本边界

当前使用源码构建得到的 `stool`。v0.6.0 发布后，macOS/Linux 可通过以下命令下载已编译包并验证 SHA-256，无需先安装 Rust、Git 或 Homebrew：

```bash
curl -fsSL https://raw.githubusercontent.com/szwathub/stool/v0.6.0/install.sh | VERSION=v0.6.0 bash
export PATH="$HOME/.local/bin:$PATH"
stool --version
```

`install.sh` 与 `newinstall.sh` 保持同源；`PREFIX` 选择安装前缀，binary 位于 `$PREFIX/bin/stool`。新安装器只接受 stool v0.6.0 及之后的稳定版本；`VERSION=latest` 取最新稳定 Release，若仍指向历史版本会停止。Windows 的新资源命名为 `stool-v0.6.0-x86_64-pc-windows-msvc.zip`，发布后解压使用 `stool.exe`。

上面的 PATH 设置只影响当前终端。zsh 用户可将该 export 行加入 `~/.zprofile`，Fish 用户执行 `fish_add_path ~/.local/bin` 持久添加。

历史 xyrev v0.5.0 的 tag、脚本和资源保持原样，固定安装命令仍是：

```bash
curl -fsSL https://raw.githubusercontent.com/Xyrev/xyrev/v0.5.0/install.sh | VERSION=v0.5.0 bash
```

该命令安装历史 `xyrev`，不会安装 stool，也不用于部署已经迁移为 v0.6.0 协议的 source；历史 [v0.5.0 Release](https://github.com/Xyrev/xyrev/releases/tag/v0.5.0) 与 `xyrev-v0.5.0-*` / `xyrev.exe` 资源名称不变。

## 从旧环境迁移

先备份旧环境，并用正常 Git 操作将 source 更新或准备为 `.stool*`、`stool.*` 和 `stool:template:` 协议；确认工作树与子模块状态符合预期。迁移不联网，也不修改 tracked source 内容，因此不会替你提交或重写配置仓库。

构建当前 stool 后检查并迁移：

```bash
stool dotfile migrate --dry-run
stool dotfile migrate
stool dotfile status
stool dotfile diff
```

默认从旧 `~/.xyrev/dotfile` 迁移到 `~/.stool/dotfile`；来源不在默认位置时使用 `stool dotfile migrate --from /path/to/old-control --dry-run` 检查，再去掉 `--dry-run` 执行。migrate 使用保存的旧路径，不接受 `--source`、`--destination` 或 `--data` 覆盖。

`--dry-run` 只检查，不写入、不联网、不运行 hooks/GPG/秘密服务或求值 env()/prompt 模板函数。实际迁移先暂存，再原子发布新控制目录，保留旧目录供回退；新目录只要已存在就停止，相同收据识别出的已完成迁移除外。仅继承能证明纯改名等价的成功记录；未知 namespace 保留，不能证明等价的 hooks/GPG 仍为待执行项。迁移不自动 apply，核对状态和 diff 后再决定部署。它不会自动改名 `~/.config/xyrev` 等业务目录；涉及业务路径变化或无法证明等价的脚本需要重新执行。

## 新 Mac 初始化私有 dotfiles

`szwathub/stool` 保持公开；以下流程以已构建或未来发布的 stool v0.6.0 为起点。`szwathub/dotfiles` 是私有仓库，初始化前需要有仓库访问权的 GitHub 账户。Git 认证用于下载仓库，GPG 备份密码用于解密密钥，两者独立。`init` 发生在软件安装之前，不能依赖后续 `apply` 安装的 Homebrew、GitHub CLI 或 GPG 来完成首次 Git 认证。首次推荐 HTTPS，SSH 作为可选方式；完整步骤、密码用途和恢复边界见[新 Mac 流程](docs/dotfile-design.md#3-新-mac从公开安装到私有仓库恢复)。

### 准备 Git

在 macOS Terminal 的 zsh 中操作。先检查 `git --version`；如果提示缺少开发工具，运行：

```bash
xcode-select --install
```

等待系统安装窗口完成后，再检查 Git：

```bash
git --version
```

### HTTPS 认证（首次推荐）

首次下载采用 HTTPS，不依赖尚未取得的 SSH 或 GPG 密钥。在浏览器创建仅授权 `szwathub/dotfiles`、具有 Contents 读取权限的 [fine-grained PAT](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens)；推送时另需写权限。先让 macOS Keychain 缓存凭据：

```bash
git config --global credential.https://github.com.helper osxkeychain
git ls-remote https://github.com/szwathub/dotfiles.git HEAD
```

在 Git 的交互提示中输入 GitHub 用户名，Password 位置粘贴 PAT；不要输入账户登录密码，也不要将 PAT 放入 URL、命令参数或环境变量。预检成功应返回 HEAD 的提交哈希。默认系统 Git 后端关闭终端凭据提示，所以必须先完成该预缓存步骤；浏览器登录本身不会为 Git 提供凭据。凭据助手见 [Git 官方文档](https://git-scm.com/docs/gitcredentials)。

### SSH 认证（可选）

已有独立 SSH 登录密钥或准备为新机生成密钥时，也可选择 SSH。使用系统自带的 OpenSSH，不需要先安装 Homebrew 或 `gh`。下面沿用仓库 SSH 配置中的 `~/.ssh/github_rsa` 路径，生成的算法仍是 Ed25519。已有同名私钥或公钥时，先检查并复用，保留原文件；只在两者都不存在时生成新密钥：

```bash
mkdir -p "$HOME/.ssh"
chmod 700 "$HOME/.ssh"
if [[ -e "$HOME/.ssh/github_rsa" || -L "$HOME/.ssh/github_rsa" || -e "$HOME/.ssh/github_rsa.pub" || -L "$HOME/.ssh/github_rsa.pub" ]]; then
  printf '%s\n' '已有 github_rsa 或 github_rsa.pub，请先检查并复用，未生成新密钥。'
else
  /usr/bin/ssh-keygen -t ed25519 -C "you@example.com (new-mac)" -f "$HOME/.ssh/github_rsa"
fi
```

在终端为新 SSH 登录密钥设置口令，它与 GPG 备份密码及原 GPG 私钥口令独立。确认密钥可用后，加入 agent 并将口令保存到 macOS Keychain；复用不带口令的密钥时省略 `--apple-use-keychain`：

```bash
[ -n "${SSH_AUTH_SOCK:-}" ] || eval "$(/usr/bin/ssh-agent -s)"
/usr/bin/ssh-add --apple-use-keychain "$HOME/.ssh/github_rsa"
pbcopy < "$HOME/.ssh/github_rsa.pub"
```

`ssh-add` 需要可用的 agent；如果提示无法连接，先运行 `eval "$(/usr/bin/ssh-agent -s)"`，再重试 `ssh-add`。在浏览器登录有仓库权限的 GitHub 账户，打开 [SSH keys 设置](https://github.com/settings/keys)，添加剪贴板中的**公钥**，类型选 Authentication Key。SSH 私钥留在本机；SSH 口令、GitHub 凭据和 GPG 密码都不要发到聊天或写入仓库。agent 与 macOS Keychain 的行为见 [GitHub SSH 指南](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent?platform=mac)。

添加后，在同一个 Terminal 验证实际仓库权限：

```bash
git ls-remote git@github.com:szwathub/dotfiles.git HEAD
```

第一次连接时，对照 [GitHub 公布的 SSH 主机指纹](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/githubs-ssh-key-fingerprints) 后确认；预检应返回 HEAD 的提交哈希。可选的 `ssh -T git@github.com` 成功时仍返回退出码 **1**，需检查欢迎消息中的账户；它只验证账户认证，不能替代仓库访问预检，见 [GitHub 连接测试说明](https://docs.github.com/en/authentication/connecting-to-github-with-ssh/testing-your-ssh-connection)。

### 初始化、预览和部署

完成所选协议的仓库预检后执行。下面采用 HTTPS；选择 SSH 时，将 URL 换为 `git@github.com:szwathub/dotfiles.git`：

```bash
stool dotfile init https://github.com/szwathub/dotfiles.git --recurse-submodules
stool dotfile diff
sudo -v
stool dotfile apply
stool dotfile verify
```

`--data git_signing_key=...` 是可选的个人 Git 签名默认值，也可在初始化提示中选择；它不是 Git 认证或密钥恢复前置。工作密钥会从同一份备份恢复，仓库签名身份由 dotfiles 的 URL 匹配规则设置。

HTTPS 凭据配置可能使 `~/.gitconfig` 在首次 apply 时出现冲突；先查看 diff，再选择备份覆盖，非交互时可核对后显式使用 `apply --force`。部署后的 macOS Git 配置仍启用 `osxkeychain`，凭据留在 Keychain 中。

初始化时核对 Git 姓名、邮箱和 Homebrew prefix（Apple Silicon 默认 `/opt/homebrew`，Intel 默认 `/usr/local`）。查看 `diff` 后再部署；完整 HOME 部署会安装软件和修改仓库声明的 macOS 设置。`sudo -v` 在本机准备管理员权限，stool 本身仍以当前用户运行。`apply` 依次检查或安装 Homebrew、安装 Brewfile（含 GPG Suite）、部署文件、隐藏提示 GPG 备份密码并恢复个人/Work 密钥，最后设置 macOS 偏好；受口令保护的原私钥可能另有 pinentry 提示。当前密码备份无需 agekey。

下载期间管理员权限可能过期；重新 `sudo -v` 后再运行 `stool dotfile apply`。若安装器仍需交互，可先在 Terminal 安装 Homebrew 或执行 `brew install --cask gpg-suite`，再重试。失败阶段不记录成功，已完成阶段保留。仓库认证失败应先修复 SSH/PAT 权限并重跑 `git ls-remote`；GPG 密码错误只需重跑 `apply`，不用重新初始化。

部署检查通过后，可核验两把私钥并各做一次签名。使用 Suite 绝对路径，避免 PATH 中其他 `gpg`；这里生成的只是临时消息和公开签名，不导出密钥：

```bash
/usr/local/MacGPG2/bin/gpg --list-secret-keys --keyid-format long --fingerprint \
  2BF609784E75AAC78CB67A7E465799B005224003 98A56F40C2A502230AC6635435DF2F117D3233C5
stool_gpg_check_dir=$(mktemp -d "${TMPDIR:-/tmp}/stool-gpg-check.XXXXXX")
printf 'stool GPG restore check\n' > "$stool_gpg_check_dir/message.txt"
for stool_gpg_key in 2BF609784E75AAC78CB67A7E465799B005224003 98A56F40C2A502230AC6635435DF2F117D3233C5; do
  /usr/local/MacGPG2/bin/gpg --local-user "$stool_gpg_key" --armor --detach-sign \
    --output "$stool_gpg_check_dir/$stool_gpg_key.asc" "$stool_gpg_check_dir/message.txt" &&
  /usr/local/MacGPG2/bin/gpg --verify "$stool_gpg_check_dir/$stool_gpg_key.asc" "$stool_gpg_check_dir/message.txt"
done
rm -r -- "$stool_gpg_check_dir"
```

两次验签都应显示对应个人/Work 身份的 `Good signature`。`stool dotfile verify` 检查文件计划与已记录的恢复摘要，不检查之后手动删掉的 keyring 条目；实际签名确认私钥可用。日常同步运行 `stool dotfile update`；HTTPS 凭据过期先重新预缓存，选择 SSH 时若新 Terminal 未加载密钥则重新执行 `ssh-add`。

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

`init` 在克隆前检查系统 Git 是否可用。缺失时停止并显示当前系统的安装命令：macOS 使用 `xcode-select --install` 安装包含 Git 的 Command Line Tools，Windows 提示 Git for Windows / winget，Linux 提示发行版包管理器。安装完成后运行 `git --version`，再重试原命令；检查失败不会创建 source、保存初始化配置或部署文件。macOS 的 `/usr/bin/git` 占位程序不会被当作已安装的 Git，也不会自动弹出安装窗口。普通仓库使用 `--builtin` 时无需这个检查；`--builtin --recurse-submodules` 仍需要系统 Git。仓库中的 Homebrew / 软件包脚本在后续 apply 阶段执行，不能替代首次克隆前的 Git 安装。

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

普通 dotfile 内容由 `add --encrypt` 导入为密文，`edit` 编辑后重新加密，`apply` 解密并部署。age 使用本机 `[encryption]` 中的 `recipients` 和 `identities` 配置，支持 X25519、多接收方及 armored 密文；既有口令型 source 文件的 `STOOL_AGE_PASSPHRASE` 兼容方式保留。GPG 密钥备份的 age 密码模式和外部密码文件另见下节，不依赖已移除的独立子命令。

普通 GPG 加密文件使用本机 `gpg`，在本机 `[encryption]` 中配置 `backend='gpg'` 和 `gpg_recipient`。`encrypted_` 文件支持 `.age` / `.asc`，可叠加 `.tmpl`。加密编辑仅在私有临时目录中解密，编辑成功后重新加密。

模板秘密函数有 `onepassword`、`bitwarden`、`pass`、`gopass`、`vault`、`keyring`、`keepassxc`、`lastpass`、`doppler`、`aws`、`azure`，另有通用 `secret(provider, ...args)`。示例 `{{ onepassword('op://vault/item/password') }}`、`{{ bitwarden('item-id').login.password }}`。各服务复用已认证的官方 CLI；同一进程内相同请求只调用一次，不将秘密缓存落盘。Bitwarden 可使用 `BW_SESSION`，或以 `BW_PASSWORD` 解锁一次；KeePassXC 从 `STOOL_KEEPASS_PASSWORD` 读取口令。服务失败时隐藏其输出。

脚本支持 `run_`、`run_once_`、`run_onchange_`，可叠加 `before_` / `after_` 和 `.tmpl`。before 阶段先执行，普通脚本与文件按解码目标路径穿插执行，after 阶段最后执行。只在成功后记录 once/onchange 状态；失败可重试，已完成文件和成功脚本保留。onchange 记录最近成功内容，A→B→A 会重新执行。提供 `STOOL_SOURCE_DIR` / `STOOL_DEST_DIR` / `STOOL_HOME` / `STOOL_OS` / `STOOL_ARCH`，以及当前运行程序绝对路径 `STOOL_EXECUTABLE`。

GPG 密钥备份与恢复使用下述原生命令和声明配置，无需仓库中的备份/恢复脚本。旧钩子中的独立加解密调用应按待发布兼容性说明迁移；普通 dotfile 的加密由导入、编辑和部署流程处理。

预览时 stool 不执行 run hooks、不写目标、部署状态或持久缓存；渲染模板仍会调用秘密提供器，`modify_` 转换脚本会在私有临时工作目录中执行。这些外部程序没有被沙箱限制，仍可能有自己的副作用。部署和预览都应使用可信仓库。diff 和 archive 包含渲染后的明文，请按目标文件的敏感程度保存。

## GPG 密钥备份与恢复

在已有私钥的机器上，显式选择完整主密钥指纹；子密钥会随主密钥一起导出：

```bash
stool dotfile gpg backup --fingerprint PERSONAL_PRIMARY_FINGERPRINT --fingerprint WORK_PRIMARY_FINGERPRINT
stool dotfile gpg restore
```

备份默认在本机控制终端隐藏输入并确认密码，使用 age 密码加密私钥和 ownertrust。密码保存在密码管理器或离线介质中；GPG 原私钥口令保持原样，系统 pinentry 可能另行要求解锁。`--passphrase-file` 可从仓库外的私有文件提供密码，备份也可从管道描述符读取。密码不会进入参数、环境变量、日志或部署状态，私钥明文只保存在内存和管道中。

默认备份目录为 source 中的 `.stooltemplates/gpg/`，保存公钥、`secret.asc.age`、`ownertrust.txt.age` 和公开清单 `backup-metadata.sh`。stool 将该历史清单作为严格的字段数据解析，不执行其中的 Shell。现有单密钥、多密钥的密码备份及独立 age 身份文件备份均可直接恢复，无需重新加密。`backup --identity /external/private/identity.agekey` 使用已有的独立 age 身份文件；恢复时先从单独备份恢复身份文件。

`--fingerprint` 可重复；备份会拒绝遗漏既有清单中的密钥。恢复前校验 SHA-256、公私钥的完整主指纹集合和 ownertrust，缺少私钥、混入未选密钥、密码错误或数据损坏都会在真实导入之前失败。只导出明确选择的密钥，不复制整个 `~/.gnupg`。`--gpg-program` 可指定 GPG 可执行文件；macOS 优先使用已安装的 GPG Suite。

仓库可用 `.stoolgpg.toml` 或 `.stoolgpg.toml.tmpl` 声明自动恢复（两者只能存在一个）：

```toml
enabled = {{ restore_gpg | default(true) | tojson }}
directory = ".stooltemplates/gpg"
program = {{ gpg_program | default('gpg') | tojson }}
identity = {{ gpg_identity | default('') | tojson }}
passphrase_file = {{ gpg_passphrase_file | default('') | tojson }}
```

配置仅包含路径和开关，密码文件必须在仓库外。`apply` / `init --apply` / `update` 依次执行 before 钩子、部署文件、原生 GPG 恢复和 after 钩子；因此可先安装 GPG Suite，再恢复密钥，最后执行系统设置。恢复到 HOME 的 `~/.gnupg`，独立 destination 跳过自动恢复。只有完整成功后才记录恢复摘要；失败可重试，重复部署不重复导入，备份变化后会重新恢复。

对于原生 GPG 恢复计划，`diff`、`status`、`apply --dry-run` 只显示待恢复项，不询问备份密码或运行 GPG；普通加密文件仍可能需要解密。`verify` 会报告尚未成功恢复的备份。GPG 声明和默认 `.stooltemplates/gpg/` 控制目录中的支持文件不会作为普通 dotfile 部署或进入 archive；自定义备份目录需另行放在控制目录或明确排除。显式 `gpg restore` 可再次导入备份。只同步普通配置时，仓库可将 `enabled` 绑定到 `restore_gpg` 数据，并使用 `--data restore_gpg=false`。

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
