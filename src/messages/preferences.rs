use colony_ui::preferences::Change;

use crate::models::*;

/// What the Preferences page sends. Every change is saved as it is made.
#[derive(Debug, Clone)]
pub enum PreferencesMessage {
    /// Open the page, or close it and go back to the screen it was opened
    /// from. The identity button and the page's Close button both send it.
    Toggle,
    /// Show a category, by its index in the category list.
    Category(usize),
    /// Open or close a collapsible section, by its key.
    ToggleSection(&'static str),
    /// A change to the preferences every Colony program shares.
    Standard(Change),
    Language(Option<String>),
    DefaultStatus(GameStatus),
    DefaultPlatform(Platform),
    ConfirmDelete,
    StartScreen(StartScreen),
    CompactList,
    ShowCovers,
    ShowStatusLabels,
    LargeTargets,
    SidebarWidth(u16),
    Notifications,
    ToastDuration(ToastDuration),
    DateFormat(DateFormat),
    DefaultSortOrder(SortOrder),
    RememberFilters,
    ShowDescriptions,
    AchievementsDisplay(AchievementsDisplay),
    DownloadCoversAuto,
    Saved(Result<(), String>),
}
