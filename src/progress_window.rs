// 進捗状況を表示するウィンドウ（7zip風ダイアログ）
// クロスプラットフォーム対応: fltk-rs を利用

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::sync::mpsc::{Receiver, Sender};
use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window, group::Pack, misc::Progress};

pub struct ProgressWindow {
    app: app::App,
    window: Window,
    progress_bar: Progress,
    label: Frame,
    #[allow(dead_code)]
    cancel_btn: Button,
    cancelled: Arc<AtomicBool>,
    overwrite_all: Option<bool>,
}

pub enum ProgressMsg {
    Progress { value: f32, text: String, done: bool },
    ConfirmOverwrite { path: String, resp_tx: Sender<bool> },
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
            overwrite_all: None,
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
                    match msg {
                        ProgressMsg::Progress { value, text, done } => {
                            if done {
                                break;
                            }
                            self.set_progress(value, &text);
                        },
                        ProgressMsg::ConfirmOverwrite { path, resp_tx } => {
                            // すべて上書き/すべてスキップが選択済みなら自動応答
                            if let Some(val) = self.overwrite_all {
                                let _ = resp_tx.send(val);
                                continue;
                            }
                            let result = self.show_overwrite_dialog(&path);
                            let ok = matches!(result, Some(0) | Some(2));
                            if matches!(result, Some(2)) {
                                self.overwrite_all = Some(true);
                            } else if matches!(result, Some(3)) {
                                self.overwrite_all = Some(false);
                            }
                            let _ = resp_tx.send(ok);
                        },
                    }
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
    fn show_overwrite_dialog(&self, path: &str) -> Option<u8> {
        use fltk::{window::Window, button::Button, frame::Frame, prelude::*};
        use std::rc::Rc;
        use std::cell::RefCell;
        let win = Rc::new(RefCell::new(Window::new(0, 0, 400, 180, "上書き確認")));
        let msg = format!("既にファイルが存在します。上書きしますか？\n{}", path);
        let _frame = Frame::new(20, 20, 360, 60, msg.as_str());
        let mut btn_overwrite = Button::new(20, 100, 70, 30, "上書き");
        let mut btn_skip = Button::new(100, 100, 70, 30, "スキップ");
        let mut btn_all_overwrite = Button::new(180, 100, 110, 30, "すべて上書き");
        let mut btn_all_skip = Button::new(20, 140, 110, 30, "すべてスキップ");
        let result = Rc::new(RefCell::new(None));
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_overwrite.set_callback(move |_| {
                *result.borrow_mut() = Some(0);
                win.borrow_mut().hide();
            });
        }
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_skip.set_callback(move |_| {
                *result.borrow_mut() = Some(1);
                win.borrow_mut().hide();
            });
        }
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_all_overwrite.set_callback(move |_| {
                *result.borrow_mut() = Some(2);
                win.borrow_mut().hide();
            });
        }
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_all_skip.set_callback(move |_| {
                *result.borrow_mut() = Some(3);
                win.borrow_mut().hide();
            });
        }
        win.borrow_mut().end();
        win.borrow_mut().show();
        while win.borrow().shown() {
            fltk::app::wait();
        }
        *result.borrow()
    }
}
