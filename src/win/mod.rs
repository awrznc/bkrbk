//! Windowsに依存する操作を行うための機能です。
//!
//! unsafeを多用しているので想定外のエラーが発生する可能性が高いです。

use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::System::LibraryLoader::GetModuleHandleA,
    Win32::UI::Input::KeyboardAndMouse::SetActiveWindow,
    Win32::{
        Graphics::Gdi::{CreateSolidBrush, UpdateWindow},
        UI::WindowsAndMessaging::*,
    },
};

pub mod dib;
mod message;
mod window;

trait OSString {
    fn to_pcstr(&self) -> PCSTR;
}

impl OSString for str {
    /// 以下の方法で代替可能です。
    /// ```rust,no_run
    /// let target = "name";
    /// let c_str = std::ffi::CString::new(target).unwrap();
    /// let pcstr = windows::core::PCSTR::from_raw(c_str.as_bytes_with_nul().as_ptr());
    /// // or
    /// let pcstr = windows::core::PCSTR(target.as_ptr());
    /// ```
    fn to_pcstr(&self) -> PCSTR {
        PCSTR::from_raw(self.as_ptr())
    }
}

#[repr(C)]
pub struct StringPair {
    /// 本体
    pub entity: String,
    /// 参照
    pub reference: PCSTR,
}

impl StringPair {
    /// 領域の確保を行います。
    /// 必ずこの処理の後に `set_reference()` を実行してください。
    pub fn new(entity: String) -> Self {
        let entity = entity + "\0";
        Self {
            entity,
            reference: PCSTR::null(),
        }
    }
    pub fn set_reference(&mut self) {
        self.reference = self.entity.as_str().to_pcstr();
    }
}

#[repr(C)]
pub struct Core {
    pub main_window_name: StringPair,
    pub class_name: StringPair,
    pub handler: HWND,
    message: MSG,
    /// 画面に描画する情報です。
    pub image: Vec<u32>,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    pub dib: dib::DIB,
    pub is_hide: bool,
    pub front: window::Window,
}

impl Drop for Core {
    /// 以下の解放処理を行います。
    ///
    /// 対象 | 役割
    /// --- | ---
    /// prop | コールバック関数上でインスタンスを参照するために利用。
    /// window | ウインドウを扱う際に利用。
    fn drop(&mut self) {
        // unsafe {
        //     //  タイミングが悪いかも（手動でウィンドウを閉じるとエラーになる）
        //     // Self::destroy_property(self.handler);

        //     // DestroyWindow(self.handler).unwrap();
        //     println!("Drop!");
        // }
    }
}

impl Core {
    /// 領域の確保を行います。
    /// ( C言語ではメモリのアドレスに依存した実装をすることが多いため、まずは領域を確保してから作業します )
    pub fn new(name: &str, width: i32, height: i32, bg: u32) -> Self {
        let w = 192;
        let h = 108;
        Self {
            // MEMO: Window名はいずれも完全包含してはならない（検索時に意図しない挙動になることがあるため）
            main_window_name: StringPair::new(String::from(name) + "_main"),
            class_name: StringPair::new(String::from(name) + "_class"),
            handler: HWND::default(),
            message: MSG::default(),
            hdc: windows::Win32::Graphics::Gdi::HDC::default(),
            image: vec![0x00_FF_FF_FF; (h * w) as usize],
            dib: dib::DIB::new(w, h),
            is_hide: true,
            front: window::Window::new(name, width, height, bg),
        }
    }

