//! Common UI components
//!
//! Shared components like buttons, inputs, loaders, etc.

use dioxus::prelude::*;

/// Loading spinner
#[component]
pub fn LoadingSpinner(#[props(default = "Loading...")] message: &'static str) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8",
            svg {
                class: "w-12 h-12 text-blue-500 animate-spin",
                fill: "none",
                view_box: "0 0 24 24",
                circle {
                    class: "opacity-25",
                    cx: "12",
                    cy: "12",
                    r: "10",
                    stroke: "currentColor",
                    stroke_width: "4"
                }
                path {
                    class: "opacity-75",
                    fill: "currentColor",
                    d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                }
            }
            p { class: "mt-4 text-gray-400", "{message}" }
        }
    }
}

/// Empty state placeholder
#[component]
pub fn EmptyState(
    title: String,
    #[props(default)] message: Option<String>,
    #[props(default)] icon: Option<String>,
) -> Element {
    let icon_path = icon.unwrap_or_else(|| {
        "M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10".to_string()
    });

    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-12 text-center",
            svg {
                class: "w-16 h-16 text-gray-600 mb-4",
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
            h3 { class: "text-xl font-medium text-gray-300 mb-2", "{title}" }
            if let Some(msg) = message {
                p { class: "text-gray-500", "{msg}" }
            }
        }
    }
}

/// Error display
#[component]
pub fn ErrorDisplay(message: String) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center p-8 text-center",
            svg {
                class: "w-12 h-12 text-red-500 mb-4",
                fill: "none",
                stroke: "currentColor",
                view_box: "0 0 24 24",
                path {
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    stroke_width: "2",
                    d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                }
            }
            p { class: "text-red-400", "{message}" }
        }
    }
}

/// Badge component
#[component]
pub fn Badge(
    text: String,
    #[props(default = "gray")] color: &'static str,
) -> Element {
    let class = match color {
        "blue" => "px-2 py-0.5 text-xs rounded-full bg-blue-600 text-white",
        "green" => "px-2 py-0.5 text-xs rounded-full bg-green-600 text-white",
        "red" => "px-2 py-0.5 text-xs rounded-full bg-red-600 text-white",
        "yellow" => "px-2 py-0.5 text-xs rounded-full bg-yellow-600 text-white",
        _ => "px-2 py-0.5 text-xs rounded-full bg-gray-600 text-white",
    };

    rsx! {
        span { class: "{class}", "{text}" }
    }
}

/// Icon button with tooltip
#[component]
pub fn IconButton(
    icon: String,
    #[props(default)] tooltip: Option<String>,
    #[props(default)] active: bool,
    onclick: EventHandler<MouseEvent>,
) -> Element {
    let class = if active {
        "btn-icon text-blue-500"
    } else {
        "btn-icon text-gray-400 hover:text-white"
    };

    rsx! {
        button {
            class: "{class}",
            title: "{tooltip.unwrap_or_default()}",
            onclick: move |e| onclick.call(e),
            svg {
                class: "w-5 h-5",
                fill: "currentColor",
                view_box: "0 0 24 24",
                path {
                    d: "{icon}"
                }
            }
        }
    }
}

/// Dropdown menu
#[component]
pub fn DropdownMenu(
    trigger: Element,
    #[props(default = false)] open: bool,
    children: Element,
) -> Element {
    let mut is_open = use_signal(|| open);

    rsx! {
        div {
            class: "relative",
            div {
                onclick: move |_| is_open.set(!is_open()),
                {trigger}
            }
            if is_open() {
                div {
                    class: "dropdown-menu top-full right-0 mt-1",
                    onclick: move |_| is_open.set(false),
                    {children}
                }
            }
        }
    }
}

/// Tab bar
#[component]
pub fn TabBar(
    tabs: Vec<String>,
    active_index: usize,
    on_change: EventHandler<usize>,
) -> Element {
    rsx! {
        div {
            class: "flex border-b border-gray-700 mb-4",
            for (idx, tab) in tabs.iter().enumerate() {
                button {
                    key: "{idx}",
                    class: if idx == active_index {
                        "px-4 py-2 border-b-2 border-blue-500 text-white"
                    } else {
                        "px-4 py-2 text-gray-400 hover:text-white"
                    },
                    onclick: move |_| on_change.call(idx),
                    "{tab}"
                }
            }
        }
    }
}

/// Pagination component
#[component]
pub fn Pagination(
    current_page: usize,
    total_pages: usize,
    on_page_change: EventHandler<usize>,
) -> Element {
    if total_pages <= 1 {
        return rsx! {};
    }

    rsx! {
        div {
            class: "flex items-center justify-center gap-2 mt-6",
            
            // Previous button
            button {
                class: "btn-secondary",
                disabled: current_page == 1,
                onclick: move |_| {
                    if current_page > 1 {
                        on_page_change.call(current_page - 1);
                    }
                },
                "Previous"
            }
            
            // Page numbers
            span {
                class: "text-gray-400 px-4",
                "Page {current_page} of {total_pages}"
            }
            
            // Next button
            button {
                class: "btn-secondary",
                disabled: current_page == total_pages,
                onclick: move |_| {
                    if current_page < total_pages {
                        on_page_change.call(current_page + 1);
                    }
                },
                "Next"
            }
        }
    }
}

/// Modal dialog
#[component]
pub fn Modal(
    title: String,
    open: bool,
    on_close: EventHandler<()>,
    children: Element,
) -> Element {
    if !open {
        return rsx! {};
    }

    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "modal-content",
                onclick: |e| e.stop_propagation(),
                
                // Header
                div {
                    class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xl font-bold", "{title}" }
                    button {
                        class: "btn-icon text-gray-400 hover:text-white",
                        onclick: move |_| on_close.call(()),
                        svg {
                            class: "w-6 h-6",
                            fill: "none",
                            stroke: "currentColor",
                            view_box: "0 0 24 24",
                            path {
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                stroke_width: "2",
                                d: "M6 18L18 6M6 6l12 12"
                            }
                        }
                    }
                }
                
                // Content
                {children}
            }
        }
    }
}

/// Format duration in seconds to MM:SS
pub fn format_duration(seconds: i32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{}:{:02}", mins, secs)
}

/// Format duration in seconds to human readable string
pub fn format_duration_long(seconds: i32) -> String {
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    
    if hours > 0 {
        format!("{} hr {} min", hours, mins)
    } else {
        format!("{} min", mins)
    }
}
