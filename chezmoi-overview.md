# chezmoi 概览与 xyrev 兼容对照

> 本轮整理：2026-10-23；xyrev 对照版本：v0.5.0。本文是上游 `twpayne/chezmoi` 的研究概览，以及本仓库已采用能力的对照，不是完整复刻 chezmoi 的需求清单。上游资料链接可能随版本变化，不以未固定版本的 stars、代码行数或功能数量判断实现范围。
>
> 当前 xyrev 的协议、验收与新机器操作以 [实现设计](chezmoi-rust-implementation.md)、[README](README.md) 和 `xyrev dotfile --help` 为准。更详细的上游技术分析及 Rust 模块映射见 [internals](chezmoi-internals.md)。

## 1. 上游项目定位

[chezmoi](https://github.com/twpayne/chezmoi) 是以 Go 编写、采用 MIT 许可证的 dotfiles 管理工具。它用 source state 表达配置来源，按机器数据计算期望状态，再与实际目标比较并部署。它适合维护不同机器、操作系统或个人/工作身份之间既共享又存在差异的配置。参见 [上游项目说明](https://github.com/twpayne/chezmoi#readme)与[官方概念文档](https://www.chezmoi.io/reference/concepts/)。

本文保留 `chezmoi`、`.chezmoi*` 和上游 Go 路径，是为了准确描述研究对象。xyrev 的命令入口是 `xyrev dotfile`，运行配置和模板数据使用 xyrev 命名。

## 2. 核心模型

| 概念 | 上游含义 | 对 xyrev 的启发 |
|---|---|---|
| Source state | 维护的文件、模板、密文、脚本和外部资源声明 | Git 仓库作为可追踪的配置来源 |
| Target state | 根据数据、模板与解密结果计算出的期望状态 | 先生成统一计划，再决定目标变更 |
| Actual state | destination 中现存的文件、权限及链接 | 比较实际状态、期望状态和上次成功基线 |
| Attributes | `dot_`、`private_`、`encrypted_`、`run_`、`.tmpl` 等路径属性 | 路径编码映射与行为，避免逐文件目标清单 |

这种模型使配置内容与部署动作可以分别检查。文件权限属性与加密是不同能力：`private_` 限制目标权限，不会加密仓库内容。属性定义参见 [chezmoi source state attributes](https://www.chezmoi.io/reference/source-state-attributes/)。

## 3. 上游能力与依赖

| 上游能力 | 研究要点 |
|---|---|
| 模板 | 使用 Go `text/template` 和 Sprig，并提供机器数据及辅助函数；参见[模板文档](https://www.chezmoi.io/user-guide/templating/) |
| 加密 | 提供 age、gpg 等配置；git-crypt/transcrypt 属于外部透明 Git 加密工具的集成，不应笼统写成内置加密实现；参见[加密](https://www.chezmoi.io/user-guide/encryption/)及[透明加密](https://www.chezmoi.io/user-guide/encryption/transparent/) |
| 秘密服务 | 模板可调用 1Password、Bitwarden、pass 等服务；各提供器有自己的安装和登录要求；参见[密码管理器](https://www.chezmoi.io/user-guide/password-managers/) |
| 脚本 | `run_`、`run_once_`、`run_onchange_` 表达执行条件，before/after 表达阶段；参见[脚本](https://www.chezmoi.io/user-guide/use-scripts-to-perform-actions/) |
| 外部资源 | 用声明取得文件、归档或其他来源；参见[外部文件](https://www.chezmoi.io/user-guide/include-files-from-elsewhere/) |
| 初始化 | 支持配置模板、仓库 URL 猜测、可选 apply 和 one-shot 等上游参数；参见[`init`](https://www.chezmoi.io/reference/commands/init/) |

单个可执行文件便于分发，但不能据此承诺任何仓库都没有运行时依赖。SSH 传输、GPG、密码管理器、子模块、LFS 和仓库自带的安装脚本，都可能需要外部程序、账户权限或管理员交互。内置 Git/age 后端的存在也不提供私有仓库凭据。

## 4. xyrev v0.5.0 兼容对照

以下“已实现”描述本仓库能力，不表示上游相同名称的每个参数、模板函数和存储格式均兼容。

| 能力 | xyrev 当前状态 | 兼容边界 |
|---|---|---|
| Git URL 初始化与更新 | 已实现 HTTPS、SSH、绝对 `file://` URL、分支、子模块、快进更新与工作树保护 | 要求完整 URL；不提供上游用户名猜测、one-shot 或 purge 参数 |
| 路径和控制文件 | 已实现主要路径属性、ignore/remove/data/templates/scripts/external、root/version 与初始化配置 | 首选 `.xyrev*`；接受文档列出的 `.chezmoi*` 别名，不代表未知控制文件或任意属性组合均受支持 |
| 预览、部署和维护 | 已实现 diff/status/managed/apply/update、add/edit/unmanaged、verify/doctor/data/cd 等 | 命令参数以 xyrev 帮助为准，不接受任意上游命令的直接替换 |
| 模板 | 已实现 MiniJinja、include、本机数据、初始化提示与编码/换行指令 | Go Template/Sprig 需要迁移；`{{ .chezmoi.os }}` 改为 `{{ xyrev.os }}`，不是自动转换 |
| 脚本与状态 | 已实现 once/onchange、阶段顺序、成功后记录及失败重试 | 文件/GPG 基线为 JSON，脚本状态为 redb；不能直接读取上游 bbolt 状态 |
| 加密文件与秘密服务 | 已实现 age、GPG 子进程、加密导入/编辑和 README 列出的服务适配 | 需要相应密钥或已认证的外部 CLI；不提供 git-crypt/transcrypt 透明加密模式，也未覆盖全部上游提供器及函数 |
| 外部资源与归档 | 已实现文件、归档、归档内文件、Git 资源、过滤、校验、缓存和 tar/tgz/zip 导出 | 按 xyrev 声明格式使用，不假定上游所有资源选项都等价 |
| 冲突与平台 | 已实现三方比较、备份覆盖、原子文件替换、终端冲突确认及平台权限处理 | macOS/Linux/Windows 由项目测试和 CI 验证；链接、ACL、解释器等差异仍按平台说明使用 |
| GPG 密钥备份与恢复 | v0.5.0 已提供原生命令及 `.xyrevgpg.toml[.tmpl]` 声明，兼容既有密码加密备份 | 这是 xyrev 的通用能力；不能用上游普通 `encrypted_` 文件流程替代选钥、完整性校验和恢复状态管理 |

“已实现”与“完全兼容”需要分别判断：路径命名可以沿用，模板语法和配置字段仍可能需要迁移。具体支持集合、CLI 和测试范围见 [实现设计](chezmoi-rust-implementation.md)及 [README](README.md)。

明确未支持的范围包括直接执行 Go Template、完整 Sprig/chezmoi 函数兼容、透明 Git 加密后端、上游用户名式 init 和 one-shot/purge，以及未列出的上游命令或提供器。它们没有因研究文档出现而自动成为当前产品承诺。

workspace 多 crate 拆分、统一 Vfs 装饰器层等属于长期可选结构；当前保持单 crate 和共享 `Plan`。结构重构不等于补齐用户功能，也不应把草图名称写成已存在的 Rust 模块。

## 5. 本仓库的新机器流程

`Xyrev/xyrev` 的程序与发行包公开，`szwathub/dotfiles` 为私有仓库，现有 GPG 密文仍保存在该仓库内。新机器先安装公开 xyrev，准备 Git/Command Line Tools，并完成 Git 仓库认证，再初始化、预览和部署。当前推荐 HTTPS 凭据预缓存流程及 SSH 替代方案统一见 [新 Mac：从公开安装到私有仓库恢复](chezmoi-rust-implementation.md#3-新-mac从公开安装到私有仓库恢复)。

GitHub 认证、GPG 备份密码、GPG 私钥原口令各有用途：认证先取得仓库，原备份密码再恢复个人和 Work 两把主密钥，实际签名仍可能需要原私钥口令。无需为恢复现有备份重新加密，也不能依赖尚未克隆的备份完成首次 Git 登录。

README 和实现设计维护可执行步骤，包括 HTTPS/PAT 的本机凭据助手、SSH 登录密钥、子模块、管理员权限、预览、恢复失败重试及双密钥验签。本文不另维护一套重复的安装命令。

## 6. 上游阅读入口

- [项目仓库](https://github.com/twpayne/chezmoi)：入口、许可证和源码。
- [官方概念](https://www.chezmoi.io/reference/concepts/)与[属性协议](https://www.chezmoi.io/reference/source-state-attributes/)：理解 source/target 与命名规则。
- [模板](https://www.chezmoi.io/user-guide/templating/)、[加密](https://www.chezmoi.io/user-guide/encryption/)和[`init`](https://www.chezmoi.io/reference/commands/init/)：核对特定版本的实际语义。
- [源码技术分析](chezmoi-internals.md)：上游实现思路与 xyrev 当前模块之间的对应关系。
