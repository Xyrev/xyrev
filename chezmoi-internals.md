# chezmoi 源码深度解析

> 基于对仓库 `internal/chezmoi` 与 `internal/cmd` 子树的实际代码阅读。所有路径均相对于仓库根。

---

## 0. 整体架构:三层状态 + 装饰器文件系统

chezmoi 的整个执行模型可以浓缩成两个核心抽象:

1. **三层状态机**

   ```
   SourceStateEntry  ──模板渲染/解密──▶  TargetStateEntry  ──reconcile──▶  ActualStateEntry
        (源)                                  (目标应有)                       (磁盘现状)
   ```

2. **装饰器模式的 `System` 接口**:所有文件系统操作走同一个 `System` 接口,`apply` / `diff` / `dry-run` / `archive` / `debug` 共用同一份 reconcile 代码,只是替换了底层 `System` 实现。

加上一个 **bbolt 持久化层** 记录脚本运行状态和"上次写入的 EntryState",就构成了 chezmoi 的全部骨架。

---

## 1. Source State → Target State 模型

### 涉及文件
- `internal/chezmoi/sourcestate.go`(3000+ 行,核心)
- `internal/chezmoi/sourcestateentry.go`
- `internal/chezmoi/targetstateentry.go`
- `internal/chezmoi/actualstateentry.go`
- `internal/chezmoi/sourcestatetreenode.go`

### 关键接口

```go
// sourcestateentry.go
type SourceStateEntry interface {
    slog.LogValuer
    Evaluate() error
    Order() ScriptOrder
    Origin() SourceStateOrigin
    SourceRelPath() SourceRelPath
    TargetStateEntry(destSystem System, destDirAbsPath AbsPath) (TargetStateEntry, error)
}
```

实现类型:
- `SourceStateFile` / `SourceStateDir` / `SourceStateImplicitDir` / `SourceStateRemove` / `SourceStateCommand`
- 对应 target: `TargetStateFile` / `TargetStateDir` / `TargetStateSymlink` / `TargetStateScript` / `TargetStateRemove` / `TargetStateModifyDirWithCmd`
- 对应 actual: `ActualStateFile` / `ActualStateDir` / `ActualStateSymlink` / `ActualStateAbsent`

### 实现原理

**Read 阶段**(`SourceState.Read`,sourcestate.go:999):递归遍历 `~/.local/share/chezmoi`,根据每个文件名分发处理:
- 特殊文件(`.chezmoidata.*`、`.chezmoitemplates`、`.chezmoiexternal.*`、`.chezmoiignore`、`.chezmoiremove`、`.chezmoiscripts`、`.chezmoiversion`)走专用解析路径
- 普通文件/目录调用 `parseFileAttr` / `parseDirAttr` 解析文件名属性,构造对应的 `SourceStateEntry` 并插入 `SourceStateEntryTreeNode`(一棵以 target 路径为索引的树)

**Apply 阶段**(sourcestate.go:765,简化):

```go
sourceStateEntry := s.root.Get(targetRelPath)
targetStateEntry, _ := sourceStateEntry.TargetStateEntry(destSystem, destAbsPath)
targetEntryState, _ := targetStateEntry.EntryState(options.Umask)

if skip, _ := targetStateEntry.SkipApply(persistentState, targetAbsPath); skip {
    return nil
}
actualStateEntry, _ := NewActualStateEntry(targetSystem, targetAbsPath, nil, nil)

// preApplyFunc 做三方比较: target vs lastWritten vs actual
// 用于检测「上次 apply 后用户在目标位置又手动改过」的冲突

if changed, err := targetStateEntry.Apply(targetSystem, persistentState, actualStateEntry); ...
return PersistentStateSet(persistentState, EntryStateBucket, targetAbsPath.Bytes(), targetEntryState)
```

