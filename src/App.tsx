/**
 * 应用外壳。
 *
 * 这里是**最小可挂载壳子**，只为让 Tauri v2 的 `generate_context!` 有前端产物可校验。
 * 真正的界面按任务逐个在 `features/` 下实现（accounts / inbox / compose / search / settings）。
 */
export default function App() {
  return (
    <main>
      <h1>dp_Gmail</h1>
      <p>本地优先跨平台桌面邮件客户端</p>
    </main>
  );
}
