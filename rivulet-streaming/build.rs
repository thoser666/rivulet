fn main() {
    #[cfg(target_os = "windows")]
    {
        // vcpkg will automatically find FFmpeg
        vcpkg::Config::new()
            .emit_includes(true)
            .probe("ffmpeg")
            .unwrap();
    }
}