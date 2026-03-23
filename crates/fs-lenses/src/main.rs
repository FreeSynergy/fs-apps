fn main() {
    #[cfg(feature = "desktop")]
    fs_components::launch_desktop(
        fs_components::DesktopConfig::new()
            .with_title("FSN Lenses")
            .with_size(1000.0, 700.0),
        fs_lenses::LensesApp,
    );
}
