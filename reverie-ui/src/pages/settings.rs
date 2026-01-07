//! Settings page

use crate::components::PageHeader;
use dioxus::prelude::*;

/// Settings page component
#[component]
pub fn SettingsPage() -> Element {
    rsx! {
        div {
            class: "space-y-6 max-w-2xl",

            PageHeader {
                title: "Settings".to_string()
            }

            // General settings
            section {
                class: "card p-6 space-y-4",
                h2 { class: "text-lg font-bold mb-4", "General" }

                // Theme
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "Theme" }
                        p { class: "text-sm text-gray-400", "Choose your preferred color scheme" }
                    }
                    select {
                        class: "bg-gray-700 border border-gray-600 rounded-lg px-3 py-2",
                        option { value: "dark", "Dark" }
                        option { value: "light", "Light" }
                        option { value: "system", "System" }
                    }
                }

                // Language
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "Language" }
                        p { class: "text-sm text-gray-400", "Select your preferred language" }
                    }
                    select {
                        class: "bg-gray-700 border border-gray-600 rounded-lg px-3 py-2",
                        option { value: "en", "English" }
                        option { value: "zh", "中文" }
                        option { value: "ja", "日本語" }
                    }
                }
            }

            // Playback settings
            section {
                class: "card p-6 space-y-4",
                h2 { class: "text-lg font-bold mb-4", "Playback" }

                // Crossfade
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "Crossfade" }
                        p { class: "text-sm text-gray-400", "Smooth transition between tracks" }
                    }
                    input {
                        r#type: "checkbox",
                        class: "w-5 h-5 accent-blue-500"
                    }
                }

                // Gapless playback
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "Gapless Playback" }
                        p { class: "text-sm text-gray-400", "Play albums without gaps" }
                    }
                    input {
                        r#type: "checkbox",
                        class: "w-5 h-5 accent-blue-500",
                        checked: true
                    }
                }

                // Replay Gain
                div {
                    class: "flex items-center justify-between",
                    div {
                        p { class: "font-medium", "Replay Gain" }
                        p { class: "text-sm text-gray-400", "Normalize volume across tracks" }
                    }
                    select {
                        class: "bg-gray-700 border border-gray-600 rounded-lg px-3 py-2",
                        option { value: "none", "None" }
                        option { value: "track", "Track" }
                        option { value: "album", "Album" }
                    }
                }
            }

            // About section
            section {
                class: "card p-6",
                h2 { class: "text-lg font-bold mb-4", "About" }

                div {
                    class: "space-y-2 text-gray-400",
                    p { "Reverie Music Server" }
                    p { "Version: 0.1.0" }
                    p { "Built with Dioxus & Rust" }

                    div {
                        class: "mt-4 pt-4 border-t border-gray-700",
                        a {
                            class: "text-blue-400 hover:underline",
                            href: "https://github.com/your-repo/reverie",
                            target: "_blank",
                            "GitHub Repository"
                        }
                    }
                }
            }
        }
    }
}
