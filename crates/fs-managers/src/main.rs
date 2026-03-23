fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_managers::ManagersApp);
}
