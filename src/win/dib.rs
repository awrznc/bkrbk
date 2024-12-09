use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::BITMAPINFO;
use windows::Win32::Graphics::Gdi::*;

#[repr(C)]
pub struct DIB {
    pub info: BITMAPINFO,
    pub bits: Vec<u32>,
    pub bmp: HBITMAP,
    pub buf: HDC,
}

impl DIB {
    pub fn new(width: i32, height: i32) -> Self {
        // 1. HBITMAP の作成
        let bmp_header = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -(height),
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            biSizeImage: 0, // BI_RGB の場合は 0 に設定可能

            // 適当
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrImportant: 0,
            biClrUsed: 0,
        };
        let bmp_info = BITMAPINFO {
            bmiHeader: bmp_header,

            // 適当
            bmiColors: [RGBQUAD::default()],
        };
        Self {
            info: bmp_info,
            bits: vec![0x00FF00FF; (width * height) as usize],
            bmp: HBITMAP::default(),
            buf: HDC::default(),
        }
    }

    pub fn reset(&mut self, window: HWND) {
        let hdc = unsafe { GetDC(window) };
        self.bmp = unsafe {
            CreateDIBSection(
                hdc,
                &self.info,
                DIB_RGB_COLORS,
                self.bits.as_ptr() as _,
                None,
                0,
            )
            .unwrap()
        };
        self.buf = unsafe { CreateCompatibleDC(hdc) };
        unsafe { SelectObject(self.buf, self.bmp) };
        unsafe { ReleaseDC(window, hdc) };
    }
}

impl Drop for DIB {
    /// 解放処理を行います。
    fn drop(&mut self) {
        if !self.bmp.is_invalid() {
            unsafe { DeleteObject(self.bmp) }.unwrap();
        }
    }
}
