# xyrev dotfile：基于 Git URL 与 chezmoi 命名规则的设计

> 已确认需求（2026-10-22）：在现有 **xyrev** 中通过 Git URL 配置 dotfiles，仓库迁移为 **chezmoi 风格命名规则**。命令入口为 `xyrev dotfile`。§16 的功能项现已接入实现，验收范围与平台差异见该节及 README。
>
> 本文基于 `chezmoi-overview.md` / `chezmoi-internals.md`。本文统一使用 `xyrev` 与 `dotfile` 模块命名；第 1 节的 workspace 是长期拆分参考。当前命令、配置和使用示例以 README 与 `xyrev dotfile --help` 为准，协议与验收以 §0.1–§0.4、§12、§16 为准，Rust 片段是接口草图，不是已经可编译的实现。

---

## 0. 设计原则

沿用 chezmoi 已被验证的架构，但用 Rust 的惯用法重写：

| chezmoi（Go） | xyrev dotfile（Rust） | 理由 |
|---|---|---|
| 接口 + 装饰器 | `trait` + 泛型 / `dyn Trait` | Rust trait 是一等公民，零成本抽象 |
| `error` 链 | `thiserror` + `anyhow`、`Result<T, DotfileError>` | 类型化错误 + 源码位置 |
| `go-vfs` 文件系统抽象 | 自定义 `Vfs` trait（可用 `vfs` crate） | 便于 dry-run / archive / mock |
| bbolt（B+Tree KV） | `redb`（纯 Rust，ACID，mmap） | 单文件、纯 Rust、零依赖 |
| Cobra | `clap` v4 derive | 生态标配 |
| Bubble Tea | `ratatui` + `crossterm` | Rust TUI 事实标准 |
| `text/template` + sprig | `minijinja`（Jinja 风格语法） | 需要明确模板迁移边界，不能直接执行 Go Template |
| `filippo.io/age`（Go 原生 age） | `age` crate（官方 Rust 实现） | 一等公民支持 |
| `copystructure.Copy` | `serde_json::Value::clone()` 或自定义深拷贝 | 模板数据用 JSON Value 承载即可 |
| `google/renameio`（原子写） | `tempfile::persist` + `fs::rename` | 跨平台原子写成熟方案 |

**核心原则**：

1. **仓库即真相，文件名即元数据**——无 sidecar manifest
2. **三层状态分离**：Source → Target → Actual
3. **装饰器矩阵**：所有 IO 走同一个 trait，diff/dry-run/archive 复用 apply 主循环
4. **lazy + 进程内缓存**：模板/秘密/bbolt 三处都是按需计算
5. **渐进集成**：默认使用系统 `git`，可选择内置 gix；模板、加密、redb 和归档共用 source/target 计划。

### 0.1 已确认的用户流程

用户提供一个可克隆的 Git URL，例如 `https://github.com/szwathub/dotfiles`，xyrev 保存仓库来源，将其克隆到本地 source 目录，再依据文件名计算 `$HOME` 下的目标路径。

```bash
# 初始化并部署；示例仓库需要先迁移为下述路径协议
xyrev dotfile init https://github.com/szwathub/dotfiles --apply

# 也可以先克隆，再检查和应用
xyrev dotfile init https://github.com/szwathub/dotfiles
xyrev dotfile diff
xyrev dotfile apply

# 查看状态和受管理文件
xyrev dotfile status
xyrev dotfile managed

# 拉取远端更新并重新应用
xyrev dotfile update
```

`init` 默认只克隆和记录本地配置，`--apply` 才部署。后续命令读取已保存的 URL、分支和目录，无需重复传 URL。`--source` 和 `--destination` 可覆盖目录，便于测试和隔离试用。

首版默认目录统一为用户主目录下的 `~/.xyrev/dotfile/`：`source/` 存 Git 工作树，`config.toml` 存本机配置，`state/` 存已应用状态，`backups/` 存覆盖前备份。默认 destination 为用户主目录。每个 source / destination 组合独立记录状态，避免测试目录与真实 HOME 混用。

### 0.2 仓库协议：路径即映射

**采用 chezmoi 风格命名，不引入逐文件映射清单。** 仓库根目录对应 destination 根目录，对每个路径分量分别解析属性。例如：

```text
dotfiles/
├── .chezmoiignore
├── README.md
├── dot_gitconfig
├── dot_gitignore
├── dot_config/
│   └── fish/
│       └── config.fish
├── private_dot_ssh/
│   └── private_config
└── dot_vimrc
```

| Source 相对路径 | 默认目标路径 | 行为 |
|---|---|---|
| `dot_gitconfig` | `~/.gitconfig` | 写入普通文件 |
| `dot_config/fish/config.fish` | `~/.config/fish/config.fish` | 按目录结构写入 |
| `private_dot_ssh/private_config` | `~/.ssh/config` | Unix 上目录为 0700、文件为 0600 |
| `dot_local/bin/executable_tool` | `~/.local/bin/tool` | Unix 上增加可执行权限 |
| `symlink_dot_example` | `~/.example` | 将 source 文件内容作为符号链接目标 |

普通 source 文件部署为独立文件；显式 `symlink_` 才创建符号链接。旧 `.symlink` 后缀需要迁移，不具有特殊语义。

支持普通文件、目录、`dot_`、`private_`、`readonly_`、`executable_`、`empty_`、`literal_`、`.literal`、文件形式的 `symlink_`，以及 `encrypted_`、`.tmpl`、`create_`、`modify_`、`remove_`、`exact_`、`external_` 和 run 脚本。未带 `empty_` 的空普通 source 文件表示目标应不存在。属性顺序和适用类型执行严格校验。

控制文件使用 `.xyrev*` 命名，同时接受对应 `.chezmoi*` 别名以方便迁移。实现了 ignore/remove（含模板）、data 文件与目录、templates/scripts/externals 目录、external 文件、root/version 和初始化配置模板；具体格式与字段见 README。未知控制文件报错。Git 元数据排除。

`.xyrevignore[.tmpl]` 按目标相对路径匹配，支持 `**`、局部规则和优先的 `!` 例外。`remove_` 只删除文件、链接和空目录；remove 控制文件和 exact 目录可以显式递归删除，删除前备份。

模板使用 **MiniJinja**，Go Template 必须迁移。例如 `{{ .chezmoi.os }}` 改为 `{{ xyrev.os }}`，`{{ if ... }}` 改为 `{% if ... %}`。本机数据、初始化提示、模板指令与秘密函数均已实现，语法以 README 为准。

### 0.3 示例仓库的迁移方式

