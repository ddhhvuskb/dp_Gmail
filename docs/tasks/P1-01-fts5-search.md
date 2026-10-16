# P1-01 FTS5 全文检索

- **阶段**：P1
- **依赖**：P0-03（表结构）、`DESIGN.md` 锁定的「SQLite (rusqlite) + FTS5」
- **状态**：✅ 已完成并验证（2026-10-16）
- **关键约束**：`DESIGN.md` 验收总则「搜索 <200ms」

---

## 1. 交付物

| 层 | 内容 |
| --- | --- |
| `storage/schema.rs` | schema v3：FTS5 外部内容虚表 `messages_fts` + 三个同步触发器 |
| `storage/search_store.rs` | `search_messages(conn, query, limit)` |
| 迁移 | `INSERT INTO messages_fts (messages_fts) VALUES ('rebuild')` 回填历史邮件 |

## 2. 分词器选择：实测而非默认

**这是本任务最关键的决策，且结论来自实测。**

| 分词器 | 查「项目进度汇报」 | 查「项目」 | 查「进度」 |
| --- | --- | --- | --- |
| `unicode61`（FTS5 默认） | ✅ 命中 | ❌ 0 | ❌ 0 |
| `trigram` | ✅ 命中 | ❌ 0 | ❌ 0 |

- **`unicode61` 对中文是坏的**：它把一整串连续汉字当成**单个 token**，所以只有整串精确匹配才能命中。中文检索用它等于没做。
- **`trigram` 可用，但有下限**：它索引的是**三字符序列**，两字查询（`项目`、`进度`、`会议`）在 `MATCH` 下**必然零命中**。

所以 `tokenize='trigram'` 是必要但不充分的，必须配短查询回退。

### 2.1 短查询 LIKE 回退

查询字符数 < 3 时改走 `messages` 表上的 `LIKE` 子串匹配。

**这不是性能取舍，是正确性要求。** 中文里双字词（「项目」「进度」「会议」「发票」）恰恰最常见；
若硬走 `MATCH`，用户搜「项目」会得到空结果，而且**不会有任何报错** ——
他只会以为「这邮件没有」，永远不知道是分词器的限制。

回退路径走不了索引，但本地库规模（万级邮件）下仍是毫秒级。

## 3. 表结构与触发器

```sql
CREATE VIRTUAL TABLE messages_fts USING fts5(
    subject, from_name, from_address, to_addresses, text_body,
    content='messages', content_rowid='id', tokenize='trigram'
);
```

- **`content='messages'`（外部内容表）**：正文不会在 FTS 表里再存一份，省一半空间。查询时列值从 `messages` 读。
- **三个触发器**（`_ai` / `_ad` / `_au`）保持索引同步。外部内容表的 `'delete'` 指令**必须提供索引时的原值**，所以删除与更新一律传 `old.*`，否则索引会静默损坏。

## 4. 查询安全

| 风险 | 处理 |
| --- | --- |
| 用户输入 `-`、`*`、`(`、`:` 等 FTS5 语法字符 | 整体包成短语查询 `"…"`，内部双引号转义为 `""`。否则轻则 `fts5: syntax error`，重则语义完全变样 |
| 用户输入 `%` / `_` | LIKE 回退路径用 `ESCAPE '\'` 转义。不转义的话搜 `%` 会返回**全部邮件** |
| 超大 `limit` | 钳到 `1..=200` |

## 5. 验收结果

| 检查项 | 实测 |
| --- | --- |
| `cargo check --all-targets` | exit 0，零警告 ✅ |
| `cargo fmt --check` / `cargo clippy -D warnings` | exit 0 ✅ |
| `cargo test --test fts5_capability_test` | **5/5 通过** ✅ |
| `cargo test --test search_store_test` | **12/12 通过** ✅ |
| 全量测试 | **132/132 通过** ✅ |

### 5.1 关键性质已被测试固化

| 性质 | 用例 |
| --- | --- |
| FTS5 与 trigram 分词器可用（换 SQLite 版本会立刻暴露） | `fts5_is_available`、`trigram_tokenizer_is_available` |
| **unicode61 确实搜不到中文子串**（钉死弃用理由） | `unicode61_cannot_search_chinese_substrings` |
| **trigram 确实搜不到两字查询**（钉死回退的必要性） | `trigram_cannot_search_two_character_queries` |
| 三字及以上中文子串经 FTS 命中（主题与正文都算） | `searches_chinese_substring_through_fts` |
| 两字 / 单字查询经 LIKE 回退命中 | `falls_back_to_like_for_*` |
| 发件人姓名与地址可搜 | `searches_sender_address_and_name` |
| 插入即索引、更新换词、删除即消失 | `triggers_index_newly_inserted_messages`、`reflects_updates`、`reflects_deletes` |
| **`%` 不被当通配符** | `escapes_like_wildcards` |
| FTS5 语法字符不炸 | `survives_fts_syntax_characters` |
| limit 钳制、空查询、时间倒序 | `clamps_limit`、`empty_query_returns_nothing`、`orders_by_date_descending` |

### 5.2 本次改动打破的两个既有测试（已修正）

FTS5 会额外创建 4 张影子表（`messages_fts_data` / `_idx` / `_docsize` / `_config`），
于是「表数量 == 4」这类断言全碎了。已改为**断言四张基表存在**，
而不是断言精确总数 —— 否则以后每次动 FTS 实现都会连带炸一片测试。
另新增 `creates_fts_index_table` 守住虚表本身。

## 6. 尚未验证

- **「搜索 <200ms」这条验收标准未实测**。需要真实规模的库（万级邮件）才能量。
  LIKE 回退路径走全表扫描，是唯一有性能风险的分支。
- 前端搜索界面（P1 的 UI 部分）尚未实现。

## 7. 关于 schema 变更

本任务把 schema 从 v2 升到 v3（新增虚表与触发器），属**表结构变更**。
依据：`DESIGN.md` 第二章明确锁定了「SQLite (rusqlite) + FTS5」，
P1 范围也明写「FTS5 搜索」，故这是**实现已批准的设计**，不是额外引入。

迁移走既有的 `PRAGMA user_version` 机制，对老库自动回填索引，**不丢数据**。
