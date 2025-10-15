#[cfg(windows)]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/rivulet_logo.ico");
    res.set("ProductName", "Rivulet");
    res.set("FileDescription", "Screen Recording & Streaming");
    res.set("CompanyName", "Rivulet Team");
    res.compile().unwrap();
}

#[cfg(not(windows))]
fn main() {
    // Nichts zu tun auf anderen Plattformen
}
