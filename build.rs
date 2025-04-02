fn main() {
    relm4_icons_build::bundle_icons(
        // Name of the file that will be generated at `OUT_DIR`
        "icon_names.rs",
        // Optional app ID
        Some("mullvadwaita"),
        // Custom base resource path:
        // * defaults to `/com/example/myapp` in this case if not specified explicitly
        // * or `/org/relm4` if app ID was not specified either
        None::<&str>,
        // Directory with custom icons (if any)
        Some("src/res/icons"),
        // List of icons to include
        [
            "copy",
            "info-outline",
            "wired-lock-closed",
            "wired-lock-none",
            "wired-lock-open",
            "arrow2-right",
            "arrow-circular-top-right",
            "cross-large-circle-filled",
            "menu-large",
            "network-workgroup",
            "globe-alt2",
            "shield-full",
            "stop-sign-large",
            "edit",
            "background-app-ghost",
            "issue",
            "function-third-order-horizontal",
            "warning-outline",
        ],
    );
}
