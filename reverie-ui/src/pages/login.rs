//! Login page

use crate::state::AuthState;
use dioxus::prelude::*;

/// Login page component
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

        // Validate inputs
        if username.read().is_empty() || password.read().is_empty() {
            error.set(Some("Please enter username and password".to_string()));
            loading.set(false);
            return;
        }

        // In production, this would make an API call to authenticate
        // For demo, accept any credentials
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
                    p { class: "text-gray-400", "Music Server" }
                }

                // Login form
                form {
                    class: "card p-8 space-y-6",
                    onsubmit: on_submit,

                    // Error message
                    if let Some(err) = error() {
                        div {
                            class: "bg-red-500/20 border border-red-500 text-red-400 px-4 py-3 rounded-lg",
                            "{err}"
                        }
                    }

                    // Server URL
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-300 mb-2",
                            "Server URL"
                        }
                        input {
                            class: "w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500",
                            r#type: "text",
                            placeholder: "http://localhost:4533/rest",
                            value: "{server_url}",
                            oninput: move |e| server_url.set(e.value())
                        }
                    }

                    // Username
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-300 mb-2",
                            "Username"
                        }
                        input {
                            class: "w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500",
                            r#type: "text",
                            placeholder: "Enter your username",
                            value: "{username}",
                            oninput: move |e| username.set(e.value())
                        }
                    }

                    // Password
                    div {
                        label {
                            class: "block text-sm font-medium text-gray-300 mb-2",
                            "Password"
                        }
                        input {
                            class: "w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500",
                            r#type: "password",
                            placeholder: "Enter your password",
                            value: "{password}",
                            oninput: move |e| password.set(e.value())
                        }
                    }

                    // Submit button
                    button {
                        class: "w-full btn-primary py-3 text-lg font-medium",
                        r#type: "submit",
                        disabled: loading(),
                        if loading() {
                            "Signing in..."
                        } else {
                            "Sign In"
                        }
                    }
                }

                // Footer
                p {
                    class: "text-center text-gray-500 text-sm mt-6",
                    "Reverie Music Server • Built with Rust & Dioxus"
                }
            }
        }
    }
}