**原理总结**:
- `SourceState` 把"源仓库里有什么"解析成树
- `TargetStateEntry` 是惰性求值的:只有调用 `TargetStateEntry()` 时才会渲染模板、解密、读取外部资源
- 每次 apply 完成后把 `EntryState`(含内容 SHA-256)写入 bbolt 的 `EntryStateBucket`,下一次 apply 用它做 **三方合并** 检测人为修改

---

## 2. 文件名属性编码 (`dot_`, `private_`, `run_once_`, …)

### 涉及文件
- `internal/chezmoi/attr.go`(403 行)
- 前缀常量定义在 `internal/chezmoi/chezmoi.go:37-57`

### 前缀字典

```go
afterPrefix      = "after_"
beforePrefix     = "before_"
createPrefix     = "create_"
dotPrefix        = "dot_"
emptyPrefix      = "empty_"
encryptedPrefix  = "encrypted_"
exactPrefix      = "exact_"
executablePrefix = "executable_"
externalPrefix   = "external_"
literalPrefix    = "literal_"
modifyPrefix     = "modify_"
oncePrefix       = "once_"
onChangePrefix   = "onchange_"
privatePrefix    = "private_"
readOnlyPrefix   = "readonly_"
removePrefix     = "remove_"
runPrefix        = "run_"
symlinkPrefix    = "symlink_"
literalSuffix    = ".literal"
TemplateSuffix   = ".tmpl"
```

### 解析算法

`parseFileAttr` 是一台 **有序前缀剥离状态机**:先识别"文件类型前缀"(`create_` / `remove_` / `run_` / `symlink_` / `modify_`),不同类型有不同的合法子前缀集合。例如 `run_` 之后能继续吃 `once_` / `onchange_`,再吃 `before_` / `after_`:

```go
case strings.HasPrefix(name, runPrefix):
    sourceFileType = SourceFileTypeScript
    name = name[len(runPrefix):]
    switch {
    case strings.HasPrefix(name, oncePrefix):
        name = name[len(oncePrefix):]; condition = ScriptConditionOnce
    case strings.HasPrefix(name, onChangePrefix):
        name = name[len(onChangePrefix):]; condition = ScriptConditionOnChange
    default:
        condition = ScriptConditionAlways
    }
    switch {
    case strings.HasPrefix(name, beforePrefix): order = ScriptOrderBefore
    case strings.HasPrefix(name, afterPrefix):  order = ScriptOrderAfter
    }
```

普通文件则按 `encrypted_ → private_ → readonly_ → empty_ → executable_` 的固定顺序剥离;然后把名字前缀 `dot_` 还原成 `.`,后缀 `.tmpl` 标记为模板,`.literal` 表示字面量(允许文件名包含 `dot_` 等字面字符串),`.age` 等加密后缀在 `encrypted_` 出现时被剥离。

最后由 `FileAttr.perm()` 计算 Unix 权限:

```
基础 0o666
| 0o111   if executable
&^ 0o077  if private    // 只有 owner 可读写
&^ 0o222  if readonly
```

**原理**:文件名本身就是元数据载体,从而做到"仓库即真相",不需要任何 sidecar manifest。

---

## 3. 模板引擎

### 涉及文件
- `internal/chezmoi/template.go`(`Template` 包装器和指令解析)
- `internal/chezmoi/templatefuncs.go`(基础函数)
- `internal/cmd/*templatefuncs.go`(每个秘密管理器/外部工具一个文件)
- `internal/cmd/config.go`(注册函数和 `chezmoi.*` 数据)

### 函数表构造

`config.go:377` 起,chezmoi 以 **sprig** 的 `TxtFuncMap()` 为基础,然后通过 `addTemplateFunc` 叠加自定义函数。每个秘密管理器(Bitwarden、1Password、KeePassXC、Vault、pass、gopass、LastPass、keyring、AWS Secrets Manager、Azure Key Vault……)都贡献自己的函数。

### 渲染主循环

