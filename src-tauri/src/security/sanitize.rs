//! HTML 净化。
//!
//! 本模块是全项目**唯一**允许产出可渲染 HTML 的入口。
//! 前端不得直接持有未净化的 HTML，也不得使用 `dangerouslySetInnerHTML`
//! （见 `AGENTS.md` 禁止项）。

/// 净化邮件 HTML。
///
/// 策略（与 `DESIGN.md` 安全约束一致）：
///
/// - `<script>` / `<style>` 连**内容**一起删除；
/// - 事件处理属性（`onerror` / `onclick` …）一律剥离；
/// - 只放行安全 URL scheme，`javascript:` / `data:` 被丢弃；
/// - 不加载远程图片，避免追踪像素（P0 不做图片代理）。
///
/// 具体规则由 `ammonia` 的默认策略承担 —— 净化器属安全代码，
/// 不自造实现（见 `AGENTS.md` 禁止项）。
pub fn sanitize_html(html: &str) -> String {
    ammonia::clean(html)
}
