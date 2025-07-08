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
    ConfirmOverwrite {
        path: String,
        resp_tx: Sender<bool>
    },
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
                            let result = self.show_overwrite_dialog(
                                path,
                                &self.cancelled
                            );
                            // キャンセルされた場合は即座にbreak
                            if self.is_cancelled() {
                                break;
                            }
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
    fn show_overwrite_dialog(&self, path: String, cancelled: &Arc<AtomicBool>) -> Option<u8> {
        use fltk::{window::Window, button::Button, frame::Frame, prelude::*, enums::Align};
        use std::rc::Rc;
        use std::cell::RefCell;
        use std::sync::atomic::Ordering;

        let win = Rc::new(RefCell::new(Window::new(0, 0, 400, 140, "ファイルの上書き確認")));
        
        // タイトルメッセージ
        let mut title_frame = Frame::new(20, 10, 360, 30, "出力先のフォルダーに既存ファイルへの同じ名前のファイルが存在しています。");
        title_frame.set_align(Align::Left | Align::Inside);
        title_frame.set_label_size(11);

        let label = format!("{}に上書きしますか？", path);
        let mut current_info = Frame::new(20, 40, 360, 30, &*label);
        current_info.set_align(Align::Left | Align::Inside);
        current_info.set_label_size(10);

        let mut btn_yes = Button::new(20, 80, 100, 25, "はい(&Y)");
        let mut btn_no = Button::new(120, 80, 100, 25, "すべてはい(&A)");
        let mut btn_cancel = Button::new(220, 80, 160, 25, "自動的に名前を変更(&U)");
        let mut btn_no_all = Button::new(20, 110, 100, 25, "いいえ(&N)");
        let mut btn_skip_all = Button::new(120, 110, 100, 25, "すべていいえ(&L)");
        let mut btn_cancel_op = Button::new(220, 110, 160, 25, "キャンセル");
        
        let result = Rc::new(RefCell::new(None));
        
        // はい（上書き）
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_yes.set_callback(move |_| {
                *result.borrow_mut() = Some(0);
                win.borrow_mut().hide();
            });
        }
        
        // すべてはい（すべて上書き）
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_no.set_callback(move |_| {
                *result.borrow_mut() = Some(2);
                win.borrow_mut().hide();
            });
        }
        
        // いいえ（スキップ）
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_no_all.set_callback(move |_| {
                *result.borrow_mut() = Some(1);
                win.borrow_mut().hide();
            });
        }
        
        // すべていいえ（すべてスキップ）
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            btn_skip_all.set_callback(move |_| {
                *result.borrow_mut() = Some(3);
                win.borrow_mut().hide();
            });
        }
        
        // キャンセル
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            let cancelled = Arc::clone(cancelled);
            btn_cancel_op.set_callback(move |_| {
                *result.borrow_mut() = Some(1); // スキップとして処理
                cancelled.store(true, Ordering::SeqCst); // 全体キャンセル
                win.borrow_mut().hide();
            });
        }
        
        // 自動的に名前を変更（今回はスキップとして処理）
        {
            let result = Rc::clone(&result);
            let win = Rc::clone(&win);
            let cancelled = Arc::clone(cancelled);
            btn_cancel.set_callback(move |_| {
                *result.borrow_mut() = Some(1); // スキップとして処理
                cancelled.store(true, Ordering::SeqCst); // 全体キャンセル
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
