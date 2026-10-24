# stool dotfile 实现说明

> 更新日期：2026-10-24。本文描述当前源码的 Rust 实现、状态与安全边界，离线密钥、macOS 偏好与 Homebrew 命令待发布；历史 xyrev v0.5.0 的发行包和行为保持原样。

产品能力见 [概览](dotfile-overview.md)，协议与新机器流程见 [设计文档](dotfile-design.md)，命令参数见 [README](../README.md) 和 `stool dotfile --help`。

## 1. 模块与数据流

CLI 入口为 `src/cli/mod.rs`。简单命令使用单文件（`init.rs`），命令组使用 `agent/`、仅 macOS 的 `apple/`、`dotfile/`、`key/` 目录。组内 `mod.rs` 定义子命令和统一导出，具体参数按用途拆分；dotfile 的导入/编辑参数共用 `maintenance.rs`，key 的类别选择由 `selection.rs` 共用。CLI 只定义命令和参数；执行逻辑位于 `src/init.rs`、`src/agent/`、`src/apple/`、`src/dotfile/`、`src/key/`，由 `main.rs` 分发。业务模块仍通过 `cli::dotfile` / `cli::key` 导入参数类型。

`src/apple/preferences.rs` 独立读取和设置当前用户的 Dock 偏好，通过 `/usr/bin/defaults` 参数调用执行。仅识别退出码 1 与确切 domain/key 的缺失诊断为未设置；其他读取错误会停止。写入后重新读取验证，预览不写入；不加载 dotfile 配置、锁或成功数据库，也不自动重启 Dock。

`src/cli/apple/brew/` 定义 init/install、prefix、dry-run 与非交互参数；`src/apple/homebrew.rs` 负责发现、前置、私有临时安装器和安装后校验，`packages.rs` 先读取 Brewfile 再确保 Homebrew 可用并显式安装，`process.rs` 管理子进程输入和退出状态。Homebrew/包操作不使用 dotfile 的 Paths、锁或 once/onchange 数据库；预览不下载或执行清单 Ruby。模拟执行器覆盖完整初始化失败链，CLI 测试使用隔离的假 brew，不运行本机安装器。

当前 dotfile 代码位于同一 Rust crate 中：

| 职责 | 模块 |
|---|---|
| clap 参数与子命令 | [src/cli/dotfile/mod.rs](../src/cli/dotfile/mod.rs) |
| 命令编排、锁和分发 | [mod.rs](../src/dotfile/mod.rs) |
| 配置、路径、权限和状态 | [storage.rs](../src/dotfile/storage.rs) |
| source 属性及控制项扫描 | [source.rs](../src/dotfile/source.rs) |
| MiniJinja、数据与初始化提示 | [template.rs](../src/dotfile/template.rs) |
| 计划、冲突与应用顺序 | [plan.rs](../src/dotfile/plan.rs) |
| 脚本幂等与子进程 | [scripts.rs](../src/dotfile/scripts.rs)、[process.rs](../src/dotfile/process.rs) |
| 普通文件加密 | [crypto.rs](../src/dotfile/crypto.rs) |
| 独立离线密钥 | [src/key/](../src/key/mod.rs)、[CLI](../src/cli/key/mod.rs) |
| Git、gix 与前置检查 | [git.rs](../src/dotfile/git.rs)、[native_git.rs](../src/dotfile/native_git.rs)、[prerequisites.rs](../src/dotfile/prerequisites.rs) |
| 秘密、外部资源与归档 | [secrets.rs](../src/dotfile/secrets.rs)、[external.rs](../src/dotfile/external.rs)、[archive.rs](../src/dotfile/archive.rs) |
| 导入/编辑、辅助命令与交互 | [maintenance.rs](../src/dotfile/maintenance.rs)、[tools.rs](../src/dotfile/tools.rs)、[ui.rs](../src/dotfile/ui.rs) |

```text
Git source + 本机配置 + --data
               │
        控制项、属性、模板与资源求值
               ▼
         期望条目与脚本
               │ 实际磁盘状态 + 上次成功基线
               ▼
              Plan
       ┌───────┼────────┐
       ▼       ▼        ▼
      预览    apply    archive
```

模块共用扫描结果和 Plan；当前文件访问使用 Rust 标准库及封装函数，没有要求先建立额外 Vfs 装饰器层或多 crate workspace。

## 2. source 协议与模板

