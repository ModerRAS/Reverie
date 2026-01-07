//! Layout components - Sidebar, AppBar, Main layout
//!
//! These components provide the main structure of the application.

use crate::state::{UiState, ViewType};
use dioxus::prelude::*;

/// Main application layout with sidebar, header, and content area
#[component]
pub fn MainLayout(children: Element) -> Element {
    let ui_state = use_context::<Signal<UiState>>();
    let sidebar_open = ui_state.read().sidebar_open;

    rsx! {
        div {
            class: "h-screen flex flex-col bg-gray-900 text-white overflow-hidden",

            // Top bar
            AppBar {}

            // Main content area
            div {
                class: "flex-1 flex overflow-hidden",

                // Sidebar
                Sidebar { open: sidebar_open }

                // Content
                main {
                    class: "flex-1 overflow-y-auto p-6",
                    {children}
                }
            }

            // Player bar at bottom
            super::player::PlayerBar {}
        }
    }
}

/// Application header bar
#[component]
pub fn AppBar() -> Element {
    let mut ui_state = use_context::<Signal<UiState>>();
    let mut search_query = use_signal(String::new);

    let toggle_sidebar = move |_| {
        let current = ui_state.read().sidebar_open;
        ui_state.write().sidebar_open = !current;
    };

    let on_search = move |evt: Event<FormData>| {
        let query = evt.value().clone();
        search_query.set(query.clone());
        ui_state.write().search_query = query;
        if !ui_state.read().search_query.is_empty() {
            ui_state.write().current_view = ViewType::Search;
        }
    };

    rsx! {
        header {
            class: "h-16 bg-gray-800 border-b border-gray-700 flex items-center px-4 gap-4",

            // Menu toggle button
            button {
                class: "btn-icon text-gray-400 hover:text-white",
                onclick: toggle_sidebar,
                svg {
                    class: "w-6 h-6",
                    fill: "none",
                    stroke: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M4 6h16M4 12h16M4 18h16"
                    }
                }
            }

            // Logo
            div {
                class: "flex items-center gap-2",
                svg {
                    class: "w-8 h-8 text-blue-500",
                    fill: "currentColor",
                    view_box: "0 0 24 24",
                    path {
                        d: "M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"
                    }
                }
                span {
                    class: "text-xl font-bold",
                    "Reverie"
                }
            }

            // Search bar
            div {
                class: "flex-1 max-w-xl mx-4",
                input {
                    class: "search-input",
                    r#type: "text",
                    placeholder: "Search music...",
                    value: "{search_query}",
                    oninput: on_search
                }
            }

            // User menu
            div {
                class: "flex items-center gap-2",
                button {
                    class: "btn-icon text-gray-400 hover:text-white",
                    svg {
                        class: "w-6 h-6",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                        }
                    }
                }
                button {
                    class: "btn-icon text-gray-400 hover:text-white",
                    svg {
                        class: "w-6 h-6",
                        fill: "none",
                        stroke: "currentColor",
                        view_box: "0 0 24 24",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            stroke_width: "2",
                            d: "M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                        }
                    }
                }
            }
        }
    }
}

/// Sidebar navigation
#[component]
pub fn Sidebar(open: bool) -> Element {
    let mut ui_state = use_context::<Signal<UiState>>();
    let current_view = ui_state.read().current_view.clone();

    let nav_items = vec![
        (ViewType::Albums, "Albums", "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"),
        (ViewType::Artists, "Artists", "M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z"),
        (ViewType::Songs, "Songs", "M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"),
        (ViewType::Playlists, "Playlists", "M4 6h16M4 10h16M4 14h16M4 18h16"),
        (ViewType::Favorites, "Favorites", "M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z"),
    ];

    let sidebar_class = if open {
        "w-64 bg-gray-800 border-r border-gray-700 flex flex-col transition-all duration-200"
    } else {
        "w-16 bg-gray-800 border-r border-gray-700 flex flex-col transition-all duration-200"
    };

    rsx! {
        aside {
            class: "{sidebar_class}",

            nav {
                class: "flex-1 py-4",

                for (view_type, label, icon_path) in nav_items {
                    {
                        let is_active = current_view == view_type;
                        let view_type_clone = view_type.clone();
                        let item_class = if is_active {
                            "sidebar-item active"
                        } else {
                            "sidebar-item"
                        };

                        rsx! {
                            button {
                                key: "{label}",
                                class: "{item_class} w-full",
                                onclick: move |_| {
                                    ui_state.write().current_view = view_type_clone.clone();
                                },
                                svg {
                                    class: "w-5 h-5 flex-shrink-0",
                                    fill: "none",
                                    stroke: "currentColor",
                                    view_box: "0 0 24 24",
                                    path {
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        stroke_width: "2",
                                        d: "{icon_path}"
                                    }
                                }
                                if open {
                                    span { class: "truncate", "{label}" }
                                }
                            }
                        }
                    }
                }
            }

            // Now playing mini info (when sidebar is open)
            if open {
                div {
                    class: "p-4 border-t border-gray-700",
                    button {
                        class: "sidebar-item w-full",
                        onclick: move |_| {
                            let current = ui_state.read().now_playing_panel_open;
                            ui_state.write().now_playing_panel_open = !current;
                        },
                        svg {
                            class: "w-5 h-5",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2"
                            }
                        }
                        span { "Now Playing" }
                    }
                }
            }
        }
    }
}

/// Page header with title and optional actions
#[component]
pub fn PageHeader(
    title: String,
    #[props(default)] subtitle: Option<String>,
    #[props(default)] children: Element,
) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-between mb-6",
            div {
                h1 { class: "text-3xl font-bold", "{title}" }
                if let Some(sub) = subtitle {
                    p { class: "text-gray-400 mt-1", "{sub}" }
                }
            }
            {children}
        }
    }
}
