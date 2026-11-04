use std::cell::Cell;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use webkit2gtk::{CookieManagerExt, WebContext, WebContextExt, WebView, WebViewExt};

pub fn run() -> i32 {
    let mut out = None;
    let mut url = "https://www.roblox.com/login".to_owned();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().map(PathBuf::from),
            "--url" => {
                if let Some(value) = args.next() {
                    url = value;
                }
            }
            _ => {}
        }
    }
    let Some(out) = out else {
        eprintln!("login-helper: --out is required");
        return 2;
    };

    if gtk::init().is_err() {
        eprintln!("login-helper: could not start GTK");
        return 2;
    }

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Roblox Sign In");
    window.set_default_size(1000, 800);
    window.set_position(gtk::WindowPosition::Center);
    window.connect_destroy(|_| gtk::main_quit());

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let title = gtk::Label::new(Some(
        "Sign in with your Roblox account. This window closes when sign-in completes.",
    ));
    title.set_margin_top(10);
    title.set_margin_bottom(4);
    title.set_margin_start(14);
    title.set_margin_end(14);
    title.set_xalign(0.0);
    let note = gtk::Label::new(Some(
        "The browser session is temporary and is cleared when this window closes.",
    ));
    note.set_margin_bottom(10);
    note.set_margin_start(14);
    note.set_margin_end(14);
    note.set_xalign(0.0);
    container.pack_start(&title, false, false, 0);
    container.pack_start(&note, false, false, 0);

    let context = WebContext::new_ephemeral();
    let webview = WebView::with_context(&context);
    webview.load_uri(&url);
    container.pack_start(&webview, true, true, 0);
    window.add(&container);
    window.show_all();
    window.present();

    let saved = Rc::new(Cell::new(false));
    let manager = context.cookie_manager().expect("cookie manager");
    let out_for_poll = out.clone();
    let saved_for_poll = saved.clone();
    glib::timeout_add_seconds_local(1, move || {
        let out = out_for_poll.clone();
        let saved = saved_for_poll.clone();
        manager.cookies(
            "https://www.roblox.com",
            None::<&gtk::gio::Cancellable>,
            move |result| {
                let Ok(mut cookies) = result else {
                    return;
                };
                let mut token = None;
                for cookie in cookies.iter_mut() {
                    if cookie.name().as_deref() == Some(".ROBLOSECURITY") {
                        token = cookie.value().map(|value| value.to_string());
                        break;
                    }
                }
                let Some(token) = token else {
                    return;
                };
                if token.is_empty() {
                    return;
                }
                if write_token(&out, &token).is_ok() {
                    saved.set(true);
                    gtk::main_quit();
                }
            },
        );
        glib::ControlFlow::Continue
    });

    gtk::main();
    if saved.get() { 0 } else { 1 }
}

fn write_token(path: &Path, token: &str) -> std::io::Result<()> {
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(token.as_bytes())?;
    file.sync_all()?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(())
}
