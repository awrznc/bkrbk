//! メッセージ関連の処理をまとめた機能です。

use windows::Win32::{
    Foundation::*,
    UI::{
        Input::KeyboardAndMouse::{GetActiveWindow, SetActiveWindow},
        WindowsAndMessaging::*,
    },
};

/// プロパティ関連の実装です。
impl super::Core {
    /// インスタンスオブジェクトのポインタを取得するためのIDを取得します。
    pub fn get_handler_id(handler: &HWND) -> String {
        format!("prop-0x{:x}", handler.0 as usize)
    }

    /// プロパティにselfを登録します。
    ///
    /// selfのアドレスが確定したら利用して下さい。
    pub fn set_property(&self) {
        let id = Self::get_handler_id(&self.handler);
        let c_str = std::ffi::CString::new(id).unwrap();
        let self_id = {
            let c_ptr: *const u8 = c_str.as_bytes_with_nul().as_ptr();
            windows::core::PCSTR::from_raw(c_ptr)
        };
        // let self_id = Self::get_handler_id(&self.handler).to_pcstr();

        let handle = HANDLE(self as *const Self as isize);
        match unsafe { SetPropA(self.handler, self_id, handle) } {
            Ok(_) => {
                let result_raw = unsafe { GetPropA(self.handler, self_id) };
                assert!(!result_raw.is_invalid());
            }
            Err(_) => panic!("SetProp error!"),
        }
    }

    /// プロパティからselfを取得します。
    pub fn get_property<'a>(handler: HWND) -> Option<&'a mut Self> {
        let id = Self::get_handler_id(&handler);
        let c_str = std::ffi::CString::new(id).unwrap();
        let self_id = {
            let c_ptr: *const u8 = c_str.as_bytes_with_nul().as_ptr();
            windows::core::PCSTR::from_raw(c_ptr)
        };
        // let self_id = Self::get_handler_id(&handler).to_pcstr();

        // TODO: 再帰的に探索できるようにする
        let property = unsafe { GetPropA(handler, self_id) };
        match property.is_invalid() {
            true => {
                let parent_handler = unsafe { GetParent(handler) };
                let parent_id = Self::get_handler_id(&parent_handler);
                let parent_c_str = std::ffi::CString::new(parent_id).unwrap();
                let parent_self_id = {
                    let c_ptr: *const u8 = parent_c_str.as_bytes_with_nul().as_ptr();
                    windows::core::PCSTR::from_raw(c_ptr)
                };
                // let parent_self_id = Self::get_handler_id(&parent_handler).to_pcstr();
                let parent_property = unsafe { GetPropA(parent_handler, parent_self_id) };
                match parent_property.is_invalid() {
                    true => None,
                    false => Some(unsafe { &mut *(parent_property.0 as *mut Self) }),
                }
            }
            false => Some(unsafe { &mut *(property.0 as *mut Self) }),
        }
    }

    /// selfを登録したプロパティを削除します。
    pub fn destroy_property(handler: HWND) {
        let id = Self::get_handler_id(&handler);
        let c_str = std::ffi::CString::new(id).unwrap();
        let self_id = {
            let c_ptr: *const u8 = c_str.as_bytes_with_nul().as_ptr();
            windows::core::PCSTR::from_raw(c_ptr)
        };
        // let self_id = Self::get_handler_id(&handler).to_pcstr();

        match unsafe { RemovePropA(handler, self_id) } {
            Ok(_) => {}
            Err(value) => panic!("{}", value),
        }
    }
}

impl super::Core {
    /// ウィンドウを作成した際の処理です。
    pub(super) fn on_create(_window: HWND) -> LRESULT {
        // unsafe {
        //     windows::Win32::Graphics::Gdi::InvalidateRect(window, None, false);
        // }

        LRESULT(0)
    }

    /// ウィンドウを作成した際の処理です。
    pub(super) fn on_destroy() -> LRESULT {
        println!("WM_DESTROY");
        unsafe {
            // SetTimerをしていないのでいらないかも
            // KillTimer(window, 1);
            PostQuitMessage(0)
        };
        LRESULT(0)
    }

    /// ウィンドウを移動した際の処理です。
    pub(super) fn on_move(window: HWND) -> LRESULT {
        if let Some(this) = Self::get_property(window) {
            if !this.front.is_hide {
                this.front.show(
                    window,
                    this.class_name.reference,
                    &this.main_window_name.entity,
                );
            }
        }
        LRESULT(0)
    }

    /// サブウィンドウをクリックした際にアクティブにしないようにする処理です。
    /// 基本的にサブウィンドウの設定上クリックされることはありませんが、将来的に更に複数のウィンドウを扱う場合に役立ちそうなので残しておきます。
    pub(super) fn on_active(window: HWND) -> LRESULT {
        if let Some(this) = Self::get_property(window) {
            unsafe {
                let active_window = GetActiveWindow();
                // 現在のアクティブウィンドウがサブウィンドウだった場合
                if this.front.hwnd == active_window {
                    // メインウィンドウをアクティブにする
                    SetActiveWindow(this.handler);
                }
            }
        }
        LRESULT(0)
    }

    /// 描画処理です。
    pub(super) fn on_paint(window: HWND) -> LRESULT {
        // MEMO: 必要以上に呼び出されているかも
        if let Some(this) = Self::get_property(window) {
            this.draw();
            this.front.draw();
        } else {
            dbg!("else!");
        }
        LRESULT(0)
    }
}
