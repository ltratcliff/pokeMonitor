use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use reqwest::header;
use scraper::{Html, Selector};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIconBuilder};

const CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);
const TIMER_INTERVAL: Duration = Duration::from_millis(100);

fn create_default_icon() -> Icon {
    // 16x16 green circle RGBA icon
    let size = 16u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let center = size as f32 / 2.0;
    let radius = center - 1.0;
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let idx = ((y * size + x) * 4) as usize;
            if dx * dx + dy * dy <= radius * radius {
                rgba[idx] = 0x00;     // R
                rgba[idx + 1] = 0xAA; // G
                rgba[idx + 2] = 0x00; // B
                rgba[idx + 3] = 0xFF; // A
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("failed to create icon")
}

async fn check_site() -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .build()?;

    let response = client
        .get("https://ltratcliff.com")
        .header(header::ACCEPT, "text/html")
        .send()
        .await?;

    let body = response.text().await?;
    let document = Html::parse_document(&body);

    let link_selector = Selector::parse("a[href]").unwrap();
    for link in document.select(&link_selector) {
        let text = link.text().collect::<String>();
        if text.trim().eq_ignore_ascii_case("about") {
            let href = link.value().attr("href").unwrap_or("");
            println!("Found About link: {}", href);
            return Ok(true);
        }
    }

    println!("No About link found.");
    Ok(false)
}

fn send_notification(title: &str, body: &str) {
    #[cfg(target_os = "macos")]
    {
        let escaped_body = body.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_title = title.replace('\\', "\\\\").replace('"', "\\\"");
        let script = format!(
            "display alert \"{}\" message \"{}\" giving up after 10",
            escaped_title, escaped_body,
        );
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = notify_rust::Notification::new()
            .summary(title)
            .body(body)
            .timeout(notify_rust::Timeout::Milliseconds(10_000))
            .show();
    }
}

enum BackgroundMsg {
    CheckResult(bool),
    Error(String),
}

fn main() {
    let event_loop = EventLoopBuilder::new().build();

    let menu = Menu::new();
    let quit_item = MenuItem::new("Quit", true, None);
    menu.append(&quit_item).unwrap();

    let _tray_icon = TrayIconBuilder::new()
        .with_tooltip("pokemonItor - monitoring ltratcliff.com")
        .with_icon(create_default_icon())
        .with_menu(Box::new(menu))
        .build()
        .expect("failed to build tray icon");

    let quit_id = quit_item.id().clone();

    // Channel for background -> main thread communication
    let (tx, rx) = mpsc::channel::<BackgroundMsg>();

    // Spawn the background checking thread with its own tokio runtime
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");
        loop {
            println!("Checking site...");
            match rt.block_on(check_site()) {
                Ok(found) => {
                    let _ = tx.send(BackgroundMsg::CheckResult(found));
                }
                Err(e) => {
                    let _ = tx.send(BackgroundMsg::Error(e.to_string()));
                }
            }
            thread::sleep(CHECK_INTERVAL);
        }
    });

    // Run the event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(
            std::time::Instant::now() + TIMER_INTERVAL,
        );

        // Check for menu events
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == quit_id {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Check for background messages
        if let Ok(msg) = rx.try_recv() {
            match msg {
                BackgroundMsg::CheckResult(found) => {
                    if found {
                        send_notification(
                            "pokemonItor",
                            "About page link detected on ltratcliff.com!",
                        );
                    }
                }
                BackgroundMsg::Error(e) => {
                    eprintln!("Error checking site: {}", e);
                    send_notification(
                        "pokemonItor - Error",
                        &format!("Failed to check site: {}", e),
                    );
                }
            }
        }

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("pokemonItor started. Monitoring ltratcliff.com every 30 minutes.");
            }
            _ => {}
        }
    });
}
