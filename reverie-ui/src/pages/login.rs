//! 登录页面

use crate::state::AuthState;
use dioxus::prelude::*;

/// 登录页面组件
#[component]
pub fn LoginPage() -> Element {
    let mut auth_state = use_context::<Signal<AuthState>>();
    let mut username = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut server_url = use_signal(|| "http://localhost:4533/rest".to_string());
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let navigator = use_navigator();

    let on_submit = move |evt: Event<FormData>| {
        evt.prevent_default();

        loading.set(true);
        error.set(None);

        // 验证输入
        if username.read().is_empty() || password.read().is_empty() {
            error.set(Some("请输入用户名和密码".to_string()));
            loading.set(false);
            return;
        }

        // 在生产环境中，这将调用 API 进行身份验证
        // 为了演示，接受任何凭据
        auth_state.write().is_authenticated = true;
        auth_state.write().username = username.read().clone();
        auth_state.write().server_url = server_url.read().clone();

        loading.set(false);
        navigator.push("/");
    };

    rsx! {
        div {
            class: "min-h-screen flex items-center justify-center bg-gray-900 px-4",

            div {
                class: "w-full max-w-md",

                // Logo
                div {
                    class: "text-center mb-8",
                    div {
                        class: "flex items-center justify-center gap-3 mb-4",
                        svg {
                            class: "w-12 h-12 text-blue-500",
                            fill: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                            }
                        }
                        h1 { class: "text-3xl font-bold text-white", "Reverie" }
                    }
                    p { class: "text-gray-400", "音乐服务器" }
                }

                // 登录表单
                form {
                    class: "card p-8 space-y-6",
                    onsubmit: on_submit,

                    // 错误消息
                    if let Some(err) = error() {
                        div {
                            class: "bg-red-500/20 border border-red-500 text-red-400 px-4 py-3 rounded-lg",
                            "{err}"
                        }
                    }

                    // 服务器地址
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-300 mb-2",
                            "服务器地址"
                        }
                        input {
                            class: "w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500",
                            r#type: "text",
                            placeholder: "http://localhost:4533/rest",
                            value: "{server_url}",
                            oninput: move |e| server_url.set(e.value())
                        }
                    }

                    // 用户名
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-300 mb-2",
                            "用户名"
                        }
                        input {
                            class: "w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500",
                            r#type: "text",
                            placeholder: "请输入用户名",
                            value: "{username}",
                            oninput: move |e| username.set(e.value())
                        }
                    }

                    // 密码
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-300 mb-2",
                            "密码"
                        }
                        input {
                            class: "w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500",
                            r#type: "password",
                            placeholder: "请输入密码",
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }
                    }

                    // 提交按钮
                    button {
                        class: "w-full btn-primary py-3 text-lg font-medium",
                        r#type: "submit",
                        disabled: loading(),
                        if loading() {
                            "正在登录..."
                        } else {
                            "登录"
                        }
                    }
                }

                // 页脚
                p {
                    class: "text-center text-gray-500 text-sm mt-6",
                    "Reverie 音乐服务器 • 基于 Rust & Dioxus 构建"
                }
            }
        }
    }
}
