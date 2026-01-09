//! 设置页面

use crate::components::PageHeader;
use dioxus::prelude::*;

/// 设置页面组件
#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div {
            class: "space-y-6 max-w-2xl",

            PageHeader {
                title: "设置".to_string()
            }

            // 通用设置
            section {
                class: "card p-6 space-y-4",
                h2 { class: "text-lg font-bold mb-4", "通用" }

                // 主题
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "主题" }
                        p { class: "text-sm text-gray-400", "选择您偏好的颜色方案" }
                    }
                    select {
                        class: "bg-gray-700 border border-gray-600 rounded-lg px-3 py-2",
                        option { value: "dark", "深色" }
                        option { value: "light", "浅色" }
                        option { value: "system", "跟随系统" }
                    }
                }

                // 语言
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "语言" }
                        p { class: "text-sm text-gray-400", "选择您偏好的语言" }
                    }
                    select {
                        class: "bg-gray-700 border border-gray-600 rounded-lg px-3 py-2",
                        option { value: "en", "English" }
                        option { value: "zh", "中文" }
                        option { value: "ja", "日本語" }
                    }
                }
            }

            // 播放设置
            section {
                class: "card p-6 space-y-4",
                h2 { class: "text-lg font-bold mb-4", "播放" }

                // 淡入淡出
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "淡入淡出" }
                        p { class: "text-sm text-gray-400", "歌曲之间的平滑过渡" }
                    }
                    input {
                        r#type: "checkbox",
                        class: "w-5 h-5 accent-blue-500"
                    }
                }

                // 无缝播放
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "无缝播放" }
                        p { class: "text-sm text-gray-400", "播放专辑时无间隔" }
                    }
                    input {
                        r#type: "checkbox",
                        class: "w-5 h-5 accent-blue-500",
                        checked: true
                    }
                }

                // 响度增益
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "响度增益" }
                        p { class: "text-sm text-gray-400", "跨曲目标准化音量" }
                    }
                    select {
                        class: "bg-gray-700 border border-gray-600 rounded-lg px-3 py-2",
                        option { value: "none", "关闭" }
                        option { value: "track", "曲目" }
                        option { value: "album", "专辑" }
                    }
                }
            }

            // 关于部分
            section {
                class: "card p-6",
                h2 { class: "text-lg font-bold mb-4", "关于" }

                div {
                    class: "space-y-2 text-gray-400",
                    p { "Reverie 音乐服务器" }
                    p { "版本: 0.1.0" }
                    p { "基于 Dioxus & Rust 构建" }

                    div {
                        class: "mt-4 pt-4 border-t border-gray-700",
                        a {
                            class: "text-blue-400 hover:underline",
                            href: "https://github.com/your-repo/reverie",
                            target: "_blank",
                            "GitHub 仓库"
                        }
                    }
                }
            }
        }
    }
}
