# chezmoi 技术研究与 xyrev 实现映射

> 本轮整理：2026-10-23；xyrev 已发布对照版本：v0.5.0，另记录当前 develop 的后续未发布 CLI 差异。本文保留上游 chezmoi 的架构研究，用于解释设计来源；Go 类型、`internal/chezmoi` 和 `internal/cmd` 路径均属于 `twpayne/chezmoi`，不是本仓库 Rust 路径。
>
> 本文不提供未固定版本的源码行号或规模断言。上游链接指向官方源码/文档，后续可能变化；伪代码只表达控制关系，省略参数、错误处理和版本差异，不是逐字摘录，也不是可直接编译的当前源码。xyrev 的实际能力以 [实现设计](chezmoi-rust-implementation.md)、[README](README.md) 和 CLI 帮助为准。

## 1. 三层状态与惰性求值

上游将源描述、期望结果和实际磁盘状态分开。`SourceStateEntry` 表达来源及求值入口，`TargetStateEntry` 提供应用和跳过判定，actual entry 描述当前磁盘；文件、目录、脚本、链接及删除操作分别建模。源码入口为 [sourcestateentry.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/sourcestateentry.go) 和 [targetstateentry.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/targetstateentry.go)。

```text
source entry
    │ 数据、模板、解密、外部资源
    ▼
target entry ── 与 actual / 上次成功状态比较 ──▶ 应用选定变更
```

保留求值函数和内容摘要，可以推迟读取、渲染或解密，并复用结果。上次成功状态为本地编辑检测提供基线；它是部署记录，不是所有文件系统操作都能自动回滚的事务。

以下 Go 风格伪代码仅用于说明边界；其中函数名是示意，不是上游 API：

```go
wanted := evaluate(source, machineData)
found := inspect(destination)
changes := compare(wanted, found, lastSuccess)
applySelected(changes, system)
recordSuccessful(changes, persistentState)
```

实际错误路径必须只记录成功项。上面的简化顺序不代表发生中途错误后可以把整个计划标记成功。

## 2. 路径属性解析