```go
// sourcestate.go:870
tmpl, _ := ParseTemplate(options.NameRelPath.String(), options.Data, templateOptions)
for _, t := range s.templates {                 // .chezmoitemplates 下的 partial
    tmpl, _ = tmpl.AddParseTree(t)
}
templateData := s.TemplateData()
if chezmoiTemplateData, ok := templateData["chezmoi"].(map[string]any); ok {
    chezmoiTemplateData["sourceFile"] = options.NameRelPath.String()
    chezmoiTemplateData["targetFile"] = options.DestAbsPath.String()
}
RecursiveMerge(templateData, options.ExtraData)
result, _ := tmpl.Execute(templateData)
```

### `chezmoi.*` 数据

`config.go:1690` 附近构造的内置变量:
`arch / os / hostname / fqdnHostname / kernel / osRelease / homeDir / destDir / sourceDir / cacheDir / configFile / username / uid / gid / group / version / windowsVersion / pathSeparator / executable` 等。

`.chezmoidata.{toml,yaml,json}` 与 `.chezmoi.{toml,yaml,json}` 通过 `RecursiveMerge` 深度合并进同一棵 map。

### 模板指令

`parseAndRemoveDirectives`(template.go:134)在交给 `text/template` 之前,扫描注释里的 `chezmoi:template:` 指令(可设置 `left-delimiter` / `right-delimiter` / `line-ending` / `encoding` / `format-indent` / `missing-key`),并把整行从源文件物理移除——这样指令本身不会再受到分隔符变更的影响。

### 防御性深拷贝

`(*Template).Execute` 用 `copystructure.Copy(data)` 在每次执行前克隆数据,防止某个模板函数 in-place 修改 map 影响后续模板。

---

## 4. 加密抽象

### 涉及文件
- `internal/chezmoi/encryption.go`(接口)
- `internal/chezmoi/ageencryption.go`(age,支持外部 binary 与内置 `filippo.io/age` 库两种模式)
- `internal/chezmoi/gpgencryption.go`(始终 fork gpg 进程)
- `internal/chezmoi/noencryption.go`(空实现)
- `internal/chezmoi/debugencryption.go`(装饰器)

### 接口

```go
type Encryption interface {
    Decrypt(ciphertext []byte) ([]byte, error)
    DecryptToFile(plaintextAbsPath AbsPath, ciphertext []byte) error
    Encrypt(plaintext []byte) ([]byte, error)
    EncryptFile(plaintextAbsPath AbsPath) ([]byte, error)
    EncryptedSuffix() string  // 用于驱动 parseFileAttr 的后缀识别
}
```

### age 内置模式

```go
recipients, _ := e.builtinRecipients()
ciphertextBuffer := &bytes.Buffer{}
armoredCiphertextWriter := armor.NewWriter(ciphertextBuffer)
ciphertextWriteCloser, _ := age.Encrypt(armoredCiphertextWriter, recipients...)
io.Copy(ciphertextWriteCloser, bytes.NewReader(plaintext))
ciphertextWriteCloser.Close(); armoredCiphertextWriter.Close()
```

### gpg 模式:私有临时目录

gpg 不便用管道,所以 `withPrivateTempDir` 在私有目录里写出文件再调用:

```go
withPrivateTempDir(func(tempDirAbsPath AbsPath) error {
    ciphertextAbsPath := tempDirAbsPath.JoinString("ciphertext" + e.EncryptedSuffix())
    os.WriteFile(ciphertextAbsPath.String(), ciphertext, 0o600)
    plaintextAbsPath := tempDirAbsPath.JoinString("plaintext")
    e.run(e.decryptArgs(plaintextAbsPath, ciphertextAbsPath))
    plaintext, err = os.ReadFile(plaintextAbsPath.String())
})
```

**原理**:`Encryption` 是个最小接口,内部细节(子进程 vs 内置库、armor 编码、临时文件)被完全封装。`EncryptedSuffix()` 反向影响文件名解析,使得加解密后缀和文件名 metadata 系统解耦。

---

## 5. `run_once_` / `run_onchange_` 幂等性

