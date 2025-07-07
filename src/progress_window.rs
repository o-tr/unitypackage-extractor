// 進捗状況を表示するウィンドウ（7zip風ダイアログ）
// Windows向け。native-windows-gui (nwg) を利用

use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use native_windows_gui as nwg;

pub struct ProgressWindow {
    window: nwg::Window,
    progress_bar: nwg::ProgressBar,
    label: nwg::Label,
    #[allow(dead_code)]
    cancel_btn: nwg::Button,
    cancelled: Arc<AtomicBool>,
}

impl ProgressWindow {
    pub fn new(title: &str, max: u32) -> Self {
        nwg::init().expect("Failed to init NWG");
        let mut window = nwg::Window::default();
        let mut progress_bar = nwg::ProgressBar::default();
        let mut label = nwg::Label::default();
        let mut cancel_btn = nwg::Button::default();
        let cancelled = Arc::new(AtomicBool::new(false));
        nwg::Window::builder()
            .size((400, 120))
            .position((300, 300))
            .title(title)
            .build(&mut window)
            .unwrap();
        nwg::ProgressBar::builder()
            .parent(&window)
            .size((360, 20))
            .position((20, 40))
            .range(0..max)
            .build(&mut progress_bar)
            .unwrap();
        nwg::Label::builder()
            .parent(&window)
            .text("")
            .size((360, 20))
            .position((20, 10))
            .build(&mut label)
            .unwrap();
        nwg::Button::builder()
            .parent(&window)
            .text("キャンセル")
            .size((80, 30))
            .position((160, 70))
            .build(&mut cancel_btn)
            .unwrap();
        // ウィンドウのクローズイベントでキャンセルフラグを立てる
        {
            let cancelled_clone = Arc::clone(&cancelled);
            let _handler = nwg::full_bind_event_handler(&window.handle, move |evt, _evt_data, _handle| {
                match evt {
                    nwg::Event::OnWindowClose => {
                        cancelled_clone.store(true, Ordering::SeqCst);
                        nwg::stop_thread_dispatch();
                    }
                    _ => {}
                }
            });
        }
        // キャンセルボタン押下時のイベント
        {
            let cancelled_clone = Arc::clone(&cancelled);
            let _handler = nwg::full_bind_event_handler(&cancel_btn.handle, move |evt, _evt_data, _handle| {
                match evt {
                    nwg::Event::OnButtonClick => {
                        cancelled_clone.store(true, Ordering::SeqCst);
                    }
                    _ => {}
                }
            });
        }
        window.set_visible(true);
        Self {
            window,
            progress_bar,
            label,
            cancel_btn,
            cancelled,
        }
    }
    pub fn set_progress(&self, value: u32, text: &str) {
        self.progress_bar.set_pos(value);
        self.label.set_text(text);
    }
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
    pub fn close(&self) {
        self.window.set_visible(false);
    }
    pub fn set_range(&self, min: u32, max: u32) {
        self.progress_bar.set_range(min..max);
    }
}
