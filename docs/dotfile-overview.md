# stool dotfile 产品概览

> 更新日期：2026-10-23。本文描述 stool v0.6.0 开发版本，尚未发布。历史 xyrev v0.5.0 的 tag、安装脚本、资源及二进制行为保持原样。

`stool dotfile` 从 Git URL 获取配置仓库，按 stool 路径协议和 MiniJinja 模板计算目标文件，预览差异后部署到用户目录，并在后续更新中保护本地修改。

使用步骤和配置详见 [README](../README.md)；完整协议、新机器流程与长期规划见 [设计文档](dotfile-design.md)；Rust 模块及状态处理见 [实现说明](dotfile-internals.md)。

## 1. 工作方式

| 对象 | 作用 |
|---|---|
| source | Git 工作树，保存配置、模板、密文、脚本与资源声明 |
| target | 根据本机数据计算出的期望内容、权限和链接 |
| destination | 实际部署位置，默认为当前用户 HOME |
| 成功基线 | 上次实际成功写入的状态，用于识别本地修改与冲突 |
| Plan | 汇总待变更文件、脚本与 GPG 恢复步骤，供预览和部署使用 |

配置来源与目标路径通过文件名和目录属性关联，例如 `dot_gitconfig.tmpl` 渲染为 `~/.gitconfig`，`private_dot_ssh/private_config` 部署为权限受限的 SSH 配置。普通文件部署为独立目标文件；显式 `symlink_` 才创建链接。

## 2. 当前能力

| 能力 | 当前行为 |
|---|---|
| Git 来源 | 完整 HTTPS、SSH、绝对 `file://` URL；分支、子模块、系统 Git 与可选 gix 后端 |
| 初始化与更新 | init 保存来源和本机配置；update 检查工作树、仅允许快进，再重新部署 |
| 路径协议 | 文件/目录权限、模板、加密、创建、转换、删除、exact、literal 与 external 子树 |
| 模板 | MiniJinja、严格缺失变量、include、初始化提示、嵌套数据覆盖、编码与换行指令 |
| 维护 | 显式 migrate、diff/status/managed/apply、add/edit/unmanaged、verify/doctor/data/execute-template/cd |
| 脚本 | run/once/onchange、before/during/after；成功后记账、失败重试、内容变化后重跑 |
| 密文与秘密 | 内置 age、本机 GPG；加密导入/编辑/部署，以及 README 列出的秘密服务适配 |
| GPG 密钥管理 | 显式选择多把主密钥、密码或独立 age 身份保护、完整性校验、原生恢复与成功摘要 |
| 外部资源 | 文件、归档、归档内文件和 Git 资源；过滤、SHA-256、TTL 缓存及路径保护 |
| 冲突与备份 | 三方比较、全计划冲突预检、覆盖前备份、单文件原子替换和部分成功记录 |
| 归档与交互 | tar/tgz/zip 导出、终端冲突确认、由 CLI 定义生成补全和 man pages |

控制项统一使用 `.stool*`，包括 `.stoolignore`、`.stooldata.*`、`.stool.toml.tmpl`、`.stooltemplates/`、`.stoolexternal.*`、`.stoolroot`、`.stoolversion` 和 `.stoolgpg.toml[.tmpl]`。模板数据使用 `stool.*` 命名空间，模板指令使用 `stool:template:`。

## 3. 当前 develop 的待发布变更

当前 develop 仅识别 stool 控制命名、`STOOL_*` 环境变量和 `~/.stool` 默认目录，不自动读取旧命名；升级旧环境应走[显式迁移流程](dotfile-design.md#14-旧控制目录迁移)，并核对 `.stool*`、`stool.*` 与 `stool:template:` 及资源引用。migrate 先检查已准备的新协议 source，不联网、不改 tracked source，也不自动 apply；保留旧控制目录供回退，只继承能证明等价的成功记录。工具不会隐式修改远端仓库。

独立 `stool dotfile encrypt` / `stool dotfile decrypt` 子命令已移除。普通 dotfile 继续通过 `add --encrypt`、`edit` 和 `apply` 管理，GPG 密钥使用 `gpg backup` / `gpg restore` 及原生恢复声明；其他文件加解密由相应工具处理。内部 age/GPG 能力、既有密文、备份格式和 `keygen` 保留。

改名及这些接口调整属于尚未发布的 v0.6.0；历史 xyrev 发行包和 tag 不变。迁移细节见 [设计文档](dotfile-design.md#13-待发布兼容性变更)。

## 4. 新机器流程

`szwathub/stool` 仓库公开，v0.6.0 发行包尚未发布；`szwathub/dotfiles` 是私有仓库。新机器先构建 stool（发布后可直接下载安装包），准备 Git/Command Line Tools，通过凭据助手完成 HTTPS 认证，再初始化、预览和部署；已有 SSH 认证可使用 SSH URL。完整步骤见 [新 Mac 流程](dotfile-design.md#3-新-mac从公开安装到私有仓库恢复)。

GitHub 凭据用于下载仓库，原 GPG 备份密码用于一次恢复个人和 Work 两把主密钥，原 GPG 私钥口令用于签名等操作。首次 Git 登录不能依赖尚未下载的备份；恢复既有备份无需重新加密。

## 5. 运行与安全边界

- 默认 Git、SSH、子模块、LFS、GPG 和秘密提供器各有相应工具或认证要求；安装一个 binary 不会自动取得私有仓库权限。
- `private_` 控制文件权限，不加密 source。密码、登录私钥与 age 身份文件应按各自功能的要求存放；原生 GPG 密码不进入 argv、环境变量或普通状态。
- 预览不执行部署 run 钩子或写入目标/成功状态；模板秘密请求、普通文件解密、modify 转换和临时下载仍可能发生。应使用可信仓库，diff 和 archive 可能包含明文。
- 原生 GPG 恢复预览不启动 GPG、不询问备份密码；真实恢复成功后才保存摘要。`verify` 检查计划和记录，不代替实际签名或 keyring 检查。
- 多文件部署与 GPG 导入不提供整体事务回滚；失败保留已完成项和备份，修复后重试。
- macOS/Linux/Windows 有实现及 CI 验证，符号链接、ACL、解释器等行为按平台使用条件判断。第三方真实账户认证仍由用户管理。

## 6. 后续规划

当前采用单 crate、共享 Plan、JSON 文件/GPG 状态和 redb 脚本状态。workspace 拆分、Vfs 层、类型化路径、并发求值、统一状态存储和 GitHub 登录引导都属于可独立评估的长期方向；它们没有被当作已经发布的能力。每项扩展应先明确状态迁移、运行依赖和可复现的测试条件。