### 涉及文件
- `internal/chezmoi/targetstateentry.go`(`TargetStateScript`)
- `internal/chezmoi/persistentstate.go`(bucket 名字)
- `internal/chezmoi/sourcestate.go:readScriptsDir`
- `internal/chezmoi/realsystem.go:RunScript`

### 关键代码 — 是否跳过执行

```go
// targetstateentry.go:413
switch t.condition {
case ScriptConditionAlways:
    return false, nil

case ScriptConditionOnce:
    contentsSHA256, _ := t.ContentsSHA256()
    scriptStateKey := []byte(hex.EncodeToString(contentsSHA256[:]))
    if v, _ := persistentState.Get(ScriptStateBucket, scriptStateKey); v != nil {
        return true, nil  // 该哈希的脚本运行过了
    }

case ScriptConditionOnChange:
    entryStateKey := []byte(targetAbsPath.String())
    entryStateBytes, _ := persistentState.Get(EntryStateBucket, entryStateKey)
    var entryState EntryState
    stateFormat.Unmarshal(entryStateBytes, &entryState)
    if bytes.Equal(entryState.ContentsSHA256.Bytes(), contentsSHA256[:]) {
        return true, nil  // 自上次运行以来未变
    }
}
```

### 原理

| 模式 | 键 | 触发再次运行的条件 |
|---|---|---|
| `run_once_` | **脚本内容的 SHA-256** | 脚本内容变化(因为新 hash → 新 key) |
| `run_onchange_` | **脚本目标路径** | 路径相同但内容 hash 不同 |
| `run_` | — | 永远执行 |

`run_once_` 的精妙之处:它把哈希 **本身** 当 key,所以同一个 `run_once_` 脚本被改写后会以"另一个 once 任务"的身份再跑一次;`run_onchange_` 则用路径作 key 并存上次内容 hash,从而能区分"内容变了"。

执行顺序由 `before_` / `after_` 前缀映射到 `ScriptOrderBefore / During / After`,在 apply 主循环里分三批跑。

---

## 6. apply / diff — 文件系统抽象 (装饰器模式)

### 涉及文件
- `internal/chezmoi/system.go`
- `internal/chezmoi/realsystem.go` + `realsystem_unix.go` / `realsystem_windows.go`
- `internal/chezmoi/dryrunsystem.go` / `debugsystem.go` / `gitdiffsystem.go`
- `internal/chezmoi/readonlysystem.go` / `erroronwritesystem.go`
- `internal/chezmoi/archivereadersystem.go` / `tarwritersystem.go` / `zipwritersystem.go`
- `internal/chezmoi/externaldiffsystem.go` / `nullsystem.go` / `dumpsystem.go`

### 接口节选

```go
type System interface {
    Chmod(name AbsPath, mode fs.FileMode) error
    Lstat(filename AbsPath) (fs.FileInfo, error)
    Mkdir(name AbsPath, perm fs.FileMode) error
    ReadDir(name AbsPath) ([]fs.DirEntry, error)
    ReadFile(name AbsPath) ([]byte, error)
    Readlink(name AbsPath) (string, error)
    Remove(name AbsPath) error; RemoveAll(name AbsPath) error
    Rename(oldPath, newPath AbsPath) error
    RunCmd(cmd *exec.Cmd) error
    RunScript(scriptName RelPath, dir AbsPath, data []byte, options RunScriptOptions) error
    WriteFile(filename AbsPath, data []byte, perm fs.FileMode) error
    WriteSymlink(oldName string, newName AbsPath) error
    UnderlyingFS() vfs.FS
}
```

### 装饰器矩阵

