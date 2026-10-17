/**
 * 关于页展示的固定信息。
 *
 * `APP_VERSION` 与 `package.json` / `src-tauri/tauri.conf.json` 里的版本号**是同一份**
 * 但这里手写了一遍 —— 前端拿不到那两个文件的版本号而不必引入构建期注入。
 * 改版本号时**三处一起改**，否则「关于」里显示的就是旧版本。
 */

export const APP_VERSION = "0.1.0";

/**
 * 联系方式。
 *
 * 不是词典 key：邮箱地址在三种语言下都一样，放进词典只会多三份同样的字符串。
 */
export const CONTACT_EMAIL = "aabb1029384756@qq.com";
