// 進捗状況を表示するウィンドウ（7zip風ダイアログ）
// クロスプラットフォーム対応: fltk-rs を利用

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::Receiver;
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window, group::Pack, misc::Progress};

pub struct ProgressWindow {
    app: app::App,
    window: Window,
    progress_bar: Progress,
    label: Frame,
    #[allow(dead_code)]
    cancel_btn: Button,
    cancelled: Arc<AtomicBool>,
}

pub struct ProgressMsg {
    pub value: f32,
    pub text: String,
    pub done: bool,
}

impl ProgressWindow {
    pub fn new(title: &str) -> Self {
        let app = app::App::default();
        let mut window = Window::new(300, 300, 400, 120, title);
        let mut pack = Pack::new(20, 10, 360, 90, "");
        pack.set_spacing(10);
        let label = Frame::new(0, 0, 360, 20, "");
        let mut progress_bar = Progress::new(0, 0, 360, 20, "");
        progress_bar.set_selection_color(fltk::enums::Color::Green);
        progress_bar.set_color(fltk::enums::Color::White);
        progress_bar.set_minimum(0.0);
        progress_bar.set_maximum(1.0);
        progress_bar.set_value(0.0);
        let mut cancel_btn = Button::new(140, 0, 80, 30, "キャンセル");
        pack.end();
        window.end();
        window.show();
        let cancelled = Arc::new(AtomicBool::new(false));
        // キャンセルボタン押下時のイベント
        {
            let cancelled_clone = Arc::clone(&cancelled);
            cancel_btn.set_callback(move |_| {
                cancelled_clone.store(true, Ordering::SeqCst);
            });
        }
        // ウィンドウのクローズイベントでキャンセルフラグを立てる
        {
            let cancelled_clone = Arc::clone(&cancelled);
            window.set_callback(move |_| {
                cancelled_clone.store(true, Ordering::SeqCst);
                app::quit();
            });
        }
        Self {
            app,
            window,
            progress_bar,
            label,
            cancel_btn,
            cancelled,
        }
    }
    pub fn set_progress(&mut self, value: f32, text: &str) {
        self.progress_bar.set_value(value as f64);
        self.label.set_label(text);
        // app::awake();
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
    pub fn close(&mut self) {
        self.window.hide();
        self.app.quit();
    }
    pub fn run_loop(&mut self, rx: Receiver<ProgressMsg>) {
        loop {
            match rx.try_recv() {
                Ok(msg) => {
                    if msg.done {
                        break;
                    }
                    println!("progress: {}%, {}", (msg.value * 100.0) as u32, msg.text);
                    self.set_progress(msg.value, &msg.text);
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => {},
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    break;
                }
            }
            if !self.window.shown() || self.is_cancelled() {
                break;
            }
            self.app.wait();
        }
        self.close();
    }
}
