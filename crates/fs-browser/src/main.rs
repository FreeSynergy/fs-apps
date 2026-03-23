fn main() {
    #[cfg(feature = "desktop")]
    fs_components::launch_desktop(
        fs_components::DesktopConfig::new()
            .with_title("FSN Browser")
            .with_size(1100.0, 750.0)
            .with_all_navigation(),
        fs_browser::BrowserApp,
    );
}
