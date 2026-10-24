# GPG 与 SSH 离线密钥备份

本文描述当前待发布源码。`stool key` 独立于 dotfiles，不读取 dotfile 配置、部署状态或 Git 仓库；安装包含本命令的 binary 后即可手动转入并恢复离线包。

## 命令与默认值

```bash
# 在仓库外的目录运行；默认备份全部可导出 GPG 和本地 SSH 私钥
mkdir -p "$HOME/Backups/stool"
cd "$HOME/Backups/stool"
stool key backup

# 显式文件路径，不覆盖已有文件
stool key backup --output "$HOME/Backups/stool/my-keys.tar.age"

# 恢复包内全部类别到当前 HOME
stool key restore --input /Volumes/Backup/my-keys.tar.age

# 可分步恢复；SSH 操作无需 Git 或 GPG
stool key restore --ssh --input /Volumes/Backup/my-keys.tar.age
stool key restore --gpg --input /Volumes/Backup/my-keys.tar.age
```

| 参数 | backup | restore |
|---|---|---|
| 无选择开关 / `--all` | 所有可备份类别 | 包内所有类别 |
| `--gpg` / `--ssh` | 只备份指定类别 | 只恢复指定类别 |
| `--input` | 可选源用户目录；默认当前 HOME，默认环境下的 GPG 尊重 GNUPGHOME | 必须指定加密包文件 |
| `--output` | 可选新文件；默认当前目录的 `stool-keys-<Unix 时间戳>.tar.age` | 可选目标用户目录；默认 HOME |
| `--passphrase-file` | 仓库外的 0600 密码文件；另支持继承的管道描述符 | 仓库外的 0600 密码文件 |

三个选择开关互斥，无开关等于 `--all`。显式类别没有密钥时失败；默认全部只要至少一类非空即可备份，同时提示缺少类别。全部恢复按包内类别执行；新机尚未安装 GPG 时，使用 `--ssh` 提前恢复登录密钥。显式 `--input /path/to/user` 从该目录的 `.gnupg` 和 `.ssh` 读取，覆盖环境中的 GNUPGHOME；恢复的 GPG 目录始终为目标用户目录下的 `.gnupg`。

## 包内内容和覆盖范围

全部内容放在一个密码加密的二进制 age 容器中，文件名后缀为 `.tar.age`。内部 tar 包含版本化 JSON 清单、完整指纹及子密钥材料标记、文件 SHA-256、警告和以下数据；公钥与清单也加密，不另留公开副本。

| 类别 | 内容 |
|---|---|
| GPG | 全部可导出的本地主/子私钥、对应公钥、这些主密钥的 ownertrust 和已有撤销证书 |
| SSH | `.ssh` 内递归发现的真实私钥文件，不限 `id_*` 名称；配对公钥保留原字节，缺失时提取生成 |

GPG 只含公钥的组不纳入；过期、撤销但仍在本地的私钥继续备份，以支持历史数据。离线主密钥下的本地私有子密钥也纳入，缺失或智能卡中的秘密无法导出，清单和终端会提示；完全不可导出的组跳过。GPG 导出前后清单变化、部分公私钥导出、子密钥遗漏均拒绝生成备份。

SSH 保留原私钥字节、文件名及原口令，不复制 `config`、`known_hosts`、socket 或 agent 状态，不跟随符号链接。只存在于 agent、硬件或 `.ssh` 之外的私钥无法由本包恢复；当前用户设置了 SSH_AUTH_SOCK 时会检查 agent 公钥并提示未纳入的身份，硬件 key handle 会提示仍需原硬件。显式源目录不借用当前用户 agent。外部 IdentityFile 和 config Include 不递归追踪，应自行保留对应离线备份。传统 PEM 私钥通过 PATH 中的 `ssh-keygen` 提取公钥；OpenSSH 格式可直接提取嵌入公钥，无需先解开原私钥口令。

不复制整份 `.gnupg` 数据库，也不备份 GPG agent/pinentry 配置、操作系统钥匙串或第三方凭据。dotfiles 只保留 GPG/SSH 配置和 Git 签名指纹及 URL 身份规则。