| 实现 | 作用 | 实现要点 |
|---|---|---|
| `RealSystem` | 真实落地 | 包 `go-vfs` 文件系统;通过 `google/renameio` 做 **原子写** |
| `DryRunSystem` | 模拟 | 读操作透传;所有写操作只 `setModified()` 然后返回 nil |
| `DebugSystem` | 日志 | 调用 `slog.Log` 记录方法名 + 参数后透传 |
| `GitDiffSystem` | 生成 unified diff | 拦截写操作,调用 `diff` 库输出 |
| `ReadOnlySystem` / `ErrorOnWriteSystem` | 安全保护 | 写操作直接报错 |
| `TarWriterSystem` / `ZipWriterSystem` | `chezmoi archive` | 把目标文件系统映射成 archive 流 |
| `ArchiveReaderSystem` | `.chezmoiexternal` | 把 tar/zip 当源文件系统访问 |

`emptySystemMixin` / `noUpdateSystemMixin` 两个 mixin 让装饰器只覆写需要的方法。

### 原理

整个 apply 主循环是 **System 无关的**——`chezmoi diff` 实际上等于 `apply` 跑在 `DryRunSystem(GitDiffSystem(realSystem))` 上。这种设计让 dry-run、diff、archive、debug 全部 **复用同一份 reconcile 逻辑**,几乎没有重复代码。

---

## 7. 密码管理器 / 秘密集成

### 涉及文件(均在 `internal/cmd/`)

`bitwardentemplatefuncs.go` / `bitwardensecretstemplatefuncs.go` / `onepasswordtemplatefuncs.go` / `keepassxctemplatefuncs.go` / `keepertemplatefuncs.go` / `lastpasstemplatefuncs.go` / `passtemplatefuncs.go` / `gopasstemplatefuncs.go` / `passholetemplatefuncs.go` / `protonpasstemplatefuncs.go` / `rbwtemplatefuncs.go` / `vaulttemplatefuncs.go` / `awssecretsmanagertemplatefuncs.go` / `azurekeyvaulttemplatefuncs.go` / `dopplertemplatefuncs.go` / `ejsontemplatefuncs.go` / `dashlanetemplatefuncs.go` / `keyringtemplatefuncs.go` / `githubtemplatefuncs.go` / `secrettemplatefuncs.go`

### 通用模式 (Bitwarden 为例)

```go
type bitwardenConfig struct {
    Command     string
    Unlock      autoBool
    session     string
    outputCache map[string][]byte           // 进程内缓存
}

func (c *Config) bitwardenTemplateFunc(args ...string) map[string]any {
    must(c.bitwardenMaybeUnlock())
    output := mustValue(c.bitwardenOutput(append([]string{"get"}, args...)))
    var data map[string]any
    must(json.Unmarshal(output, &data))
    return data
}

func (c *Config) bitwardenOutput(args []string) ([]byte, error) {
    key := strings.Join(args, "\x00")        // argv 序列化为 cache key
    if data, ok := c.Bitwarden.outputCache[key]; ok {
        return data, nil
    }
    output, err := c.bitwardenUncachedOutput(args)
    if err != nil { return nil, err }
    if c.Bitwarden.outputCache == nil {
        c.Bitwarden.outputCache = make(map[string][]byte)
    }
    c.Bitwarden.outputCache[key] = output
    return output, nil
}
```

### 原理

每个秘密函数遵循同一套三步流水线:

1. **解锁**:复用 `$BW_SESSION` 之类的环境变量,缺失时调用 `bw unlock --raw` 一次,token 缓存进 `Config.Bitwarden.session`
2. **执行 CLI**:`os/exec` 调用对应 vendor 命令
3. **JSON 解码 + 进程内缓存**:以 `argv` 拼接字符串作 key 缓存输出,使得同一份秘密在多个模板中只 fork 一次进程

`must` / `mustValue` 用 panic 透出错误,在模板执行边界被 `recover` 转换为模板错误。少数函数(GitHub API、1Password)还叠加了 TTL 缓存。

---

## 8. init / bootstrap 流程

### 涉及文件
- `internal/cmd/initcmd.go`(356 行)

### 流程(`runInitCmd`)

