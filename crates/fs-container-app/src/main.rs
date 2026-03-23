fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_container_app::Container);
}
