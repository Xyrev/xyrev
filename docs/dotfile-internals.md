# stool dotfile 实现说明

> 更新日期：2026-10-24。本文描述 stool v0.6.0 的 Rust 开发版本、状态与安全边界；本版本尚未发布，历史 xyrev v0.5.0 的发行包和行为保持原样。

产品能力见 [概览](dotfile-overview.md)，协议与新机器流程见 [设计文档](dotfile-design.md)，命令参数见 [README](../README.md) 和 `stool dotfile --help`。

## 1. 模块与数据流

当前 dotfile 代码位于同一 Rust crate 中：

| 职责 | 模块 |
|---|---|
| clap 参数与子命令 | [src/cli/dotfile/mod.rs](../src/cli/dotfile/mod.rs) |
| 命令编排、锁和分发 | [mod.rs](../src/dotfile/mod.rs) |
| 显式旧状态迁移与回退 | [migrate.rs](../src/dotfile/migrate.rs) |
| 配置、路径、权限和状态 | [storage.rs](../src/dotfile/storage.rs) |
| source 属性及控制项扫描 | [source.rs](../src/dotfile/source.rs) |
| MiniJinja、数据与初始化提示 | [template.rs](../src/dotfile/template.rs) |
| 计划、冲突与应用顺序 | [plan.rs](../src/dotfile/plan.rs) |
| 脚本幂等与子进程 | [scripts.rs](../src/dotfile/scripts.rs)、[process.rs](../src/dotfile/process.rs) |
| 普通文件加密与 GPG 密钥管理 | [crypto.rs](../src/dotfile/crypto.rs)、[gpg.rs](../src/dotfile/gpg.rs) |
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

当前 develop 的控制扫描只读取 `.stool*`，初始化配置入口为 `.stool.toml.tmpl`，模板指令入口为 `stool:template:`。旧命名不作为公开兼容入口自动加载，运行目录及环境变量使用 `~/.stool` 与 `STOOL_*`。旧环境通过显式迁移流程处理，source 改名和引用仍需核对。

MiniJinja 引擎提供严格缺失变量、受控 include、编码/换行指令及数据合并。覆盖顺序为仓库数据、本机 `[data]`、本次 `--data`；`stool` 命名空间保留。初始化提示只在初始化配置阶段开放，非终端提示要求默认值。模板与资源求值可能涉及外部程序或网络，应把 source 视为可信输入。

## 3. Plan 与冲突

`Plan::build` 比较期望条目、实际磁盘快照和上次成功基线。内容、类型、权限与链接目标共同决定操作；实际状态已经等于期望时直接收敛，目标在部署后被用户修改且与期望不同才需要冲突处理。

计划阶段检查重复/大小写碰撞目标、source/control 重叠、路径越界和已有链接父目录。默认先汇总冲突；终端可确认备份覆盖，非交互默认失败。`--force` 针对目标覆盖，不清理 source 的 Git 修改。

应用顺序为：

1. before 钩子。
2. 文件与 during 脚本，按解码目标路径组织。
3. 原生 GPG 恢复计划。
4. after 钩子。

文件在同目录写入临时文件、设权限后替换；覆盖和显式删除前保存备份。每个成功项及时记账，中途失败保留已完成项，重试重新比较实际状态。数据库写入和目标文件变更不是跨文件整体事务。

## 4. 状态与并发

默认控制目录为 `~/.stool/dotfile/`：

| 存储 | 内容 |
|---|---|
| `config.toml` | 来源 URL、分支、目录、Git 选项、本机数据及工具配置 |
| `state/<namespace>.json` | 文件/目录/链接基线及最近成功 GPG 恢复摘要 |
| `state/scripts.redb` | once/onchange 的成功状态 |
| `backups/` | 覆盖前内容、权限与操作元数据 |
| `cache/` | 部署时持久保存的外部资源缓存 |

namespace 根据 source/destination 配对生成，防止隔离试运行与真实 HOME 共用基线。写操作持有命令锁；JSON 使用私有权限和原子替换，脚本预览读取 redb 临时快照，避免写打开原数据库。

`run_once_` 按成功内容摘要跳过，`run_onchange_` 保留最近成功内容，所以 A→B→A 会再次执行。失败不记录成功，阶段顺序和幂等条件分别处理。

### 4.1 显式迁移

`stool dotfile migrate` 在普通命令配置加载前单独分发，从旧控制目录读取保存路径，默认目标为 `~/.stool/dotfile`，拒绝 source/destination/data 覆盖。用户先把 source 准备为新协议，迁移自身不联网、不修改 tracked 内容、不部署目标。

