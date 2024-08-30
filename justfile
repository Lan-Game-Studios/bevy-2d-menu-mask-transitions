run:
    mold -run cargo run --example basic --features bevy/dynamic_linking,bevy/wayland,bevy/embedded_watcher

run-release:
    mold -run cargo run --example basic --features bevy/dynamic_linking,bevy/wayland,bevy/embedded_watcher --release
