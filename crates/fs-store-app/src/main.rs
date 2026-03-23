fn main() {
    #[cfg(feature = "desktop")]
    dioxus::launch(fs_store_app::StoreApp);
}