迁移通过暂存和原子发布建立新控制根并保留旧根，任何已存在的目标均停止，同一收据识别出的已完成迁移除外。状态映射只继承可证明纯改名等价的基线或成功记录；未知 namespace 保留，不能证明等价的 hooks/GPG 保持 pending。业务目录（例如 `~/.config/xyrev`）不会自动改名；涉及这类路径变化的脚本必须重新执行，不能只按品牌词替换继承成功。dry-run 不写入、不调用 hooks/GPG/秘密服务，不求值 env()/prompt 模板函数。使用方法见 [设计文档](dotfile-design.md#14-旧控制目录迁移)。

## 5. 普通文件加密与秘密

内部 age 支持 X25519、多接收方、armor 和密码处理；普通 GPG 加解密使用本机子进程管道。普通密文通过 `add --encrypt` 导入、`edit` 修改后重加密、`apply` 解密部署。交互编辑为编辑器创建私有临时工作文件，这与原生 GPG 密钥备份的明文处理边界不同。

当前 develop 移除了独立 `encrypt` / `decrypt` CLI；内部实现、`keygen`、普通文件流程和 GPG 密钥命令仍保留。完整待发布变更见 [设计文档](dotfile-design.md#13-待发布兼容性变更)。

秘密提供器调用已认证的 CLI 或系统存储，并在同一进程内复用相同请求结果。秘密缓存不持久化，失败输出受隐藏处理；服务自己的认证与副作用仍由服务及其工具负责。

## 6. 原生 GPG 生命周期

备份默认枚举 GPG secret keyring，按完整主指纹收集含有可导出本地主密钥或子密钥的组，导出可用私钥、对应公钥和 ownertrust；只含公钥的条目不纳入，过期或撤销的本地密钥仍保留。离线主密钥配有本机私有子密钥时也会纳入；缺失或硬件私钥只能保留公开部分，提示其私钥无法从备份恢复。完全没有本地可导出私钥的组跳过并提示；空集合在读取密码或改写备份前失败。

可选 `--fingerprint` 按完整主指纹筛选，拒绝不存在或完全不可导出的显式选择。覆盖前检查既有清单及 `public.asc`，拒绝遗漏旧主/子指纹；若既有组当前含不可导出组件，旧格式缺少原私钥可用性记录，无法证明旧私钥材料不丢失，因此保守拒绝。部分私钥组可首次备份到新目录，重复备份时需保留原目录并使用全新 `--output`，不读取旧密码或合并旧密文。新输出通过校验后写入暂存目录，再发布备份数据和清单。

| 文件 | 内容 |
|---|---|
| `public.asc` | 公钥与公开身份 |
| `secret.asc.age` | 私钥导出的 age 密文 |
| `ownertrust.txt.age` | 所选密钥的信任数据密文 |
| `backup-metadata.sh` | 模式、完整主指纹和摘要；严格解析的数据，不执行 Shell |

`.stoolgpg.toml[.tmpl]` 决定是否恢复、支持目录和 GPG/凭据路径。计划准备只读公开清单与密文摘要；执行时才读取备份密码或独立 age 身份文件，校验摘要、公私钥主指纹集合与 ownertrust 后导入 HOME keyring。

密码和私钥明文只在受控内存及进程管道中处理，不写入 argv、环境变量、日志、普通状态或明文临时文件。密码文件需在仓库外、为私有普通文件且不是链接；备份另支持继承的管道描述符。GPG 原私钥口令由 GPG/pinentry 处理。

完整恢复成功后才保存公开摘要；相同备份与配置不重复导入，A→B→A 会再恢复。显式 `gpg restore` 可重新导入。失败可能已导入部分数据，不承诺 keyring 回滚；四份备份数据发布也不是跨文件事务，中断会由后续一致性校验发现。

默认 `.stooltemplates/gpg/` 位于控制目录，不作为普通目标部署或归档；自定义备份目录应放在控制目录或显式忽略。独立 destination 跳过自动恢复。`verify` 检查记录和计划，不执行签名或保证 keyring 未被事后修改。

## 7. 预览、归档与外部操作

`diff`、`status` 和 `apply --dry-run` 不执行部署 run 钩子，不写目标或成功状态，也不持久更新资源缓存。为了计算目标，它们仍可能调用秘密服务、解密普通密文、执行 `modify_` 转换或临时下载资源；外部程序并不运行在沙箱中。

原生 GPG 恢复预览不启动 GPG、不询问备份密码，此约束只针对该恢复计划。`managed` 不解密普通 encrypted 条目或准备 GPG 恢复，但数据和规则模板仍会加载。diff 和 archive 可能包含目标明文，应按目标数据的敏感程度处理。

archive 消费同一 Plan 中的目标内容、权限和链接，不执行 run 钩子。资源提取校验路径、重复项、父链接和文件类型，避免写出预定边界；SHA-256 校验负责内容一致性，不能替代对来源的信任。

## 8. Git 与新机器依赖

系统 Git 子进程关闭终端凭据提示，所以私有仓库应先在普通终端通过 credential helper 或 SSH 预检完成认证。gix 可处理普通仓库；SSH 仍需要 SSH 客户端，子模块/LFS 使用系统工具。缺少 Git 或 macOS Command Line Tools 会停止并给出安装指引。

update 先检查未提交、未跟踪和子模块修改，只允许快进。部署冲突不会回滚已拉取 source，用户修复目标冲突后重试 apply。

新机器的 HTTPS 登录、公开安装器、管理员权限、GPG 恢复和双密钥验签步骤统一见 [设计文档](dotfile-design.md#3-新-mac从公开安装到私有仓库恢复)。不能依赖后续 apply 安装的软件完成首次私库认证。

## 9. 验证与长期结构

Rust/Python 夹具覆盖临时 Git/HTTP、属性与模板、冲突与失败重试、外部资源、模拟秘密 CLI、内部加密及一次性 GPG keyring。macOS/Linux/Windows 与最低 Rust 版本按 CI 矩阵验证；模拟服务测试不等于所有真实账户都已经认证。

当前使用 PathBuf、运行时校验、anyhow 上下文和同步求值。多 crate、Vfs、类型化路径/错误、并发和统一状态存储仅属于长期可选重构，详见 [长期规划](dotfile-design.md#7-长期规划与未采用方案)。功能和状态变更应同时更新测试、CLI 文档及迁移说明。
