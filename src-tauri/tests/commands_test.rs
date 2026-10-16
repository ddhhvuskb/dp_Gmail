//! `commands` 层的参数校验测试。
//!
//! 命令本体（`#[tauri::command]`）需要完整的 Tauri 运行时才能调用，
//! 但其**参数校验**部分是纯函数，可独立验证 —— 这也是命令层唯一有逻辑的地方。

use dp_gmail::commands::messages::{normalize_paging, MAX_PAGE_SIZE};

#[test]
fn clamps_limit_down_to_upper_bound() {
    assert_eq!(
        normalize_paging(u32::MAX, 0).0,
        MAX_PAGE_SIZE,
        "超大 limit 必须被压到上限，否则会把整个文件夹读进内存"
    );
}

#[test]
fn clamps_limit_up_to_one() {
    assert_eq!(normalize_paging(0, 0).0, 1, "limit 为 0 应归一为 1");

    // limit = 1 本就是下界，不应被改动
    assert_eq!(normalize_paging(1, 0).0, 1);
}

#[test]
fn passes_through_valid_limit() {
    assert_eq!(normalize_paging(50, 100), (50, 100));
    assert_eq!(normalize_paging(MAX_PAGE_SIZE, 0).0, MAX_PAGE_SIZE);
}

#[test]
fn keeps_offset_untouched() {
    assert_eq!(
        normalize_paging(10, u32::MAX).1,
        u32::MAX,
        "offset 透传，不做钳制"
    );
    assert_eq!(normalize_paging(10, 0).1, 0);
}