上游 [attr.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/attr.go) 区分文件类型、脚本条件和阶段，按适用对象解析有序属性。不能任意打乱或组合前缀后假定含义相同。规范以 [source state attributes](https://www.chezmoi.io/reference/source-state-attributes/) 为准。

| 属性类别 | 典型名称 | 关注点 |
|---|---|---|
| 类型 | `create_`、`modify_`、`remove_`、`symlink_`、`run_` | 决定目标是文件、链接、删除还是动作 |
| 内容处理 | `encrypted_`、`.tmpl` | 决定是否解密、渲染 |
| 权限与空文件 | `private_`、`readonly_`、`executable_`、`empty_` | 影响权限与空内容的语义 |
| 路径转义 | `dot_`、`literal_`、`.literal` | 解码目标名称或保留字面量 |
| 目录行为 | `exact_`、`external_` 等 | 涉及目录成员及外部来源 |
| 脚本行为 | `once_`、`onchange_`、`before_`、`after_` | 条件和执行阶段分别编码 |

路径属性把映射和行为放进 source 名称；它不排斥数据、外部资源或恢复策略等控制文件。`private_` 表达访问权限，并非密文标记。具体权限还要考虑文件/目录类型、umask 和操作系统，不能把一种 Unix 位运算直接当作 Windows 实现。

xyrev 采用这套命名思路，但由自己的解析器验证类型与顺序；支持集合见实现设计，不以本表推导任意上游组合都兼容。

## 3. 模板、数据和指令

chezmoi 使用 Go `text/template` 与 Sprig，内置数据位于 `.chezmoi`；用户数据、配置数据和模板片段共同参与渲染。官方说明见 [templating](https://www.chezmoi.io/user-guide/templating/)。

[template.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/template.go) 对模板执行增加指令处理、数据复制及编码/换行处理。指令在模板解析前处理，避免改变分隔符后再无法识别指令本身；复制数据用于避免函数修改共享数据。具体函数表与机器变量由上游命令层组合，阅读入口为 [config.go](https://github.com/twpayne/chezmoi/blob/master/internal/cmd/config.go)。

xyrev 使用 MiniJinja，并保留严格缺失变量、include、初始化提示、数据覆盖、编码和换行等能力；这不是 Go Template 的解释器。典型迁移关系为：

| 上游写法 | xyrev 写法或边界 |
|---|---|
| `{{ .chezmoi.os }}` | `{{ xyrev.os }}` |
| Go 的 `if` / `end` 模板动作 | Jinja 的 `{% if ... %}` / `{% endif %}` |
| `.chezmoi.homeDir` 等变量 | 使用 README 列出的 `xyrev.home_dir` 等字段，不假定字段逐项等价 |
| Sprig 和上游辅助函数 | 改用 Jinja 或 xyrev 明确提供的函数，不自动迁移 |
| `.chezmoitemplates`、部分控制文件别名 | 支持相应迁移入口，不代表其中的 Go 模板可直接运行 |

当前 Rust 入口为 [template.rs](src/dotfile/template.rs)。

## 4. 加密边界与明文处理

上游通过加密抽象分离内容处理与路径解析。官方 [加密文档](https://www.chezmoi.io/user-guide/encryption/)区分 age、gpg 等用法；[透明加密](https://www.chezmoi.io/user-guide/encryption/transparent/)由 transcrypt/git-crypt 等外部 Git 工具参与，不能据此断言所有后端都内置。

上游 [gpgencryption.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/gpgencryption.go) 中的 GPG 加解密流程会在私有临时目录组织输入/输出文件，再执行配置的 gpg。这里描述的是该上游实现选择，**不是 GPG 不能使用 stdin/stdout 的限制**。

当前 develop 已在 v0.5.0 之后移除独立 `xyrev dotfile encrypt` / `xyrev dotfile decrypt` 子命令，尚未发布新版本；已发布 v0.5.0 仍保留当时接口。旧脚本应迁移到 dotfile 导入/编辑/部署或原生 GPG 密钥命令，其他文件加解密改用相应工具。该变更不删除内部加密，也不改变本节的上游研究，详见 [待发布兼容性说明](chezmoi-rust-implementation.md#13-待发布兼容性变更)。

xyrev 当前实现应分别理解：

- [crypto.rs](src/dotfile/crypto.rs) 内置 age，供 `add --encrypt`、`edit` 和 `apply` 等流程调用；普通 GPG 加解密使用子进程管道，不照搬上述临时文件流程。
- [gpg.rs](src/dotfile/gpg.rs) 提供 GPG 密钥备份和恢复。私钥与 ownertrust 的明文在内存和 GPG 管道间流转，密码不写入参数、环境变量、日志或状态。
- 已加密文件的 `edit` 为编辑器提供私有临时工作文件；这是交互编辑功能，不能与 GPG 密钥备份/恢复的明文处理承诺混为一谈。

v0.5.0 原生命令支持显式选择多把主密钥、密码加密、独立 age 身份文件、校验、导入及成功状态。`.xyrevgpg.toml[.tmpl]` 声明恢复策略，现有密码密文无需重新加密；历史 `backup-metadata.sh` 仅作字段数据解析，不能作为 Shell 执行。仓库无需另外维护备份/恢复脚本。

## 5. 脚本幂等与阶段

上游 [TargetStateScript](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/targetstateentry.go) 使用脚本内容摘要和持久状态决定是否跳过。条件与阶段是两组独立信息：

| 条件 | 状态判定思路 |
|---|---|
| `run_` | 每次应用执行 |
| `run_once_` | 已成功执行过的内容摘要可跳过；改动内容会形成新的 once 任务 |
| `run_onchange_` | 比较该脚本最近成功内容；内容变化后再次执行 |

before、普通阶段和 after 决定脚本与文件的相对顺序；“失败也算执行过”会破坏重试语义。上游执行顺序另见 [application order](https://www.chezmoi.io/reference/application-order/)。

xyrev 的 [scripts.rs](src/dotfile/scripts.rs) 用 redb 记录成功执行；onchange 保存最近成功版本，因此 A→B→A 会再次运行。脚本命名中的数字只是同阶段内的排序约定。原生 GPG 恢复在 [Plan](src/dotfile/plan.rs) 中作为单独步骤：before 钩子和目标文件完成后恢复，再执行 after 钩子。普通阶段脚本仍按目标路径与文件穿插处理。

## 6. System 装饰器的研究价值

上游 [system.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/system.go) 抽象文件操作及命令/脚本执行；相应实现可以承担真实写入、预览、日志或归档工作。阅读时应区分共同接口与各命令的具体组装，不应把某一条装饰器栈写成所有版本的固定调用链。

| 上游实现类别 | 目的 |
|---|---|
| RealSystem | 实际访问目标系统 |
| DryRunSystem / ReadOnlySystem | 抑制或拒绝相应写操作 |
| GitDiffSystem / DebugSystem | 记录差异或调试信息 |
| TarWriterSystem / ZipWriterSystem | 将目标内容输出为归档 |
| ArchiveReaderSystem | 让归档内容参与读取 |

这些抽象有利于重用状态转换和测试，但不代表所有副作用都消失，或加入一种用途完全没有工程成本。模板秘密函数、外部下载和命令调用仍需逐项审查。

xyrev 当前通过 [plan.rs](src/dotfile/plan.rs) 的共享 `Plan` 计算变化，再由预览、应用和归档路径消费；项目没有名为 `RealVfs`、`DryRunVfs` 或 `TarWriterVfs` 的完整 Rust 装饰器层。这些名称只属于长期结构草图。

## 7. 秘密服务适配

上游命令层注册提供器函数。以 [bitwardentemplatefuncs.go](https://github.com/twpayne/chezmoi/blob/master/internal/cmd/bitwardentemplatefuncs.go) 为例，它会处理会话或解锁、调用配置的 CLI、解码结果，并按参数缓存进程内结果。这个例子说明“提供器适配器”的组织方式，不保证其他服务采用完全相同的登录、缓存或错误处理规则。

xyrev 的 [secrets.rs](src/dotfile/secrets.rs) 实现 README 列出的 1Password、Bitwarden、pass、gopass、Vault、Keyring、KeePassXC、LastPass、Doppler、AWS 和 Azure 等适配。同一进程中复用相同请求的结果，不把秘密缓存持久化；用户仍需准备对应 CLI 和认证。

预览不会运行 run hooks 或修改部署状态，但模板求值仍可能调用秘密服务，普通加密文件仍可能需要解密，`modify_` 也会运行转换程序。原生 GPG 备份预览不启动 GPG、不询问恢复密码，是另一个明确约束；不能把它推广成所有模板和内容处理均不执行外部程序。

## 8. init / bootstrap 的范围

上游 [`init` 文档](https://www.chezmoi.io/reference/commands/init/)定义克隆或建立 source、生成配置、可选 apply，以及 one-shot、purge、URL 猜测等参数；命令层入口为 [initcmd.go](https://github.com/twpayne/chezmoi/blob/master/internal/cmd/initcmd.go)。这些是上游接口，不是 xyrev 隐含支持的选项。

xyrev 则要求完整 Git URL，默认系统 Git，可显式选择 gix。SSH 仍需 SSH 客户端；子模块/LFS 会调用系统 Git 等工具。缺失 Git/Command Line Tools 时会给出安装引导，不会凭空生成账户认证，也不应把“内置 Git”解读成任何私有仓库都可匿名初始化。

新机器先安装公开 xyrev，准备 Git 并预先通过凭据助手或 SSH 完成私有仓库认证，然后 init、diff、权限准备、apply 和 GPG 恢复。系统软件安装发生在后续 apply，不能用它作为首次 clone 的认证前置。完整且统一的 HTTPS 主流程及 SSH 替代方案见 [新 Mac：从公开安装到私有仓库恢复](chezmoi-rust-implementation.md#3-新-mac从公开安装到私有仓库恢复)。

当前 `szwathub/dotfiles` 为私有仓库，xyrev 发行包仍公开。Git 登录凭据、原 GPG 备份密码和各私钥原口令分别管理；现有个人与 Work 密钥通过一次备份密码输入恢复。

## 9. bbolt 研究与 xyrev 状态格式

上游 [persistentstate.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/persistentstate.go) 定义条目、脚本、外部 Git 资源及配置状态等逻辑 bucket。[boltpersistentstate.go](https://github.com/twpayne/chezmoi/blob/master/internal/chezmoi/boltpersistentstate.go) 提供 bbolt 存储、只读/读写打开和事务访问；读取时复制事务中的字节数据，避免事务结束后继续引用底层内存。不存在的数据库需要按空状态处理，不能让只读预览为了初始化数据库而写入。

这里可借鉴的是状态职责、只读边界和成功记账，而不是必须沿用同一种数据库或 bucket 格式。数据库事务也不能替代目标文件系统的跨文件事务。

xyrev 当前采用：

| 内容 | 存储与职责 |
|---|---|
| 文件、链接、目录的上次成功基线 | `~/.xyrev/dotfile/state/<source-destination-key>.json`，由 [storage.rs](src/dotfile/storage.rs) 原子保存 |
| 最近成功 GPG 恢复摘要 | 同一 JSON 状态中的公开摘要，按 source/destination 隔离 |
| run_once/run_onchange 成功状态 | `state/scripts.redb`，由 [scripts.rs](src/dotfile/scripts.rs) 管理 |
| 并发与部分成功 | 命令锁保护；部署失败保留已完成项和覆盖前备份，不承诺多文件整体回滚 |

因此“全部 EntryState 已进入 redb”或“直接复用上游 bbolt”都不是当前实现事实。

## 10. 当前 Rust 模块映射

| 职责 | xyrev 当前源码 |
|---|---|
| CLI 参数与子命令 | [src/cli/dotfile/mod.rs](src/cli/dotfile/mod.rs) |
| 命令编排与配置/状态入口 | [mod.rs](src/dotfile/mod.rs)、[storage.rs](src/dotfile/storage.rs) |
| 属性、source 扫描、模板与数据 | [source.rs](src/dotfile/source.rs)、[template.rs](src/dotfile/template.rs) |
| 统一计划、冲突、文件/GPG/脚本顺序 | [plan.rs](src/dotfile/plan.rs) |
| 脚本状态与子进程 | [scripts.rs](src/dotfile/scripts.rs)、[process.rs](src/dotfile/process.rs) |
| 普通加密与原生 GPG 密钥管理 | [crypto.rs](src/dotfile/crypto.rs)、[gpg.rs](src/dotfile/gpg.rs) |
| Git 与缺失工具引导 | [git.rs](src/dotfile/git.rs)、[native_git.rs](src/dotfile/native_git.rs)、[prerequisites.rs](src/dotfile/prerequisites.rs) |
| 秘密、外部资源与归档 | [secrets.rs](src/dotfile/secrets.rs)、[external.rs](src/dotfile/external.rs)、[archive.rs](src/dotfile/archive.rs) |
| 日常维护、诊断与交互 | [maintenance.rs](src/dotfile/maintenance.rs)、[tools.rs](src/dotfile/tools.rs)、[ui.rs](src/dotfile/ui.rs) |

这份映射描述 v0.5.0 的单 crate 布局。将通用类型、模板、状态或文件系统抽象拆成多个 crate，以及加入更完整的 Vfs 装饰器，均可作为长期可选重构；当前没有要求先建立 `xyrev-dotfile-*` workspace 才算实现功能。

## 11. 兼容和验收边界

- 已采用：Git 仓库来源、路径属性、三层比较、模板/秘密、脚本幂等、外部资源、预览、归档与状态记录等思路；xyrev 原生 GPG 密钥管理还提供自己的声明与兼容校验。
- 需要迁移：Go Template/Sprig、变量命名、部分控制字段、CLI 参数和状态数据。采用相同文件名前缀不等于无修改运行整个上游仓库。
- 当前未支持：完整上游函数/命令集、用户名式 URL 猜测、one-shot/purge、透明 Git 加密后端，以及 README 未列出的提供器和选项。
- 验证依据：项目测试与 macOS/Linux/Windows CI，而不是研究文档中的源码数量或“零依赖”推断；真实账户登录、SSH/PAT、私有 LFS 和第三方秘密服务仍需自己的认证条件。

阅读上游时，先由属性和 source/target 接口理解数据流，再追踪模板、System、持久状态和具体命令。阅读当前 xyrev 时，从本节 Rust 映射和实现设计进入，避免把上游 Go 文件路径或教学伪代码当成已经存在的 Rust 架构。