## 密码、权限与恢复安全

备份隐藏输入密码并确认，恢复隐藏输入一次。将备份密码保存在密码管理器或独立离线介质；原 GPG/SSH 私钥口令不改变，也不能代替外层备份密码。错误密码或损坏数据先失败，不创建目标密钥。

新包校验成功后才以不覆盖方式保存。备份输出、恢复目标和密码文件拒绝位于 Git 工作区；拒绝路径穿越、符号链接、特殊归档项、重复或大小写冲突及文件/目录碰撞。包大小受 64 MiB 上限约束。恢复先校验 age 认证、清单、所有文件摘要、公私钥指纹与 SSH 配对，再预检所有文件冲突及 GPG keyring 中的链接。已有同内容 SSH/撤销证书跳过并收紧权限；不同内容不覆盖且先于 GPG 导入报错。

目录使用 0700、密钥与包使用 0600；Windows 使用当前用户、SYSTEM 和管理员的受限 DACL。密码和私钥明文在受控内存及进程管道中处理，不进入 argv、环境变量、日志或部署状态。程序不写中间明文 tar、导出私钥临时文件；恢复时的最终 `.ssh` 和 `.gnupg` 内容是用户主动要求的落盘目标。GPG、agent 和系统自身的缓存/交换区行为由系统管理。

GPG 始终通过 PATH 中的 `gpg` 调用。恢复先在隔离 keyring 检查完整公私钥包结构，再导入目标，导入后检查所有原本可导出的私钥组件。GPG 导入没有事务回滚；中途失败可能保留部分已导入内容，修复原因后重跑恢复。GPG 失败时不写 SSH。成功后写 SSH 和撤销证书时遇到 IO 错误也可能保留部分已完成文件，重试会接受相同内容。恢复不自动启动 SSH agent、上传公钥或赋予 GitHub 访问权。

这里只支持新的密码加密离线格式；旧四文件 GPG 备份、独立 age 身份加密包、旧 `stool dotfile gpg` 和自动恢复声明均不兼容。先在旧机器完成并验证新包，再移除仓库备份。已有 dotfile 配置中不再使用的 restore_gpg/gpg_identity/gpg_passphrase_file/gpg_program 数据可手动删除；不会自动删除外部身份文件、现有密钥或改写 Git 历史。

## 新机器与验收

推荐顺序：安装包含 `stool key` 的版本 → 手动转入离线包 → 恢复 SSH（已有 GPG 可直接恢复全部）→ Git 认证 → 初始化、预览及部署 dotfiles → 若此前只恢复 SSH，再手动恢复 GPG。完整 macOS 命令见 [README](../README.md#新-mac-初始化私有-dotfiles)。Linux/Windows 先按本机方式安装 GPG；这份 dotfiles 的 Homebrew 钩子仅在 macOS HOME 部署运行。

除查看指纹外，可用隔离目录对两把 GPG 分别做签名和验签：

```bash
stool_check_dir=$(mktemp -d)
printf 'stool offline restore check\n' > "$stool_check_dir/message.txt"
for stool_check_key in 2BF609784E75AAC78CB67A7E465799B005224003 98A56F40C2A502230AC6635435DF2F117D3233C5; do
  gpg --local-user "$stool_check_key" --armor --detach-sign \
    --output "$stool_check_dir/$stool_check_key.asc" "$stool_check_dir/message.txt" &&
  gpg --verify "$stool_check_dir/$stool_check_key.asc" "$stool_check_dir/message.txt"
done
```

两次应显示对应身份的 Good signature。SSH 可对每把恢复的文件执行 `ssh-keygen -y -f <private-file>` 与配对 `.pub` 比较；这些命令只输出公钥，不导出私钥。自动测试仅使用一次性合成密钥，分别验证签名、解密、ownertrust、撤销证书、SSH 原始字节和原口令。

仓库当前树删除旧加密包不会撤回 Git 历史中的已下载副本；本次不改写历史或自动轮换身份。离线包和密码应分开保存，并另留一份异地副本。