    /// システムの初期化を行います。
    pub fn init(&mut self) -> std::result::Result<(), Error> {
        let instance = unsafe { GetModuleHandleA(None) }.map_err(Error::from)?;
        assert!(!instance.is_invalid());

        let width = self.dib.info.bmiHeader.biWidth;
        let height = -(self.dib.info.bmiHeader.biHeight);
        for pixel in self.image.iter_mut() {
            *pixel = 0x00_FF_FF_FF;
        }

        self.class_name.set_reference();
        self.main_window_name.set_reference();

        unsafe {
            let wc = WNDCLASSA {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wndproc),
                hInstance: HINSTANCE::from(instance),
                hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                hbrBackground: CreateSolidBrush(COLORREF(0x00_FF_FF_00)),
                lpszClassName: self.class_name.reference,
                ..Default::default()
            };

            let atom = RegisterClassA(&wc);
            assert!(atom != 0);

            let window_handler = CreateWindowExA(
                WINDOW_EX_STYLE::default(),
                self.class_name.reference,
                self.main_window_name.reference,
                WS_SYSMENU | WS_VISIBLE,
                200,
                200,
                width,
                height,
                None,
                None,
                instance,
                None,
            );

            fix_window_size(window_handler, width, height);

            self.handler = window_handler;
            self.front
                .init(self.class_name.reference, window_handler, instance);

            self.dib.reset(self.handler);

            Ok(())
        }
    }

    pub fn reset(&self) {
        self.set_property();
        unsafe {
            ShowWindow(self.handler, SW_RESTORE);
            UpdateWindow(self.handler);
            SetActiveWindow(self.handler);
        }

        self.front.reset();
    }

    /// 画面の更新を行います。
    ///
    /// * 返り値がSomeだった場合の真理値はメッセージの取得有無を表します。
    /// * 返り値がNoneだった場合はアプリケーションの終了を表します。
    pub fn update(&mut self) -> Option<bool> {
        let mut message: MSG = self.message;
        unsafe {
            // if GetMessageA(&mut message, HWND(0), 0, 0).into() {
            if PeekMessageA(&mut message, HWND(0), 0, 0, PM_REMOVE).into() {
                TranslateMessage(&message);
                DispatchMessageA(&message);

                match message.message {
                    WM_QUIT => None,
                    WM_PAINT => Some(false),
                    _ => Some(true),
                }
            } else {
                Some(false)
            }
        }
    }

    pub fn call_draw(&self) {
        unsafe {
            windows::Win32::Graphics::Gdi::InvalidateRect(self.handler, None, false);
        }
    }

    pub fn draw(&self) {
        use windows::Win32::Graphics::Gdi::*;

        let image = &self.image;
        let width = self.dib.info.bmiHeader.biWidth;
        let height = -(self.dib.info.bmiHeader.biHeight);
        // 描画
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(self.handler, &mut ps);

            StretchDIBits(
                hdc,
                // to
                0,
                0,
                width,
                height,
                // from
                0,
                0,
                width,
                height,
                // image info
                Some(image.as_ptr() as _),
                // image header
                &self.dib.info,
                DIB_RGB_COLORS,
                SRCCOPY,
            );

            EndPaint(self.handler, &ps);

            // WM_PAINTの呼び出し、全ての領域を再描画する
            // InvalidateRect(self.handler, None, false);
        }
    }

    /// コールバック関数
    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match message {
            WM_CREATE => Self::on_create(window),
            WM_DESTROY => Self::on_destroy(),
            WM_ACTIVATE => Self::on_active(window),
            WM_MOVE => Self::on_move(window),
            WM_PAINT => Self::on_paint(window),
            _ => unsafe { DefWindowProcA(window, message, wparam, lparam) },
        }
    }
}

/// アプリケーションを終了するメッセージを発行します。
pub fn quit() {
    unsafe { PostQuitMessage(0) }
}

/// ディスプレイの大きさを取得します。
fn get_display_size() -> (i32, i32) {
    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    (width, height)
}

/// ウィンドウのサイズを指定した描画範囲のサイズをもとに変更し、
/// 位置をディスプレイの中央に移動させます。
fn fix_window_size(window: HWND, width: i32, height: i32) {
    // 描画範囲の計算
    let (frame_width, frame_height) = unsafe {
        // ディスプレイの左上を0としたウィンドウの座標を取得する
        let mut window_rect: RECT = RECT::default();
        GetWindowRect(window, &mut window_rect).unwrap();

        // ウィンドウの左上を0としたウィンドウの右下の座標を取得する
        let mut client_rect: RECT = RECT::default();
        GetClientRect(window, &mut client_rect).unwrap();

        (
            window_rect.right - window_rect.left - client_rect.right,
            window_rect.bottom - window_rect.top - client_rect.bottom,
        )
    };
    let corrected_width = width + frame_width;
    let corrected_height = height + frame_height;

    // 描画する場所の計算
    let (x, y) = get_display_size();
    let corrected_x = (x / 2) - (corrected_width / 2);
    let corrected_y = (y / 2) - (corrected_height / 2);

    unsafe {
        SetWindowPos(
            window,
            None,
            corrected_x,
            corrected_y,
            corrected_width,
            corrected_height,
            SWP_SHOWWINDOW,
        )
        .unwrap();
    }
}