以 [szwathub/dotfiles](https://github.com/szwathub/dotfiles) 的当前模块布局为输入，迁移示例如下：

| 原仓库路径 | 新 source 路径 | 目标路径 |
|---|---|---|
| `git/.gitconfig.symlink` | `dot_gitconfig` | `~/.gitconfig` |
| `git/.gitignore.symlink` | `dot_gitignore` | `~/.gitignore` |
| `fish/config.fish.symlink` | `dot_config/fish/config.fish` | `~/.config/fish/config.fish` |
| `fish/functions.symlink/` | `dot_config/fish/functions/` | `~/.config/fish/functions/` |
| `ssh/config.symlink` | `private_dot_ssh/private_config` | `~/.ssh/config` |
| `vim/.vimrc.symlink` | `dot_vimrc` | `~/.vimrc` |
| `vim/.vim.symlink/` | `dot_vim/` | `~/.vim/` |

迁移保留文件内容，按新规则调整 source 路径，并检查配置中的仓库绝对路径引用。Git 用户名等原先由脚本写入的本机差异需要单独处理；可改为 MiniJinja 模板并通过本机 `[data]` 配置差异。

该仓库的 Fish functions 和 Vim 插件包含 Git 子模块。移动它们时同步更新 `.gitmodules` 中的 path，保留子模块关系。首版通过显式 `--recurse-submodules` 初始化；需要的子模块未检出时停止部署并提示。子模块中的 Git 元数据同样排除。

原 `bootstrap`、Homebrew 安装与 macOS 系统设置不属于普通 dotfile 文件部署。迁移时从部署源中排除；可按需改为 `run_once_` / `run_onchange_` 等入口。这份文档只定义迁移方案，远端仓库迁移应单独实施。

### 0.4 部署、冲突与更新规则

1. **先规划后写入**：解析全部 source，生成统一 `ChangePlan`，校验非法属性、重复目标、目标路径越界、未支持能力及冲突。`diff`、`status`、`apply --dry-run` 共用计划，目标目录和已应用状态保持只读。
2. **只处理受管理路径**：普通目录只创建或更新声明的子项，保留未管理文件。从 source 移除文件表示解除管理并保留目标；显式删除通过 `remove_` / `exact_` 或 remove 控制文件声明。
3. **已有文件冲突**：首次应用时目标已存在且不同，或应用后用户又修改过目标，默认列出冲突并在任何目标写入前退出。同内容及同权限项跳过；明确 `--force` 时先备份再覆盖。
4. **三方比较**：同时比较期望状态、磁盘实际状态和上次成功写入状态。实际已经等于期望时直接收敛；不能仅因实际与上次状态不同就一律报冲突。文件类型、内容、权限和链接目标都参与判断。
5. **写入可追踪**：普通文件采用同目录临时文件，设置正确权限后替换；目录、符号链接和类型冲突分别处理。检测到已有父目录是符号链接时默认报错，防止解析后写出 destination。失败时报告已完成项和备份位置，只记录实际成功的写入；首版不承诺跨多个文件的事务回滚。
6. **更新保留本地工作**：`update` 先校验 source 工作树及子模块状态，存在本地修改或分支分叉时停止；远端更新仅允许快进。拉取成功后重新生成计划并应用。部署冲突不重置 source 或目标，修复后可重试 `apply`。
7. **重复执行幂等**：第二次 `apply` 不重写相同文件、不重复备份；`--force` 不作用于 Git 工作树清理。Git 下载失败和认证失败返回非零退出码，保留已有可用 source。

---

## 1. 项目布局与长期拆分参考

当前先扩展 `src/cli/dotfile/`，将属性解析、source 扫描、计划、文件系统、Git 和状态逻辑放在独立的 `src/dotfile/` 模块。`main.rs` 负责接入现有 `xyrev dotfile` 分发，已有 `init` / `agent` / `apple` 保持各自职责。

下列 Cargo workspace 是功能增长后的拆分参考，CLI 保持 `xyrev`，功能库统一使用 `xyrev-dotfile-*` 命名。拆分时先把路径、元数据、错误等公共类型放入独立基础 crate，避免 `core` 与 `vfs` 相互依赖；CLI 层负责组合后端。首版不要求预先建立这些 crate：

```
xyrev/
├── Cargo.toml                # workspace
├── crates/
│   ├── xyrev/                # bin：CLI 入口（clap）
│   │   └── src/
│   │       ├── main.rs
│   │       └── commands/     # 每条子命令一个文件
│   │           ├── apply.rs
│   │           ├── add.rs
│   │           ├── diff.rs
│   │           ├── init.rs
│   │           ├── edit.rs
│   │           ├── status.rs
│   │           ├── managed.rs
│   │           ├── update.rs
│   │           └── archive.rs
│   ├── xyrev-dotfile-core/           # lib：核心模型与 reconcile
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── attr.rs          # 文件名属性解析（见 §3）
│   │       ├── source_state.rs  # SourceState + tree
│   │       ├── target_state.rs
│   │       ├── actual_state.rs
│   │       ├── entry/           # 三层 entry 定义
│   │       │   ├── source.rs
│   │       │   ├── target.rs
│   │       │   └── actual.rs
│   │       ├── reconcile.rs     # apply/diff 主循环
│   │       ├── path.rs          # AbsPath / RelPath / SourceRelPath newtype
│   │       ├── perm.rs          # 权限计算
│   │       └── error.rs
│   ├── xyrev-dotfile-vfs/            # lib：文件系统抽象 + 装饰器
│   │   └── src/
│   │       ├── lib.rs           # Vfs trait
│   │       ├── real.rs
│   │       ├── dry_run.rs
│   │       ├── debug.rs
│   │       ├── git_diff.rs
│   │       ├── read_only.rs
│   │       ├── tar_writer.rs
│   │       ├── zip_writer.rs
│   │       └── archive_reader.rs
│   ├── xyrev-dotfile-template/       # lib：模板引擎 + 内置函数 + chezmoi.* 数据
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine.rs        # minijinja wrapper
│   │       ├── directives.rs    # chezmoi:template: 指令解析
│   │       ├── funcs/           # 每类函数一个模块
│   │       │   ├── basic.rs
│   │       │   ├── system.rs
│   │       │   ├── onepassword.rs
│   │       │   ├── bitwarden.rs
│   │       │   ├── keepassxc.rs
│   │       │   ├── vault.rs
│   │       │   ├── pass.rs
│   │       │   ├── keyring.rs
│   │       │   └── age.rs
│   │       └── data.rs          # chezmoi.* 变量构造
│   ├── xyrev-dotfile-encryption/     # lib：加密抽象
│   │   └── src/
│   │       ├── lib.rs           # Encryption trait
│   │       ├── age.rs           # 内置 age crate
│   │       ├── gpg.rs           # fork gpg 子进程
│   │       └── none.rs
│   ├── xyrev-dotfile-state/          # lib：持久化（redb 封装 bbolt 的语义）
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── persistent.rs    # PersistentState trait
│   │       ├── redb_impl.rs
│   │       ├── mock.rs
│   │       └── debug.rs
│   ├── xyrev-dotfile-git/            # lib：gix 封装 + 外部 git fallback
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builtin.rs       # gix
│   │       └── external.rs      # git CLI
│   └── xyrev-dotfile-tui/            # lib（可选）：ratatui 组件
├── tests/
│   ├── e2e/                  # txtar 风格 golden test
│   └── fixtures/
└── docs/
```

---

## 2. 类型系统：path、属性、权限

### 2.1 path newtype

chezmoi 用 `AbsPath` / `RelPath` / `SourceRelPath` 三个 newtype 防止混淆。Rust 版本自然借助类型系统：

```rust
// crates/xyrev-dotfile-core/src/path.rs
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AbsPath(PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RelPath(PathBuf);

/// 仓库内的相对路径（可能含 dot_ 等前缀的那一份原始形式）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceRelPath(PathBuf);

impl AbsPath {
    pub fn new(p: impl Into<PathBuf>) -> Result<Self, PathError> {
        let p = p.into();
        if !p.is_absolute() {
            return Err(PathError::NotAbsolute(p));
        }
        Ok(Self(p))
    }
    pub fn join(&self, rel: &RelPath) -> AbsPath { AbsPath(self.0.join(&rel.0)) }
    pub fn as_path(&self) -> &Path { &self.0 }
}
```

好处：编译期阻止"把相对路径喂给需要绝对路径的 API"这类错误——chezmoi 在 Go 里要靠约定。

### 2.2 属性编码

```rust
// crates/xyrev-dotfile-core/src/attr.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFileType {
    File,
    Script { condition: ScriptCondition, order: ScriptOrder },
    Symlink,
    Create,     // 仅当目标不存在时写入
    Modify,     // 以当前内容为 stdin，脚本输出作为新内容
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptCondition { Always, Once, OnChange }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptOrder { Before, During, After }

#[derive(Debug, Clone)]
pub struct FileAttr {
    pub target_name: String,      // dot_ 还原成 .
    pub file_type: SourceFileType,
    pub is_template: bool,        // .tmpl
    pub is_encrypted: bool,
    pub executable: bool,
    pub private: bool,
    pub readonly: bool,
    pub empty: bool,              // 允许空文件
    pub literal: bool,
}

impl FileAttr {
    pub fn parse(source_name: &str) -> Self { /* §3 */ }
    pub fn perm(&self) -> u32 {
        let mut m: u32 = 0o666;
        if self.executable { m |= 0o111; }
        if self.private    { m &= !0o077; }
        if self.readonly   { m &= !0o222; }
        m
    }
}
```

---

## 3. 文件名解析：有序状态机

对应 chezmoi `parseFileAttr`。Rust 版用 `strip_prefix` 链式剥离，返回 `FileAttr`：

下面是覆盖长期属性集合的解析草图。实际实现先按 §0.2 的属性子集提供支持，并依据官方属性顺序验证类型组合；其余已保留的属性在规划时返回不支持错误。不要把草图中的默认分支当成对所有组合的兼容性保证。

```rust
const DOT_PREFIX:        &str = "dot_";
const PRIVATE_PREFIX:    &str = "private_";
const READONLY_PREFIX:   &str = "readonly_";
const EMPTY_PREFIX:      &str = "empty_";
const EXECUTABLE_PREFIX: &str = "executable_";
const ENCRYPTED_PREFIX:  &str = "encrypted_";
const CREATE_PREFIX:     &str = "create_";
const MODIFY_PREFIX:     &str = "modify_";
const REMOVE_PREFIX:     &str = "remove_";
const RUN_PREFIX:        &str = "run_";
const SYMLINK_PREFIX:    &str = "symlink_";
const ONCE_PREFIX:       &str = "once_";
const ONCHANGE_PREFIX:   &str = "onchange_";
const BEFORE_PREFIX:     &str = "before_";
const AFTER_PREFIX:      &str = "after_";
const LITERAL_PREFIX:    &str = "literal_";
const TEMPLATE_SUFFIX:   &str = ".tmpl";
const LITERAL_SUFFIX:    &str = ".literal";

pub fn parse_file_attr(raw: &str) -> FileAttr {
    let mut attr = FileAttr::default();
    let mut name = raw;

    // 1. 文件类型前缀（互斥）
    if let Some(rest) = name.strip_prefix(RUN_PREFIX) {
        let (rest, cond)  = strip_condition(rest);
        let (rest, order) = strip_order(rest);
        attr.file_type = SourceFileType::Script { condition: cond, order };
        name = rest;
    } else if let Some(rest) = name.strip_prefix(REMOVE_PREFIX) {
        attr.file_type = SourceFileType::Remove; name = rest;
    } else if let Some(rest) = name.strip_prefix(CREATE_PREFIX) {
        attr.file_type = SourceFileType::Create; name = rest;
    } else if let Some(rest) = name.strip_prefix(MODIFY_PREFIX) {
        attr.file_type = SourceFileType::Modify; name = rest;
    } else if let Some(rest) = name.strip_prefix(SYMLINK_PREFIX) {
        attr.file_type = SourceFileType::Symlink; name = rest;
    }

    // 2. 普通文件属性（有固定顺序）
    if matches!(attr.file_type, SourceFileType::File | SourceFileType::Create | SourceFileType::Modify) {
        if let Some(rest) = name.strip_prefix(ENCRYPTED_PREFIX)  { attr.is_encrypted = true; name = rest; }
        if let Some(rest) = name.strip_prefix(PRIVATE_PREFIX)    { attr.private      = true; name = rest; }
        if let Some(rest) = name.strip_prefix(READONLY_PREFIX)   { attr.readonly     = true; name = rest; }
        if let Some(rest) = name.strip_prefix(EMPTY_PREFIX)      { attr.empty        = true; name = rest; }
        if let Some(rest) = name.strip_prefix(EXECUTABLE_PREFIX) { attr.executable   = true; name = rest; }
    }

    // 3. literal_ 前缀：剩下的字符串按字面量看待
    if let Some(rest) = name.strip_prefix(LITERAL_PREFIX) {
        attr.literal = true;
        attr.target_name = rest.to_string();
        return attr;
    }

    // 4. 后缀识别
    if let Some(stem) = name.strip_suffix(TEMPLATE_SUFFIX) { attr.is_template = true; name = stem; }
    if let Some(stem) = name.strip_suffix(LITERAL_SUFFIX)  { attr.literal = true; name = stem; }

    // 5. dot_ → .
    if let Some(rest) = name.strip_prefix(DOT_PREFIX) {
        attr.target_name = format!(".{}", rest);
    } else {
        attr.target_name = name.to_string();
    }
    attr
}
```

目录版本 `parse_dir_attr` 类似但更简单，只识别 `exact_` / `private_` / `readonly_` / `dot_`。

---

## 4. Vfs trait：装饰器骨架

### 4.1 接口

```rust
// crates/xyrev-dotfile-vfs/src/lib.rs
use std::fs::{FileType, Metadata, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use xyrev_dotfile_core::path::AbsPath;

pub trait Vfs: Send + Sync {
    fn lstat(&self, p: &AbsPath) -> io::Result<Metadata>;
    fn read_file(&self, p: &AbsPath) -> io::Result<Vec<u8>>;
    fn read_dir(&self, p: &AbsPath) -> io::Result<Vec<DirEntry>>;
    fn readlink(&self, p: &AbsPath) -> io::Result<std::path::PathBuf>;

    fn mkdir(&self, p: &AbsPath, perm: u32) -> io::Result<()>;
    fn write_file(&self, p: &AbsPath, data: &[u8], perm: u32) -> io::Result<()>;
    fn write_symlink(&self, old: &str, new: &AbsPath) -> io::Result<()>;
    fn chmod(&self, p: &AbsPath, perm: u32) -> io::Result<()>;
    fn remove(&self, p: &AbsPath) -> io::Result<()>;
    fn remove_all(&self, p: &AbsPath) -> io::Result<()>;
    fn rename(&self, old: &AbsPath, new: &AbsPath) -> io::Result<()>;

    fn run_cmd(&self, cmd: &mut std::process::Command) -> io::Result<std::process::Output>;
    fn run_script(&self, name: &str, dir: &AbsPath, data: &[u8],
                  opts: &RunScriptOptions) -> io::Result<()>;
}
```

### 4.2 装饰器矩阵

每个装饰器都用 **组合** 包裹内层 `Vfs`：

```rust
// crates/xyrev-dotfile-vfs/src/dry_run.rs
pub struct DryRunVfs<V: Vfs> {
    inner: V,
    modified: std::sync::atomic::AtomicBool,
}

impl<V: Vfs> Vfs for DryRunVfs<V> {
    // 读操作全部透传
    fn read_file(&self, p: &AbsPath) -> io::Result<Vec<u8>> { self.inner.read_file(p) }
    fn lstat(&self, p: &AbsPath) -> io::Result<Metadata> { self.inner.lstat(p) }
    /* ... */

    // 写操作只打标记
    fn write_file(&self, _: &AbsPath, _: &[u8], _: u32) -> io::Result<()> {
        self.modified.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    fn mkdir(&self, _: &AbsPath, _: u32) -> io::Result<()> {
        self.modified.store(true, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
    /* 其他写操作同理 */
}
```

**装饰器栈举例**：
- `xyrev dotfile apply`       → `RealVfs`
- `xyrev dotfile apply --dry-run` → `DryRunVfs<RealVfs>`
- `xyrev dotfile diff`        → `GitDiffVfs<DryRunVfs<RealVfs>>`（外层记录 diff，内层拦截实际写入）
- `xyrev dotfile archive tar` → `TarWriterVfs`
- `xyrev dotfile verify`      → `ReadOnlyVfs<RealVfs>`

主循环完全不知道底层是哪一层——这就是装饰器模式的威力，reconcile 代码只写一遍。

首版以 §0.4 的 `ChangePlan` 为共同输出，diff 与 dry-run 直接展示计划。上面的装饰器栈是可选后端方案；若采用它，不能让外层 dry-run 提前吞掉写调用而使内层 diff 没有输入。

### 4.3 原子写

`RealVfs::write_file` 用 `tempfile::NamedTempFile::persist` 实现：

```rust
fn write_file(&self, p: &AbsPath, data: &[u8], perm: u32) -> io::Result<()> {
    let parent = p.as_path().parent().ok_or_else(|| io::ErrorKind::InvalidInput)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)?;
    tmp.write_all(data)?;
    tmp.as_file().set_permissions(Permissions::from_mode(perm))?;
    tmp.as_file().sync_all()?;
    tmp.persist(p.as_path())
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    Ok(())
}
```

---

## 5. 三层状态机

### 5.1 SourceStateEntry trait

```rust
// crates/xyrev-dotfile-core/src/entry/source.rs
pub trait SourceStateEntry {
    fn origin(&self) -> &SourceOrigin;
    fn source_rel_path(&self) -> &SourceRelPath;
    fn order(&self) -> ScriptOrder { ScriptOrder::During }

    /// 惰性求值：渲染模板、解密、读取外部资源。只在 apply 真正需要时调用。
    fn target_state_entry(
        &self,
        ctx: &EvalContext<'_>,
    ) -> Result<Box<dyn TargetStateEntry>, DotfileError>;
}

pub struct SourceStateFile {
    source_rel: SourceRelPath,
    attr: FileAttr,
    raw_contents: Lazy<Vec<u8>>,   // 磁盘 IO 延迟到第一次调用
}

pub struct SourceStateDir  { /* ... */ }
pub struct SourceStateScript { /* ... */ }
pub struct SourceStateRemove { /* ... */ }
```

`Lazy<T>` 可用 `once_cell::sync::OnceCell` 实现，精确对应 chezmoi 的 `Evaluate()` 语义。

### 5.2 TargetStateEntry trait

```rust
pub trait TargetStateEntry {
    fn entry_state(&self, umask: u32) -> Result<EntryState, DotfileError>;

    /// 判断本次 apply 是否应该跳过（run_once_ / run_onchange_ 幂等判定）
    fn skip_apply(
        &self,
        state: &dyn PersistentState,
        target: &AbsPath,
    ) -> Result<bool, DotfileError> { Ok(false) }

    /// 真正执行写入，返回是否有变更
    fn apply(
        &self,
        vfs: &dyn Vfs,
        state: &mut dyn PersistentState,
        actual: &ActualStateEntry,
    ) -> Result<bool, DotfileError>;
}
```

### 5.3 ActualStateEntry（磁盘现状）

```rust
pub enum ActualStateEntry {
    File    { path: AbsPath, perm: u32, contents: Vec<u8>, sha256: [u8; 32] },
    Dir     { path: AbsPath, perm: u32 },
    Symlink { path: AbsPath, target: PathBuf },
    Absent  { path: AbsPath },
}

impl ActualStateEntry {
    pub fn probe(vfs: &dyn Vfs, path: &AbsPath) -> Result<Self, DotfileError> {
        match vfs.lstat(path) {
            Err(e) if e.kind() == io::ErrorKind::NotFound =>
                Ok(Self::Absent { path: path.clone() }),
            Err(e) => Err(e.into()),
            Ok(meta) => {
                let ft = meta.file_type();
                if ft.is_symlink() { /* ... */ }
                else if ft.is_dir() { /* ... */ }
                else { /* file: read + sha256 */ }
            }
        }
    }
}
```

### 5.4 EntryState（写入 redb 用于三方合并）

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EntryState {
    pub kind: EntryKind,
    pub mode: u32,
    pub contents_sha256: [u8; 32],
    pub target_sha256:   Option<[u8; 32]>, // 符号链接用
    pub overwrite:       bool,
}
```

---

## 6. Reconcile 主循环

本节保留逐 entry 执行的技术草图。首版需要先遍历生成并校验完整 `ChangePlan`，再进入写入阶段，确保后面才发现的冲突不会导致前面已经写入。执行期间发生 IO 错误时的部分成功处理见 §0.4。

```rust
// crates/xyrev-dotfile-core/src/reconcile.rs
pub struct ApplyOptions {
    pub umask: u32,
    pub include: EntryTypeSet,
    pub exclude: EntryTypeSet,
    pub recursive: bool,
    pub pre_apply: Option<Box<PreApplyHook>>,
}

pub fn apply(
    source: &SourceState,
    vfs: &dyn Vfs,
    state: &mut dyn PersistentState,
    dest_dir: &AbsPath,
    opts: &ApplyOptions,
) -> Result<ApplyReport, DotfileError> {
    let mut report = ApplyReport::default();
    // before 脚本
    for script in source.scripts(ScriptOrder::Before) {
        run_script_if_needed(script, vfs, state, &mut report)?;
    }

    // 普通 entries
    for (target_rel, src_entry) in source.walk_ordered() {
        let target_abs = dest_dir.join(&target_rel);
        let tgt = src_entry.target_state_entry(&source.eval_ctx())?;
        let tgt_state = tgt.entry_state(opts.umask)?;

        if tgt.skip_apply(state, &target_abs)? {
            report.skipped += 1; continue;
        }

        let actual = ActualStateEntry::probe(vfs, &target_abs)?;

        // 三方合并：比较 tgt_state / last_written / actual
        if let Some(last) = state.get_entry_state(&target_abs)? {
            three_way_check(&last, &actual, &tgt_state, opts)?;
        }

        if tgt.apply(vfs, state, &actual)? {
            report.changed += 1;
            state.set_entry_state(&target_abs, &tgt_state)?;
        } else {
            report.unchanged += 1;
        }
    }

    for script in source.scripts(ScriptOrder::After) {
        run_script_if_needed(script, vfs, state, &mut report)?;
    }
    Ok(report)
}
```

**`three_way_check`**：
- 若 `actual.sha256 != last.sha256`，说明用户在上次 apply 后手动改过磁盘。默认行为是交互式提示（`ratatui` 菜单），CI 模式下报错。

---

## 7. 模板引擎

### 7.1 选型

**minijinja**：后续模板阶段的候选，采用 Jinja 风格表达式与控制语句，可注册自定义函数和 loader。它与 chezmoi 使用的 Go Template 语法不同，例如变量应写为 `{{ name }}`，而不是 `{{ .name }}`。函数、管道、条件语句及错误处理语义都需要单独迁移和测试。

首版只应用静态内容，遇到 `.tmpl` 在写入目标前报错。采用 chezmoi 风格文件名不等同于承诺完整模板兼容。参考：[MiniJinja syntax](https://docs.rs/minijinja/latest/minijinja/syntax/index.html)、[chezmoi templating](https://www.chezmoi.io/user-guide/templating/)。

### 7.2 封装

```rust
// crates/xyrev-dotfile-template/src/engine.rs
use minijinja::{Environment, Value, context};

pub struct TemplateEngine {
    env: Environment<'static>,
    data: Value,   // chezmoi.* + .chezmoidata.* 合并结果
}

impl TemplateEngine {
    pub fn new(data: TemplateData) -> Self {
        let mut env = Environment::new();
        env.set_keep_trailing_newline(true);
        env.set_lstrip_blocks(true);
        crate::funcs::basic::register(&mut env);
        crate::funcs::system::register(&mut env);
        // ...每个秘密管理器有自己的 register
        Self { env, data: serde_json::to_value(data).unwrap().into() }
    }

    pub fn render(&self, name: &str, source: &str, extra: &Value) -> Result<String, DotfileError> {
        let (source, directives) = parse_and_remove_directives(source);
        let env = directives.apply_overrides(self.env.clone());
        let tmpl = env.template_from_str(&source)?;
        // 深拷贝防止模板间污染
        let data = deep_merge(self.data.clone(), extra.clone());
        Ok(tmpl.render(data)?)
    }
}
```

### 7.3 `chezmoi:template:` 指令

```rust
// crates/xyrev-dotfile-template/src/directives.rs
#[derive(Default)]
pub struct Directives {
    pub left_delimiter:  Option<String>,
    pub right_delimiter: Option<String>,
    pub line_ending:     Option<LineEnding>,
    pub encoding:        Option<String>,
    pub missing_key:     Option<MissingKey>,
}

/// 扫描每一行的注释，找出 `chezmoi:template: key=value` 指令并从源码中**物理移除**。
/// 这样后续如果调整了分隔符，指令本身不会受影响。
pub fn parse_and_remove_directives(src: &str) -> (String, Directives) {
    let mut dirs = Directives::default();
    let mut out = String::with_capacity(src.len());
    for line in src.split_inclusive('\n') {
        if let Some(kv) = extract_directive(line) {
            dirs.merge(kv);
            continue;          // 指令行不写回
        }
        out.push_str(line);
    }
    (out, dirs)
}
```

### 7.4 `chezmoi.*` 内置数据

```rust
// crates/xyrev-dotfile-template/src/data.rs
#[derive(Serialize)]
pub struct ChezmoiData {
    pub arch:            String,   // std::env::consts::ARCH
    pub os:              String,   // std::env::consts::OS
    pub hostname:        String,   // sysinfo / hostname crate
    pub fqdn_hostname:   String,
    pub kernel:          KernelInfo,
    pub os_release:      BTreeMap<String, String>, // /etc/os-release
    pub home_dir:        PathBuf,
    pub dest_dir:        PathBuf,
    pub source_dir:      PathBuf,
    pub cache_dir:       PathBuf,
    pub config_file:     PathBuf,
    pub username:        String,
    pub uid:             u32,
    pub gid:             u32,
    pub group:           String,
    pub version:         VersionInfo,
    pub path_separator:  String,
    pub executable:      PathBuf,
}
```

后续模板阶段需要分别定义数据文件与本机配置的 schema、优先级及加载时机，使用显式 `deep_merge` 辅助函数合并 `serde_json::Value`；它没有内置的递归 `merge` 方法。模板兼容边界仍以 §7.1 为准。

---

## 8. 加密抽象

```rust
// crates/xyrev-dotfile-encryption/src/lib.rs
pub trait Encryption: Send + Sync {
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncError>;
    fn encrypt(&self, plaintext:  &[u8]) -> Result<Vec<u8>, EncError>;
    fn encrypted_suffix(&self) -> &str;  // 反向影响 parse_file_attr 的后缀识别
}
```

### 8.1 内置 age

```rust
// crates/xyrev-dotfile-encryption/src/age.rs
use age::{Decryptor, Encryptor, armor::{ArmoredWriter, Format}};
use std::io::{Read, Write};

pub struct AgeEncryption {
    recipients: Vec<Box<dyn age::Recipient + Send + Sync>>,
    identities: Vec<Box<dyn age::Identity + Send + Sync>>,
    suffix: String,
}

impl Encryption for AgeEncryption {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, EncError> {
        let encryptor = Encryptor::with_recipients(self.recipients.clone())
            .ok_or(EncError::NoRecipients)?;
        let mut out = Vec::new();
        let armored = ArmoredWriter::wrap_output(&mut out, Format::AsciiArmor)?;
        let mut w = encryptor.wrap_output(armored)?;
        w.write_all(plaintext)?;
        w.finish()?.finish()?;
        Ok(out)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncError> {
        let decryptor = match age::Decryptor::new(armor_strip(ciphertext)?)? {
            Decryptor::Recipients(d) => d,
            _ => return Err(EncError::PassphraseNotSupported),
        };
        let mut out = Vec::new();
        decryptor.decrypt(self.identities.iter().map(|b| b.as_ref()))?.read_to_end(&mut out)?;
        Ok(out)
    }

    fn encrypted_suffix(&self) -> &str { &self.suffix }
}
```

### 8.2 gpg：私有临时目录

gpg 不易走 stdin/stdout，沿用 chezmoi 的 "私有 tmp dir → 子进程 → 读回" 模式。用 `tempfile::TempDir`（自动 0o700 权限）：

```rust
pub struct GpgEncryption { cmd: PathBuf, recipient: Option<String>, /* ... */ }

impl Encryption for GpgEncryption {
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, EncError> {
        let dir = tempfile::Builder::new().prefix("xyrev-dotfile-gpg-").tempdir()?;
        let cipher_path = dir.path().join(format!("ciphertext{}", self.encrypted_suffix()));
        let plain_path  = dir.path().join("plaintext");
        std::fs::write(&cipher_path, ciphertext)?;
        let status = std::process::Command::new(&self.cmd)
            .arg("--output").arg(&plain_path)
            .arg("--decrypt").arg(&cipher_path)
            .status()?;
        if !status.success() { return Err(EncError::GpgFailed(status)); }
        Ok(std::fs::read(plain_path)?)
    }
    /* encrypt 类似 */
}
```

---

## 9. 持久化：redb

### 9.1 为什么不是 sled？

- `sled` 长期处于 beta，磁盘格式不保证稳定，不适合长期存本地状态
- `redb` 是纯 Rust、ACID、单文件、mmap、stable 1.x 版本，API 和 bbolt 几乎可以 1:1 对应

### 9.2 Bucket → TableDefinition

```rust
// crates/xyrev-dotfile-state/src/redb_impl.rs
use redb::{Database, TableDefinition};
use serde::{Serialize, Deserialize};

const ENTRY_STATE:        TableDefinition<&[u8], &[u8]> = TableDefinition::new("entry_state");
const SCRIPT_STATE:       TableDefinition<&[u8], &[u8]> = TableDefinition::new("script_state");
const CONFIG_STATE:       TableDefinition<&[u8], &[u8]> = TableDefinition::new("config_state");
const GIT_EXTERNAL_STATE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("git_repo_external");

pub struct RedbPersistentState {
    db: Database,
    read_only: bool,
}

impl PersistentState for RedbPersistentState {
    fn get(&self, bucket: Bucket, key: &[u8]) -> Result<Option<Vec<u8>>, DotfileError> {
        let r = self.db.begin_read()?;
        let t = r.open_table(bucket.table_def())?;
        Ok(t.get(key)?.map(|v| v.value().to_vec()))   // clone，不依赖事务生命周期
    }
    fn set(&mut self, bucket: Bucket, key: &[u8], value: &[u8]) -> Result<(), DotfileError> {
        if self.read_only { return Err(DotfileError::ReadOnlyState); }
        let w = self.db.begin_write()?;
        { let mut t = w.open_table(bucket.table_def())?; t.insert(key, value)?; }
        w.commit()?;
        Ok(())
    }
    /* delete / iter */
}
```

**装饰器**：`MockPersistentState`（内存 HashMap，测试用）、`NullPersistentState`（啥都不记）、`ReadOnlyPersistentState`、`DebugPersistentState`（tracing::debug 日志）。

### 9.3 幂等脚本判定

```rust
impl TargetStateEntry for TargetStateScript {
    fn skip_apply(&self, state: &dyn PersistentState, target: &AbsPath)
        -> Result<bool, DotfileError>
    {
        match self.condition {
            ScriptCondition::Always => Ok(false),
            ScriptCondition::Once => {
                let key = hex::encode(self.contents_sha256);
                Ok(state.get(Bucket::ScriptState, key.as_bytes())?.is_some())
            }
            ScriptCondition::OnChange => {
                let Some(bytes) = state.get(Bucket::EntryState, target.as_path().as_os_str().as_encoded_bytes())? else {
                    return Ok(false);
                };
                let es: EntryState = postcard::from_bytes(&bytes)?;
                Ok(es.contents_sha256 == self.contents_sha256)
            }
        }
    }
}
```

- `Once`：key 用 **脚本内容的 SHA-256 本身** —— 内容变化等同于"新的 once 任务"
- `OnChange`：key 是目标路径，值是上次内容 hash，对比决定是否再跑

---

## 10. 秘密管理器模式

与 chezmoi 一致：**解锁 → 调 CLI → 解析 JSON → 进程内缓存**。用一个泛型模板：

```rust
// crates/xyrev-dotfile-template/src/funcs/secret_base.rs
pub struct SecretManager<U: Unlocker, P: OutputParser> {
    cmd: PathBuf,
    unlocker: U,
    parser: P,
    cache: RwLock<HashMap<String, Value>>,
}

impl<U: Unlocker, P: OutputParser> SecretManager<U, P> {
    pub fn get(&self, args: &[&str]) -> Result<Value, DotfileError> {
        let key = args.join("\u{0}");
        if let Some(v) = self.cache.read().unwrap().get(&key) { return Ok(v.clone()); }
        self.unlocker.ensure_unlocked()?;
        let output = std::process::Command::new(&self.cmd).args(args).output()?;
        if !output.status.success() {
            return Err(DotfileError::SecretManager { stderr: String::from_utf8_lossy(&output.stderr).into() });
        }
        let parsed = self.parser.parse(&output.stdout)?;
        self.cache.write().unwrap().insert(key, parsed.clone());
        Ok(parsed)
    }
}
```

注册到 minijinja：

```rust
pub fn register(env: &mut Environment<'_>, bw: Arc<SecretManager<BwUnlock, JsonParser>>) {
    env.add_function("bitwarden", move |args: Vec<String>| -> Result<Value, minijinja::Error> {
        bw.get(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .map_err(|e| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, e.to_string()))
    });
}
```

秘密管理器属于后续扩展阶段。届时可按需要接入 `bitwarden` / `pass` / `onepassword` / `keyring` / `gopass`，各自独立验证；Git URL 与静态文件部署闭环不依赖它们。

---

## 11. CLI 层（clap derive）

在已有 `xyrev` CLI 中扩展嵌套命令。根级 `--verbose` 沿用现有定义，dotfile 子命令避免重复占用其短选项。首版接口草图：

```rust
// src/cli/dotfile/mod.rs（拟定接口）
#[derive(clap::Args, Debug)]
pub struct DotfileCommand {
    #[arg(long, global = true)]
    pub source: Option<std::path::PathBuf>,
    #[arg(long, global = true)]
    pub destination: Option<std::path::PathBuf>,
    #[command(subcommand)]
    pub command: DotfileCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DotfileCommands {
    Init(InitArgs),
    Diff,
    Status,
    Apply(ApplyArgs),
    Update,
    Managed,
}

#[derive(clap::Args, Debug)]
pub struct InitArgs {
    pub repo: String,
    #[arg(long)]
    pub branch: Option<String>,
    #[arg(long)]
    pub apply: bool,
    #[arg(long)]
    pub recurse_submodules: bool,
}

#[derive(clap::Args, Debug, Default)]
pub struct ApplyArgs {
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub force: bool,
}
```

各命令在 `src/cli/dotfile/*.rs` 中负责参数与报告，核心逻辑在 `src/dotfile/`。初始化、扫描、计划、执行等错误通过 `Result` 传到顶层，输出带路径与原因的错误并返回非零退出码。

`diff`、`status` 和 `apply --dry-run` 只读取 source、destination 和状态，生成并展示同一份 `ChangePlan`。`apply` 校验计划后执行；`init --apply` 与 `update` 复用该执行路径。首版检测到冲突就失败，需要覆盖时由用户调用 `apply --force`；备份规则见 §0.4。

`add` / `edit` / `archive` / `execute-template` / `unmanaged` / `data` / `verify` / `doctor` / `encrypt` / `decrypt` / `keygen` / `cd` 均已接入 `xyrev dotfile`。顶层 `completions` 和 `man` 从同一 clap 定义生成。

---

## 12. init / bootstrap

入口为 `xyrev dotfile init <git-url> [--branch <branch>] [--apply] [--recurse-submodules] [--builtin]`。xyrev 自身安装与 dotfile 初始化是两个独立步骤；原有顶层 `xyrev init` 继续承担应用配置初始化职责。

首版接受完整 HTTPS URL（包括有、无 `.git` 后缀的地址）、`ssh://` URL 和 `git@host:owner/repo.git` 形式，并接受用于本地测试的绝对 `file://` URL。私有仓库认证复用系统 Git 的 SSH 与凭据配置；进程参数逐项传递。`user` / `user/repo` 等 GitHub 缩写及裸本地路径导入后续扩展，不应通过字符串拼接改变用户明确提供的非 GitHub URL。

初始化流程：

0. 默认系统后端及 `--recurse-submodules` 在克隆前执行 Git 可用性检查；缺失或无法运行时停止并给出当前平台的安装、验证及重试指引。macOS 识别 `/usr/bin/git` 占位程序，先检查 Xcode / Command Line Tools，避免隐式启动安装 GUI。普通内置后端不强制依赖系统 Git；子模块与 LFS 回退仍需 Git。安装指引不执行包管理器或修改系统。
1. 加载本机配置，确定 source、destination 与各自状态目录；确保 source 不会成为本次部署目标。
2. source 不存在时，在同一父目录的临时目录中 clone，成功后再移动到正式位置。未传 `--branch` 时跟随远端默认分支，不硬编码 `main` 或 `master`。
3. source 已存在时，验证其为预期仓库并核对 origin、分支及本机配置。同一来源重复初始化复用已有工作树；URL 或分支不一致、已有非仓库目录等情况返回明确错误，不清空或重置目录。远端更新由 `update` 完成。
4. 用户传入 `--recurse-submodules` 时初始化并检出子模块；记录该选择供后续 `update` 复用。需要的子模块未检出、认证失败或 clone 失败时停止，不将不完整工作树视为可部署 source。
5. 原子保存 URL、实际分支、source / destination 和子模块选项到本机 `config.toml`。URL 中的认证秘密不写入日志或普通配置。
6. 未传 `--apply` 时报告克隆位置及下一步 `diff` / `apply` 命令。传入时，按 §0.2 解析完整 source，生成计划并按 §0.4 部署。

默认 Git 后端使用系统 `git`。`--builtin` 使用 gix 完成普通仓库的克隆、分支选择、状态校验和快进更新。更新先在同级临时目录准备 checkout，保留原 Git 配置、hooks、分支、标签和历史；确认原 source 仍未变化后替换，失败时恢复原目录。SSH 传输依赖 SSH 客户端；子模块及 LFS 回退到系统 Git。HTTPS/SSH 的真实凭据服务未在隔离测试中连接验证，不能宣称所有机器均零外部依赖。

初始化会渲染 `.xyrev.toml.tmpl`（兼容 `.chezmoi.toml.tmpl`），仅允许修改 data/encryption/git/editor。可配置 `git.lfs=true`，初始化和更新后调用 git-lfs 拉取对象。发现尚未检出的指针会停止部署。

---

## 13. 错误处理

```rust
// crates/xyrev-dotfile-core/src/error.rs
#[derive(thiserror::Error, Debug)]
pub enum DotfileError {
    #[error("io error at {path}: {source}")]
    Io { path: PathBuf, #[source] source: std::io::Error },

    #[error("template `{name}`: {source}")]
    Template { name: String, #[source] source: minijinja::Error },

    #[error("encryption failed: {0}")]
    Encryption(#[from] EncError),

    #[error("persistent state: {0}")]
    State(#[from] StateError),

    #[error("invalid source file name `{name}`: {reason}")]
    InvalidSourceName { name: String, reason: String },

    #[error("target `{path}` was modified outside xyrev dotfile")]
    ThreeWayConflict { path: PathBuf },

    #[error("script `{name}` failed with exit code {code}")]
    ScriptFailed { name: String, code: i32 },

    #[error("read-only persistent state")]
    ReadOnlyState,

    /* ... */
}
```

顶层 CLI 用 `anyhow::Error`，库层用 `DotfileError`。`tracing` + `tracing-subscriber` 做结构化日志，`--verbose` 提升到 `debug`。

---

## 14. 测试策略

### 14.1 单元测试

- `attr.rs`：property-test（`proptest`）验证 round-trip：`FileAttr → name → parse → FileAttr`
- `template/directives.rs`：snapshot test（`insta`）
- 每个 `Vfs` 装饰器用 `MockVfs` 作为 inner

### 14.2 E2E：txtar 风格

chezmoi 的 scripttest 是它质量的关键。Rust 等价物：

以下为模板阶段的夹具示意，首版使用静态文件夹具，覆盖 §0.2 与 §0.4 的规则。

```
tests/e2e/apply_basic.txtar
----
-- source/dot_gitconfig --
[user]
    name = {{ name }}
-- source/.chezmoidata.toml --
name = "Alice"
-- expected/.gitconfig --
[user]
    name = Alice
-- script --
xyrev dotfile --source source --destination home apply
diff home/.gitconfig expected/.gitconfig
```

用 `assert_cmd` + 自定义 txtar parser 驱动。

### 14.3 golden test

`insta` 对模板渲染结果、parse_file_attr 结果做快照。跨平台差异用 `insta` 的 filter 归一化。

### 14.4 CI 矩阵

GitHub Actions 上跑 `ubuntu-latest` / `macos-latest` / `windows-latest`；Windows 要跳过符号链接与 Unix 权限相关的 case。

---

## 15. 与 chezmoi 相比的 Rust 特有优势

| 方面 | Rust 收益 |
|---|---|
| **Path 类型** | `AbsPath` / `RelPath` newtype 编译期防呆 |
| **错误** | `thiserror` 层次清晰，栈追溯不丢 |
| **trait / 泛型** | 泛型可通过单态化减少动态分发，`dyn Trait` 仍有动态分发开销 |
| **并发** | `rayon::par_bridge` 并行模板渲染，所有秘密 CLI fork 用 `tokio` 异步复用 |
| **单二进制** | 可按平台与依赖选择静态链接目标；普通 release 构建不保证全静态 |
| **依赖选型** | 可优先选择 Rust 实现；外部 Git、SSH、gpg、密码管理器等运行时依赖需单独声明 |

**代价**：
- 编译时间长
- `dyn Trait` 生态下的生命周期标注会比 Go 啰嗦
- minijinja 的函数签名比 Go `text/template` 的 reflection 路径更严格——好处是错误更早暴露

---

## 16. MVP → v1.0 路线图

当前功能已实现；macOS 上 42 项测试在当前工具链和 Rust 1.74 下通过。严格 Clippy、格式检查与 release 构建纳入验收。本机测试使用隔离 HOME、临时 Git 远端、本地 HTTP 服务和模拟第三方 CLI。真实 age 加解密参与测试，远端仓库迁移与第三方账户登录由用户管理。没有改动真实 HOME 的 dotfiles 或远端仓库。

### Git URL 与静态部署

- [x] init URL / 分支 / 子模块，重复初始化校验与本机配置
- [x] source/target/actual 三方比较、diff/status/managed/apply/update
- [x] 冲突预检、覆盖备份、原子文件替换、部分成功记录与重试
- [x] source/destination 状态隔离、并发互斥和路径保护

### 日常维护与模板

- [x] add/edit/unmanaged、递归导入、属性编码、编辑器调用
- [x] MiniJinja、本机数据、嵌套覆盖、include、严格缺失变量与编码/换行指令
- [x] 模板化 ignore/remove/data/external 与初始化配置、非终端提示默认值
- [x] root/version、verify/doctor/data/execute-template/cd
- [x] 自动补全与递归 man pages，从 clap 命令定义生成

### 脚本、秘密与外部资源

- [x] run/once/onchange 与 before/during/after 顺序；redb 成功状态持久化与失败重试
- [x] create/modify/remove/exact/external 属性、只读目录更新与删除备份
- [x] 内置 age、GPG 子进程、加密导入/编辑和内存秘密缓存
- [x] 1Password/Bitwarden/pass/gopass/Vault/Keyring/KeePassXC/LastPass/Doppler/AWS/Azure 适配
- [x] 外部文件/归档/单文件归档/Git、TTL 缓存、SHA-256、过滤与越界校验
- [x] Git LFS、tar/tgz/zip 导出、ratatui 终端冲突确认
- [x] gix 克隆和快进更新、外部 Git 子模块/LFS 回退

保持单一 crate，`src/dotfile/` 按扫描、模板、计划、脚本、秘密、加密、Git、外部资源等模块拆分；workspace 拆分属于长期结构参考，不影响功能使用。文件基线继续使用便于查看和迁移的 JSON，脚本成功状态使用 redb，两个存储均受同一命令锁保护。

| 能力 | macOS/Linux | Windows |
|---|---|---|
| 文件/目录/模板/加密/归档 | Rust 实现 | 同一实现 |
| private/readonly | Unix mode；只读目录写入期间临时解锁 | PowerShell DACL，文件只读属性 |
| executable | Unix execute bits | 文件 execute bits 归一化 |
| symlink | 系统 symlink | 需要开发者模式或系统权限 |
| 脚本 | shebang 或 sh | shebang 或 PowerShell |
| 内置 Git / 第三方适配 | 公共实现，按需外部工具 | 公共实现，按需外部工具 |

macOS 已本机验证；Linux/Windows 添加 CI 测试矩阵，Windows ACL 与 PowerShell 的实际运行结果以对应 runner 为准。HTTPS/SSH 凭据、真实秘密管理器服务与私有 LFS 服务不使用真实账户进行自动验收。

---

## 17. 推荐 crate 清单

| 领域 | crate |
|---|---|
| CLI | `clap` (derive), `clap_complete` |
| 错误 | `thiserror`, `anyhow` |
| 日志 | `tracing`, `tracing-subscriber` |
| 模板 | `minijinja` |
| 加密 | `age`, `sha2`, `hex` |
| KV | `redb` |
| Git | `gix` (builtin), `duct` (调外部 git) |
| FS | `tempfile`, `walkdir`, `ignore`（.gitignore 语义） |
| 归档 | `tar`, `zip`, `flate2` |
| 进程 | `duct`（比 `std::process::Command` 更顺手的管道） |
| 异步 | `tokio`（秘密 CLI 并发） |
| 序列化 | `serde`, `serde_json`, `toml`, `serde_yaml`, `postcard`（EntryState 二进制） |
| 系统信息 | `sysinfo`, `hostname`, `whoami`, `uzers` |
| TUI | `ratatui`, `crossterm` |
| 正则 | `regex`, `once_cell::sync::Lazy` |
| 测试 | `assert_cmd`, `insta`, `proptest`, `tempfile` |
| 密码环 | `keyring` |

---

## 18. 几个非显而易见的工程决策

1. **延迟解密**：`SourceStateFile` 存密文，只有 `target_state_entry()` 被调时才解密——和 chezmoi 的 laziness 一致。好处是 `xyrev dotfile managed` 这类命令无需解锁密钥。

2. **`redb` 事务粒度**：按小批次提交已成功写入目标的 `EntryState`，并在部分失败返回前提交已成功项。数据库事务无法替代目标文件系统事务；提交失败须报告并保留备份，重试时重新比较实际状态，不能将未完成的计划整体标记为已应用。

3. **模板数据深拷贝**：`minijinja::Value` 是 `Arc` 共享，不存在"模板函数 in-place 改共享 map"的风险，所以 chezmoi 的 `copystructure.Copy` 防御在 Rust 里天然不需要。但要注意：秘密管理器的**进程内缓存**必须加锁（`RwLock`）。

4. **Windows 特殊处理**：
   - `private_` 使用受保护 DACL；Windows runner 验证平台行为
   - 符号链接需要 admin 或开发者模式
   - 脚本走 PowerShell 而非 bash；检测脚本 shebang

5. **三方比较的 UX**：首版执行 §0.4 的冲突策略：先判断 actual 是否已经等于 target，再比较 last_written。冲突默认报错，`--force` 先备份再覆盖；终端中可选择备份覆盖，自动化默认报错。跳过冲突项不能把未写入的期望状态记录成已应用状态。

6. **crate 切分的重要性**：`xyrev-dotfile-core` 不依赖 `clap` / `minijinja`，理论上可以被其他工具复用（例如 GUI 前端、LSP server 做 dotfiles 补全）。这是 chezmoi 没有的"礼物"——Go 模块粒度不如 Cargo workspace 细。

---

## 19. 实现边界与参考资料

当前产品目标是：**`xyrev dotfile` 通过 Git URL 读取采用 chezmoi 风格命名的仓库，计算目标文件并幂等部署。**

- §0、§11、§12 定义本次集成的仓库与命令协议，§16 定义首版及后续能力边界。
- 原有 Rust 类型、trait、模板、加密和数据库片段提供技术参考，按阶段补全实现与测试。
- 文件属性兼容、模板兼容、控制文件兼容和零外部依赖是不同验收项，不能相互替代。

参考：[chezmoi 文件属性](https://www.chezmoi.io/reference/source-state-attributes/)、[chezmoi 模板](https://www.chezmoi.io/user-guide/templating/)、[示例仓库](https://github.com/szwathub/dotfiles)。


## 20. 原生 GPG 备份与恢复（v0.5.0）

GPG 密钥管理是 xyrev 的通用能力。dotfiles 仓库只声明恢复策略、保存公钥及 age 密文，不再维护选钥、密码输入、校验、导入逻辑或专用备份/恢复 Shell 脚本。

- `xyrev dotfile gpg backup --fingerprint FPR [--fingerprint FPR ...]` 显式选择完整主密钥，默认密码加密；支持外部私有密码文件、独立 age 身份文件和 GPG 程序路径覆盖。
- `xyrev dotfile gpg restore` 读取备份，校验摘要、公私钥主指纹集合和 ownertrust 后导入 HOME 下的 GPG keyring。兼容历史单/多密钥 `backup-metadata.sh`，仅解析字段，不执行 Shell；原有密码密文无需重新生成。
- `.xyrevgpg.toml[.tmpl]` 声明 `enabled`、`directory`、`program`、`identity`、`passphrase_file`，只允许一种声明文件。默认支持目录为 `.xyrevtemplates/gpg`。
- 统一计划按 before 钩子、目标文件、原生恢复、after 钩子执行。状态按 source/destination 隔离，只保存最近成功恢复的公开摘要；失败不记成功，A→B→A 重新恢复，未变备份不重复导入。
- `diff`、`status`、`apply --dry-run` 不执行 GPG、不读密码；`verify` 对尚未成功恢复的备份返回非零。独立 destination 跳过自动恢复。声明与备份支持文件不会作为目标部署或进入 archive。
- 私钥明文仅在内存和 GPG 标准输入管道中流转，密码不进入 argv、环境变量、日志或状态。写入源备份时拒绝遗漏既有密钥；密码错误、导出不完整、内容不匹配时不发布新备份。

实现及跨平台命令说明以 README 和 `xyrev dotfile gpg --help` 为准。
