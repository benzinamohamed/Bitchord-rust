pub mod explore_view;
pub mod home_view;
pub mod library_view;
pub mod now_playing;
pub mod player_bar;
pub mod search_view;
pub mod settings_view;
pub mod sidebar;
pub mod theme;

pub use explore_view::ExploreView;
pub use home_view::HomeView;
pub use library_view::LibraryView;
pub use now_playing::NowPlayingView;
pub use player_bar::{PlayerBar, PlayerBarAction};
pub use search_view::SearchView;
pub use settings_view::{AppSettings, SettingsView};
pub use sidebar::{NavTab, Sidebar};
pub use theme::BitChordTheme;
