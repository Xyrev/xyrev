# stool dotfile 产品概览

> 更新日期：2026-10-24。描述当前源码，包含待发布的离线密钥、macOS 偏好与 Homebrew 命令；旧 tag 和发行包行为不变。

`stool dotfile` 从 Git URL 获取配置仓库，按 stool 路径协议和 MiniJinja 模板计算目标文件，预览后部署到用户目录，并保护后续本地修改。`stool key` 独立处理仓库外密钥备份，不依赖 dotfiles。

使用见 [README](../README.md)，协议与流程见 [设计文档](dotfile-design.md)，代码和状态见 [实现说明](dotfile-internals.md)，密钥格式与恢复见 [离线密钥说明](key-backup.md)。

## 工作方式与能力

| 对象 | 作用 |
|---|---|
| source | Git 工作树，保存配置、模板、脚本与资源声明 |
| target / destination | 计算后的期望条目 / 实际部署位置，默认 HOME |
| 成功基线 | 上次成功写入状态，用于发现本地修改与冲突 |
| Plan | 汇总待变更文件和脚本，供预览、部署、归档复用 |

`dot_gitconfig.tmpl` 渲染为 `~/.gitconfig`；`private_dot_ssh/private_config` 部署权限受限的 SSH 配置。普通文件是独立目标文件，只有显式 `symlink_` 才创建链接。

| 能力 | 当前行为 |
|---|---|
| Git | 完整 HTTPS/SSH/file URL、分支、子模块、系统 Git 与可选 gix；更新仅快进 |
| 文件 | 权限、模板、加密、创建/转换/删除、exact、literal、external 子树 |
| 模板 | MiniJinja、严格缺失变量、include、初始化提示、嵌套数据、编码和换行指令 |
| 维护 | diff/status/managed/apply、add/edit/unmanaged、verify/doctor/data/execute-template/cd |
| 脚本 | run/once/onchange、before/during/after、成功记账、失败重试 |
| 加密与秘密 | 普通配置的内部 age/GPG、加密导入/编辑/部署、已认证秘密服务适配 |
| macOS 与 Homebrew | 独立 apple init 与 apple brew init/install，按需设置偏好并显式安装清单 |
| 离线密钥 | `stool key backup/restore`，默认全部 GPG/SSH、密码加密单文件、手动恢复 |
| 资源与归档 | SHA-256、TTL 缓存、过滤和路径检查；tar/tgz/zip |
| 冲突 | 三方比较、冲突预检、覆盖前备份、单文件原子替换、部分成功记录 |

控制命名为 `.stool*`，包括 ignore/data/templates/scripts/external/root/version 和 `.stool.toml.tmpl`；模板空间为 `stool.*`，指令为 `stool:template:`。

## 兼容边界

旧控制命名、本机状态迁移及独立 encrypt/decrypt CLI 不提供。普通 `add --encrypt`、`edit`、`apply`、内部加密和 `dotfile keygen` 保留。

旧 GPG 四文件格式、age 身份文件备份、`stool dotfile gpg` 和自动恢复声明已删除，不做兼容。先在旧机器建立并验证新 tar.age 包，再清理 source。离线包的公钥、清单和秘密均一起加密；dotfiles 仅保留配置和签名指纹。不自动删除旧控制目录、现有密钥或改写 Git 历史。

## 新机器流程

安装包含 `stool key` 的构建 → 手动转入离线包 → 恢复 SSH（已有 GPG 可恢复全部）→ 完成 Git SSH 或 HTTPS 认证 → init/diff/apply/verify → 如需要再手动恢复 GPG。macOS 在配置部署后显式运行 `stool apple brew install` 安装 ~/.Brewfile（含 GPG Suite），再恢复 GPG。完整命令见 [新 Mac 流程](dotfile-design.md#3-新-mac从公开安装到私有仓库恢复)。

macOS 偏好由 `stool apple init --dry-run` 预览、`stool apple init` 应用，安装当前构建后即可独立运行；不再由示例 dotfiles 的 apply/update 设置。具体行为见 [README](../README.md#macos-偏好初始化)。

安装 binary 不提供私库授权；SSH 公钥仍需 GitHub 账号授权，HTTPS 仍需提前通过 helper 缓存凭据。所有 dotfile 命令均不恢复 GPG/SSH 密钥或询问离线备份密码，密钥操作无需先 init。

## 运行和安全边界

- `private_` 控制权限，不加密 source。离线密钥包和密码文件拒绝放在 Git 工作区；SSH-only 不需要 Git/GPG，GPG 操作从 PATH 调用 `gpg`。
- 预览不执行 run 钩子或写目标/成功状态，但秘密请求、普通文件解密、modify 转换和临时下载仍可能发生。可信 source 是前提，diff/archive 可能含明文。
- `dotfile verify` 只检查配置；密钥可用性需通过实际签名、解密或 SSH 配对验证。
- 多文件部署、GPG 导入及最终密钥写入不提供整批回滚；完成项保留，修复后重试。
- 硬件秘密和 agent-only SSH 私钥不可导出；具体范围、权限和密码规则见离线密钥说明。

## 长期规划

目前单 crate、共享 Plan、JSON 文件基线和 redb 脚本状态，密钥命令无部署成功状态。workspace、Vfs、类型化路径、并发求值、统一状态存储及原生 GitHub 登录均为待评估方向，不属于当前已实现能力。每项扩展应明确迁移、依赖和可复现验证条件。
