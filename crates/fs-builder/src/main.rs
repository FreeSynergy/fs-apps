fn main() {
    #[cfg(feature = "desktop")]
    fs_components::launch_desktop(
        fs_components::DesktopConfig::new()
            .with_title("FreeSynergy \u{2014} Builder")
            .with_size(1100.0, 760.0),
        fs_builder::BuilderApp,
    );
}
