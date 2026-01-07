//! Application routing
//!
//! Defines all routes and the main App component.

use crate::components::MainLayout;
use crate::pages::*;
use crate::state::{AppContext, AuthState, PlayerState, UiState, ViewType};
use dioxus::prelude::*;

/// Route definitions
#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/login")]
    Login {},
    
    #[layout(AppLayout)]
        #[route("/")]
        Home {},
        
        #[route("/albums")]
        Albums {},
        
        #[route("/album/:id")]
        AlbumDetail { id: String },
        
        #[route("/artists")]
        Artists {},
        
        #[route("/artist/:id")]
        ArtistDetail { id: String },
        
        #[route("/songs")]
        Songs {},
        
        #[route("/playlists")]
        Playlists {},
        
        #[route("/playlist/:id")]
        PlaylistDetail { id: String },
        
        #[route("/favorites")]
        Favorites {},
        
        #[route("/search")]
        Search {},
        
        #[route("/settings")]
        Settings {},
    #[end_layout]
    
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

/// Main application component
#[component]
pub fn App() -> Element {
    // Initialize global state
    use_context_provider(|| Signal::new(AuthState::default()));
    use_context_provider(|| {
        Signal::new(PlayerState {
            volume: 0.8,
            ..Default::default()
        })
    });
    use_context_provider(|| {
        Signal::new(UiState {
            sidebar_open: true,
            ..Default::default()
        })
    });

    rsx! {
        Router::<Route> {}
    }
}

/// App layout wrapper (for authenticated routes)
#[component]
fn AppLayout() -> Element {
    rsx! {
        MainLayout {
            Outlet::<Route> {}
        }
    }
}

/// Login route component
#[component]
fn Login() -> Element {
    rsx! { LoginPage {} }
}

/// Home route component
#[component]
fn Home() -> Element {
    rsx! { HomePage {} }
}

/// Albums route component
#[component]
fn Albums() -> Element {
    rsx! { AlbumsPage {} }
}

/// Album detail route component
#[component]
fn AlbumDetail(id: String) -> Element {
    rsx! { AlbumDetailPage { id: id } }
}

/// Artists route component
#[component]
fn Artists() -> Element {
    rsx! { ArtistsPage {} }
}

/// Artist detail route component
#[component]
fn ArtistDetail(id: String) -> Element {
    rsx! { ArtistDetailPage { id: id } }
}

/// Songs route component
#[component]
fn Songs() -> Element {
    rsx! { SongsPage {} }
}

/// Playlists route component
#[component]
fn Playlists() -> Element {
    rsx! { PlaylistsPage {} }
}

/// Playlist detail route component
#[component]
fn PlaylistDetail(id: String) -> Element {
    rsx! { PlaylistDetailPage { id: id } }
}

/// Favorites route component
#[component]
fn Favorites() -> Element {
    rsx! { FavoritesPage {} }
}

/// Search route component
#[component]
fn Search() -> Element {
    rsx! { SearchPage {} }
}

/// Settings route component
#[component]
fn Settings() -> Element {
    rsx! { SettingsPage {} }
}

/// 404 Not Found component
#[component]
fn NotFound(route: Vec<String>) -> Element {
    rsx! {
        div {
            class: "min-h-screen flex flex-col items-center justify-center bg-gray-900 text-white",
            h1 { class: "text-6xl font-bold text-gray-500", "404" }
            p { class: "text-xl text-gray-400 mt-4", "Page not found" }
            p { class: "text-gray-500 mt-2", "The page /{route.join(\"/\")} does not exist." }
            Link {
                to: Route::Home {},
                class: "mt-6 btn-primary",
                "Go Home"
            }
        }
    }
}