解析器逐目录分量处理 stool 路径属性，校验类型和顺序，构造文件、目录、链接、删除或脚本条目。`dot_` 解码点文件名，`private_`、`readonly_`、`executable_` 表达权限，`encrypted_` 和 `.tmpl` 触发内容处理。`create_`、`modify_`、`remove_`、`exact_` 和 `external_` 有各自的适用对象；完整集合见 [设计协议](dotfile-design.md#21-路径和控制文件)。

当前 develop 的控制扫描只读取 `.stool*`，初始化配置入口为 `.stool.toml.tmpl`，模板指令入口为 `stool:template:`。旧命名不作为公开兼容入口自动加载，运行目录及环境变量使用 `~/.stool` 与 `STOOL_*`。旧环境重新初始化，source 命名、本机配置和资源引用仍需核对。

MiniJinja 引擎提供严格缺失变量、受控 include、编码/换行指令及数据合并。覆盖顺序为仓库数据、本机 `[data]`、本次 `--data`；`stool` 命名空间保留。初始化提示只在初始化配置阶段开放，非终端提示要求默认值。模板与资源求值可能涉及外部程序或网络，应把 source 视为可信输入。

## 3. Plan 与冲突

`Plan::build` 比较期望条目、实际磁盘快照和上次成功基线。内容、类型、权限与链接目标共同决定操作；实际状态已经等于期望时直接收敛，目标在部署后被用户修改且与期望不同才需要冲突处理。

计划阶段检查重复/大小写碰撞目标、source/control 重叠、路径越界和已有链接父目录。默认先汇总冲突；终端可确认备份覆盖，非交互默认失败。`--force` 针对目标覆盖，不清理 source 的 Git 修改。

应用顺序为：

1. before 钩子。
2. 文件与 during 脚本，按解码目标路径组织。
3. after 钩子。

文件在同目录写入临时文件、设权限后替换；覆盖和显式删除前保存备份。每个成功项及时记账，中途失败保留已完成项，重试重新比较实际状态。数据库写入和目标文件变更不是跨文件整体事务。

## 4. 状态与并发

默认控制目录为 `~/.stool/dotfile/`：

| 存储 | 内容 |
|---|---|
| `config.toml` | 来源 URL、分支、目录、Git 选项、本机数据及工具配置 |
| `state/<namespace>.json` | 文件/目录/链接基线 |
| `state/scripts.redb` | once/onchange 的成功状态 |
| `backups/` | 覆盖前内容、权限与操作元数据 |
| `cache/` | 部署时持久保存的外部资源缓存 |

namespace 根据 source/destination 配对生成，防止隔离试运行与真实 HOME 共用基线。写操作持有命令锁；JSON 使用私有权限和原子替换，脚本预览读取 redb 临时快照，避免写打开原数据库。

`run_once_` 按成功内容摘要跳过，`run_onchange_` 保留最近成功内容，所以 A→B→A 会再次执行。失败不记录成功，阶段顺序和幂等条件分别处理。

### 4.1 旧环境重新初始化

程序只加载新命名下的本机配置与状态，不包含旧状态迁移模块。升级时使用新协议 source 重新 `init`，新 namespace 的基线和成功记录从实际部署建立，不继承旧 once/onchange 成功记录。旧控制目录保持原样；本机配置和仓库外凭据路径需重新设置，部署前检查待执行脚本及冲突。旧 GPG 格式和自动恢复接口不兼容；先创建并验证新的离线包。操作步骤见 [设计文档](dotfile-design.md#14-旧控制目录迁移)。

## 5. 普通文件加密与秘密

内部 age 支持 X25519、多接收方、armor 和密码处理；普通 GPG 加解密使用本机子进程管道。普通密文通过 `add --encrypt` 导入、`edit` 修改后重加密、`apply` 解密部署。交互编辑为编辑器创建私有临时工作文件，离线密钥备份则只以内存和管道处理中间明文。

当前 develop 移除了独立 `encrypt` / `decrypt` CLI；内部实现、`keygen`、普通文件流程仍保留；密钥改由独立 `stool key` 处理。完整兼容性变更见 [设计文档](dotfile-design.md#13-本版兼容性变更)。

秘密提供器调用已认证的 CLI 或系统存储，并在同一进程内复用相同请求结果。秘密缓存不持久化，失败输出受隐藏处理；服务自己的认证与副作用仍由服务及其工具负责。

## 6. 独立离线密钥生命周期

`src/cli/key/mod.rs` 定义 `stool key backup/restore`，`src/key/mod.rs` 负责路径、密码、单文件容器、清单和恢复编排，`gpg.rs` / `ssh.rs` 分别导出与校验。默认全部类别，显式开关互斥；不加载 dotfile 配置、Plan、锁或状态。参数与包格式详见 [离线密钥说明](key-backup.md)。

备份枚举全部可导出 GPG 组和 `.ssh` 真实私钥，检查完整主/子指纹、稳定清单及公私钥配对，保存对应 ownertrust 和已有撤销证书。缺失、硬件材料和 agent-only SSH 私钥有明确警告。私钥保留原口令；不导出整份数据库或运行 SSH 配置中的命令。

内存中构造 tar，JSON 清单包含版本、文件摘要及材料标记；整体由 age 密码加密，公钥和清单也不公开。自检解密后使用不覆盖的方式保存新文件。密码在终端隐藏输入并确认，外部密码文件要求私有权限且在 Git 外，备份另支持继承管道；没有密码参数或环境变量入口。中间明文缓冲以 SecretVec/SecretString 等控制，不写临时明文包。

恢复完全解密、检查认证及清单后，先预检 SSH/撤销证书冲突和现有 GPG keyring 的链接；在隔离临时 keyring 进行 show-only 结构校验，再导入目标并检查所有原本可导出的私钥组件。临时 keyring 不导入私钥。GPG 2.2 在空 keyring 中可能把 show-only 的私钥标为缺失，因此此阶段比对完整指纹，最终以导入后的材料清单判断是否成功。

GPG 导入成功后才写 SSH 和撤销证书，使用最终文件名 create_new、先收紧权限再写内容，无中间明文私钥文件。已有相同内容跳过并收紧权限，不同内容不覆盖。GPG 导入或文件写入失败可能留下部分完成项，不承诺事务回滚；修复后可重试。密钥目录 0700、文件 0600，Windows 复用受限 DACL helper。

key 命令不会上传密钥、启动 SSH agent 或写部署成功记录。旧四文件格式、身份文件模式、`dotfile gpg`、自动恢复声明及其状态字段均已移除；普通加密文件不受影响。

## 7. 预览、归档与外部操作

`diff`、`status` 和 `apply --dry-run` 不执行部署 run 钩子，不写目标或成功状态，也不持久更新资源缓存。为了计算目标，它们仍可能调用秘密服务、解密普通密文、执行 `modify_` 转换或临时下载资源；外部程序并不运行在沙箱中。

所有 dotfile 命令都不处理离线密钥包。`managed` 不解密普通 encrypted 条目，但数据和规则模板仍会加载。diff 和 archive 可能包含目标明文，应按目标数据的敏感程度处理。

archive 消费同一 Plan 中的目标内容、权限和链接，不执行 run 钩子。资源提取校验路径、重复项、父链接和文件类型，避免写出预定边界；SHA-256 校验负责内容一致性，不能替代对来源的信任。

## 8. Git 与新机器依赖

系统 Git 子进程关闭终端凭据提示，所以私有仓库应先在普通终端通过 credential helper 或 SSH 预检完成认证。gix 可处理普通仓库；SSH 仍需要 SSH 客户端，子模块/LFS 使用系统工具。缺少 Git 或 macOS Command Line Tools 会停止并给出安装指引。

update 先检查未提交、未跟踪和子模块修改，只允许快进。部署冲突不会回滚已拉取 source，用户修复目标冲突后重试 apply。

新机器的离线包转入、SSH/HTTPS 登录、管理员权限和 GPG 手动恢复步骤统一见 [设计文档](dotfile-design.md#3-新-mac从公开安装到私有仓库恢复)。不能依赖后续 apply 安装的软件完成首次私库认证。

## 9. 验证与长期结构

Rust/Python 夹具覆盖临时 Git/HTTP、属性与模板、冲突与失败重试、外部资源、模拟秘密 CLI、内部加密及一次性 GPG keyring。macOS/Linux/Windows 与最低 Rust 版本按 CI 矩阵验证；模拟服务测试不等于所有真实账户都已经认证。

当前使用 PathBuf、运行时校验、anyhow 上下文和同步求值。多 crate、Vfs、类型化路径/错误、并发和统一状态存储仅属于长期可选重构，详见 [长期规划](dotfile-design.md#7-长期规划与未采用方案)。功能和状态变更应同时更新测试、CLI 文档及迁移说明。