1. **`oneShot` 模式** 自动展开 `force + apply + depth=1 + purge + purgeBinary`,服务于 `curl | sh` 一键安装场景
2. 若 `~/.local/share/chezmoi/.git` 不存在:
   - 无参数:`git init` 或内置 `go-git`
   - 有 repo 参数:`guessRepoURL` 通过一组正则把 `user`、`user/repo`、`sr.ht/~user`、`gitlab.com/...` 等简写补全成完整 URL,然后 clone:

```go
args := []string{"clone"}
if c.init.recurseSubmodules { args = append(args, "--recurse-submodules") }
if c.init.branch != ""      { args = append(args, "--branch", c.init.branch) }
if c.init.depth  != 0       { args = append(args, "--depth", strconv.Itoa(c.init.depth)) }
args = append(args, repoURLStr, workingTreeRawPath.String())
c.run(chezmoi.EmptyAbsPath, c.Git.Command, args)
```

3. `checkVersion` 校验仓库的 `.chezmoiversion` 与 binary 兼容
4. `createAndReloadConfigFile` 处理 `.chezmoi.toml.tmpl` —— 这是 `promptString` / `promptBool` / `promptChoice` 等交互式模板函数的舞台 —— 然后写出最终 `chezmoi.toml`
5. 可选 `git lfs pull`
6. `--apply`:跑一遍 `applyArgs`(和 `chezmoi apply` 同一代码路径)
7. `--purge` / `--purge-binary`:删除源目录、缓存、persistent state、配置,最后甚至自删 binary

### 原理

`UseBuiltinGit` 让 init 在 **没有 git 的全新机器** 上也能跑通——这是 chezmoi 单二进制 bootstrap 承诺的核心保障。同时把"clone → 提示用户 → 写配置 → apply"压扁成一条线性流水线,使新机器 `sh -c "$(curl ...)" -- init --apply user` 真正可行。

---

## 9. 持久化状态 (bbolt)

### 涉及文件
- `internal/chezmoi/boltpersistentstate.go`
- `internal/chezmoi/persistentstate.go`

### Bucket 列表

```go
ConfigStateBucket          = []byte("configState")
EntryStateBucket           = []byte("entryState")
GitRepoExternalStateBucket = []byte("gitRepoExternalState")
ScriptStateBucket          = []byte("scriptState")
```

### 关键设计点

```go
switch _, err := system.Stat(path); {
case errors.Is(err, fs.ErrNotExist):
    empty = true   // bbolt 不能以 read-only 打开不存在的文件
}
options := bbolt.Options{
    OpenFile: func(name string, flag int, perm fs.FileMode) (*os.File, error) {
        rawPath, _ := system.RawPath(NewAbsPath(name))
        dir, _ := filepath.Split(rawPath.String())
        os.MkdirAll(dir, 0o777)
        return os.OpenFile(rawPath.String(), flag, perm)
    },
    ReadOnly: mode == BoltPersistentStateReadOnly,
    Timeout:  time.Second,    // flock 超时
}
```

### 原理

- **底层**:`bbolt`(etcd 维护的 boltdb fork),纯 Go 单文件 KV,B+Tree + mmap + 写时复制 + 全局事务,天然支持崩溃一致性
- **写**:`db.Update(tx → CreateBucketIfNotExists(bucket).Put(key, value))`
- **读**:`db.View` 中读出的 byte slice 只在事务内有效,所以代码总是 **clone** 一份再返回
- **read-only 兼容**:bbolt 不允许 read-only 打开不存在的 DB,chezmoi 用 `empty=true` 标志在前面短路所有读
- **并发**:1 秒 flock 超时,允许并发的 chezmoi 进程礼貌等待
- **装饰器同样有效**:`MockPersistentState`(测试)、`NullPersistentState`(完全无状态)、`DebugPersistentState`(日志)、`ReadOnlyPersistentState`(保护 diff)

