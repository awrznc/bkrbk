/// 色 (24bit Color) を扱うための構造体です。
#[derive(Debug, Default)]
#[repr(C)]
pub struct Color {
    /// <p style="color: #FF0000">赤要素</p>
    pub red: u8,
    /// <p style="color: #00FF00">緑要素</p>
    pub green: u8,
    /// <p style="color: #0000FF">青要素</p>
    pub blue: u8,
}

impl std::fmt::Display for Color {
    /// 色の情報を表示します。
    /// ```
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "\x1b[48;2;{};{};{}m  \x1b[0m {:?}",
            self.red, self.green, self.blue, self,
        )
    }
}

impl Color {
    /// Color 構造体を初期化して返します。
    pub fn new(color: u32) -> Self {
        Self {
            red: ((color & 0x00_FF_00_00) >> 16) as u8,
            green: ((color & 0x00_00_FF_00) >> 8) as u8,
            blue: (color & 0x00_00_00_FF) as u8,
        }
    }

    /// 指定した色と透過率でアルファブレンディングを行います。
    pub fn alpha_blend(&self, alpha: u8, blend_color: u32) -> u32 {
        let bco = Self::new(blend_color);
        let a = alpha as f64 / 255.0;

        // 背景色 + (重ねる色 - 背景色) * (透過率 / 0xff)
        let red = self.red as f64 + ((bco.red as f64 - self.red as f64) * a);
        let green = self.green as f64 + ((bco.green as f64 - self.green as f64) * a);
        let blue = self.blue as f64 + ((bco.blue as f64 - self.blue as f64) * a);

        Self::as_u32(red as u8, green as u8, blue as u8)
    }

    /// u8 型で表現された RGB を u32 型に変換します。
    pub fn as_u32(r: u8, g: u8, b: u8) -> u32 {
        ((r as u32) << 16) + ((g as u32) << 8) + (b as u32)
    }

    /// u8 型の値をモノクロ値として u32 型に変換します。
    pub fn u8_to_u32(c: u8) -> u32 {
        Self::as_u32(c, c, c)
    }

    /// RGB を u32 型に変換します。
    pub fn to_rgb_u32(&self) -> u32 {
        ((self.red as u32) << 16) + ((self.green as u32) << 8) + (self.blue as u32)
    }

    /// BGR を u32 型に変換します。
    pub fn to_bgr_u32(&self) -> u32 {
        ((self.blue as u32) << 16) + ((self.green as u32) << 8) + (self.red as u32)
    }
}
