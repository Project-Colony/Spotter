use crate::models::*;

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    ChangeSection(SettingsSection),
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
    SaveShown,
}