它存的是什么?
- `EntryStateBucket`: 每个 target 路径上次写入时的 `EntryState`(类型、mode、内容 SHA-256),用于 **三方冲突检测**
- `ScriptStateBucket`: `run_once_` 已运行过的脚本 hash → `{Name, RunAt}`
- `GitRepoExternalStateBucket`: `.chezmoiexternal` 引用的 git 仓库的最后同步信息
- `ConfigStateBucket`: 配置文件本身的状态(检测配置变更触发提示)

---

## 10. 跨子系统的几个共性原则

| 原则 | 体现 |
|---|---|
| **仓库即真相,文件名即元数据** | 所有属性靠 `parseFileAttr` 编码,无 sidecar manifest |
| **三层状态分离** | Source(解析)→ Target(渲染/解密)→ Actual(磁盘)三层互不耦合 |
| **装饰器无处不在** | `System`、`Encryption`、`PersistentState` 都有 Real / Mock / ReadOnly / Debug / Null 装饰器矩阵 |
| **lazy + 缓存** | 模板渲染、秘密管理器调用、bbolt 都是 lazy + 进程内缓存 |
| **可测试性** | 抽象 FS + 抽象 PersistentState + golden test(`chezmoitest` / `archivetest`)使跨平台行为高度可验证 |
| **零外部依赖能跑通** | go-git、内置 age、纯 Go bbolt 都是可选启用的"无外部 binary"实现 |

---

## 11. 一个完整 `chezmoi apply` 的调用链(把上述模块串起来)

1. `cmd/applycmd.go` → `Config.runApplyCmd`
2. 打开 `BoltPersistentState`(若是 dry-run 包一层 `ReadOnlyPersistentState`)
3. 构造 `System` 栈:`RealSystem` → 视情况包 `DryRunSystem` / `GitDiffSystem` / `DebugSystem`
4. `SourceState.Read` 读取 `~/.local/share/chezmoi`,期间执行 `parseFileAttr` 构树
5. 对每个 target 路径调用 `SourceState.Apply`:
   - `sourceStateEntry.TargetStateEntry()` —— 这里触发 **模板渲染**(template.go)与 **解密**(encryption.go)
   - 计算 `EntryState`(含 SHA-256)
   - `SkipApply` 查 `ScriptStateBucket` / `EntryStateBucket` 决定是否跳过(`run_once_` / `run_onchange_` 幂等)
   - `preApplyFunc` 用三方比较检测人为修改
   - `targetStateEntry.Apply(system, ...)` 通过 `System` 接口写入(实际写哪儿取决于装饰器栈)
   - 写回新的 `EntryState` 到 `EntryStateBucket`
6. 脚本类 entry 按 `before / during / after` 顺序由 `RealSystem.RunScript` 调用解释器执行

整条流水线没有任何一处直接 `os.WriteFile` 或硬编码 `git`/`age`/`gpg` 命令路径——这就是 chezmoi 工程质量的来源:**所有副作用都流过抽象接口**,从而 dry-run、diff、archive、test 这些"另类用途"都可以零成本接入。

---

## 12. 推荐阅读切入点

如果你想自己读源码,建议按下面的顺序进入:

1. `internal/chezmoi/chezmoi.go` —— 看常量,建立词汇表
2. `internal/chezmoi/attr.go` —— 理解文件名 → 属性的全部规则
3. `internal/chezmoi/system.go` + `realsystem.go` + `dryrunsystem.go` —— 理解装饰器骨架
4. `internal/chezmoi/sourcestateentry.go` + `targetstateentry.go` —— 理解三层状态接口
5. `internal/chezmoi/sourcestate.go` 的 `Read` 与 `Apply` —— 主循环
6. `internal/chezmoi/boltpersistentstate.go` —— 看持久化是怎么挂上去的
7. `internal/cmd/initcmd.go` + `internal/cmd/applycmd.go` —— CLI 层如何把上述东西编排起来
8. `internal/cmd/bitwardentemplatefuncs.go` —— 一个完整的秘密管理器集成模板

掌握以上 8 个文件,你基本就掌握了 chezmoi 90% 的设计。
