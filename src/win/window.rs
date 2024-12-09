//! ウィンドウを操作するための機能です。

use windows::{
    core::*,
    Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*},
};

pub struct Window {
    name: super::StringPair,
    pub hwnd: super::HWND,
    width: i32,
    height: i32,
    pub image: Vec<u32>,
    pub dib: super::dib::DIB,
    pub is_hide: bool,

    pub bg: u32,
}

impl Window {
    pub fn new(name: &str, width: i32, height: i32, bg: u32) -> Self {
        Self {
            name: super::StringPair::new(String::from(name) + "_aaaa"),
            hwnd: Default::default(),
            width,
            height,
            image: vec![0x00_FF_00_FF; (width * height) as usize],
            dib: super::dib::DIB::new(width, height),
            is_hide: true,
            bg,
        }
    }

    pub fn init(&mut self, class_name: PCSTR, parent: HWND, instance: HMODULE) {
        self.name.set_reference();
        self.hwnd = unsafe {
            CreateWindowExA(
                // WINDOW_EX_STYLE::default(),
                WS_EX_LAYERED,
                class_name,
                self.name.reference,
                WS_POPUP,
                500,
                500,
                self.width,
                self.height,
                parent,
                None,
                instance,
                None,
            )
        };
    }

    pub fn reset(&self) {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
            UpdateWindow(self.hwnd);

            // 透明化
            SetLayeredWindowAttributes(
                self.hwnd,
                COLORREF(self.bg),
                0xFF,
                LWA_COLORKEY | LWA_ALPHA,
            )
            .unwrap();
        }
    }

    pub fn call_draw(&self) {
        unsafe {
            windows::Win32::Graphics::Gdi::InvalidateRect(self.hwnd, None, false);
        }
    }

    pub fn draw(&self) {
        use windows::Win32::Graphics::Gdi::*;

        let image = &self.image;
        let width = self.width;
        let height = self.height;
        // 描画
        unsafe {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(self.hwnd, &mut ps);

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

            EndPaint(self.hwnd, &ps);

            // WM_PAINTの呼び出し、全ての領域を再描画する
            // InvalidateRect(self.handler, None, false);
        }
    }

    /// ウィンドウを表示します。
    pub fn show(&mut self, window: HWND, class: PCSTR, main: &str) {
        self.change_window_show_status(window, class, main, SW_RESTORE);
        self.is_hide = false;
    }

    /// ウィンドウを隠します。
    pub fn hide(&mut self, window: HWND, class: PCSTR, main: &str) {
        self.change_window_show_status(window, class, main, SW_HIDE);
        self.is_hide = true;
    }

    /// ウィンドウの表示を操作します。
    fn change_window_show_status(
        &self,
        window: HWND,
        class: PCSTR,
        main: &str,
        ncmdshow: SHOW_WINDOW_CMD,
    ) {
        let main_window_name = main;
        let length = main_window_name.len();
        let mut window_text = vec![0; length];
        let text_size = unsafe { GetWindowTextA(window, &mut window_text) as usize };

        // NULL文字の分 -1 する
        if text_size != length - 1 || window_text != main_window_name.as_bytes() {
            return;
        }

        let mut window_info: WINDOWINFO = Default::default();
        unsafe { GetWindowInfo(window, &mut window_info).unwrap() };
        let x: i32 = window_info.rcWindow.left;
        let y: i32 = window_info.rcWindow.top;

        unsafe {
            let sub_window = FindWindowA(class, self.name.reference);
            MoveWindow(sub_window, x, y - 27, self.width, self.height, true).unwrap();
            ShowWindow(sub_window, ncmdshow);
        }
    }
}
