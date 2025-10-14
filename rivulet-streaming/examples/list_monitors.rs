use xcap::Monitor;

fn main() -> anyhow::Result<()> {
    println!("🖥️  Monitor Information\n");

    let monitors = Monitor::all()?;

    println!("Found {} monitor(s):\n", monitors.len());

    for (i, monitor) in monitors.iter().enumerate() {
        println!("Monitor {}:", i);
        println!("  Name:       {}", monitor.name());
        println!("  ID:         {}", monitor.id());
        println!("  Resolution: {}x{}", monitor.width(), monitor.height());
        println!("  Position:   ({}, {})", monitor.x(), monitor.y());
        println!("  Is Primary: {}", monitor.is_primary());
        println!();
    }

    Ok(())
}