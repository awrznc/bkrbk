//! 処理間隔をハンドリングするための機能です。

use std::time::{Duration, Instant};

/// fps の実測値を扱うための構造体です。
struct FpsPrinter {
    /// 最後に更新を行った時刻
    last_updated: Instant,

    /// 最後の更新から経過したフレーム数
    count: u128,

    /// 理想の更新間隔
    fps: u32,
}

impl FpsPrinter {
    fn new(fps: u32, now: Instant) -> Self {
        Self {
            last_updated: now,
            count: 0,
            fps,
        }
    }

    /// 指定したフレームが経過した際のfpsの実測値を表示します。
    fn print(&mut self, last_updated: Instant) -> Option<f32> {
        self.count += 1;
        if self.count >= self.fps as u128 {
            let elapsed = last_updated.saturating_duration_since(self.last_updated);
            let fps = self.fps as f32 * 1000.0 / elapsed.as_millis() as f32;

            {
                use std::io::Write;
                print!("fps: {:.1}\r", fps);
                std::io::stdout().flush().unwrap();
            }

            self.last_updated = last_updated;
            self.count = 0;
            Some(fps)
        } else {
            None
        }
    }
}

/// タイマーを扱うための構造体です。
pub struct Timer {
    /// 待機時間
    pub interval: Duration,

    /// 構造体の初期化を行った時刻
    initialize_time: Instant,

    /// 最後に更新を行った時刻
    last_updated: Instant,

    /// 更新間隔の実測値を表示します
    printer: FpsPrinter,
}

impl Default for Timer {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            interval: Duration::new(0, 1_000_000_000u32 / Self::DEFAULT_FPS),
            initialize_time: now,
            last_updated: now,
            printer: FpsPrinter::new(Self::DEFAULT_FPS, now),
        }
    }
}

impl Timer {
    /// デフォルトの画面の更新間隔です。
    const DEFAULT_FPS: u32 = 30;

    /// Timer 構造体を初期化して返します。
    pub fn new(fps: u32) -> Self {
        let now = Instant::now();
        Self {
            interval: Duration::from_secs(1) / fps,
            initialize_time: now,
            last_updated: now,
            printer: FpsPrinter::new(fps, now),
        }
    }

    /// 更新時間まで待機します。
    /// ```rust
    /// // 30fps
    /// let fps = 30;
    /// use tr_engine::timer;
    /// let mut t = timer::Timer::new(fps);
    ///
    /// // 1秒間にcounterを30回更新する
    /// use std::time::{Duration, SystemTime};
    /// let mut counter = 0;
    /// let before = SystemTime::now();
    /// loop {
    ///     let after = SystemTime::now();
    ///     if after.duration_since(before).unwrap().as_secs() == 1 {
    ///         break;
    ///     }
    ///     counter += 1;
    ///     t.sleep();
    /// }
    /// assert_eq!(counter, fps);
    /// ```
    pub fn sleep(&mut self) {
        // 前回の更新終了時刻から現在時刻を引いた差分の時間を取得
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_updated);

        // 待機時間を取得
        let count = elapsed.as_millis() / self.interval.as_millis();
        let min_elapsed = elapsed.saturating_sub(self.interval * count as u32);
        let wait_time = self.interval.saturating_sub(min_elapsed);

        // 待機
        sleep(wait_time);

        // 更新時間（予測）の更新
        self.last_updated = now + wait_time;

        // fpsの表示
        // NOTE: 確認したいときにコメントインする？テストに使えるかも
        #[cfg(debug_assertions)]
        self.print()
    }

    /// fps の実測値を表示します。
    pub fn print(&mut self) {
        self.printer.print(self.last_updated);
    }

    /// 初期化時点から経過した秒数を取得します。
    pub fn get_operating_time(&self) -> u64 {
        self.initialize_time.elapsed().as_secs()
    }
}

/// 指定した時間分 Sleep します。
///
/// NOTE: OS が Windows の場合は、デフォルトの Sleep の精度が悪いため、タイマーの解像度を一時的に上げる処理が入ります。
/// ただし、 `Windows 10, version 2004` よりも前のバージョンでは「全てのシステムに」影響が出てしまうことに注意してください。
#[inline(always)]
pub fn sleep(wait_time: Duration) {
    #[cfg(target_os = "windows")]
    unsafe {
        /// 最小タイマー解像度 (ミリ秒単位)
        const MINIMUM_TIMER_RESOLUTION: u32 = 1;
        windows::Win32::Media::timeBeginPeriod(MINIMUM_TIMER_RESOLUTION);
        std::thread::sleep(wait_time);
        windows::Win32::Media::timeEndPeriod(MINIMUM_TIMER_RESOLUTION);
    }

    #[cfg(not(target_os = "windows"))]
    std::thread::sleep(wait_time);
}

/// タイムゾーンを扱うための構造体です。
pub struct TimeZone {
    pub hour: u16,
    pub minute: u16,
    // NOTE: 必要になったらコメントインする
    // pub second: u16,
    // pub milli_seconds: u16,
}

/// 日時を扱うための構造体です。
pub struct DateTime {
    pub year: u16,
    pub month: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milli_seconds: u16,
}

impl DateTime {
    pub fn new(
        year: u16,
        month: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milli_seconds: u16,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            milli_seconds,
        }
    }
}

/// 時間を扱うための構造体です。
pub struct Time {
    pub datetime: DateTime,
    pub timezone: TimeZone,
}

impl Time {
    pub fn format_iso_8601(&self) -> String {
        format!(
            "{:>04}-{:>02}-{:>02}T{:>02}:{:>02}:{:>02}.{}+{:>02}:{:>02}",
            self.datetime.year,
            self.datetime.month,
            self.datetime.day,
            self.datetime.hour,
            self.datetime.minute,
            self.datetime.second,
            self.datetime.milli_seconds,
            self.timezone.hour,
            self.timezone.minute
        )
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.format_iso_8601())
    }
}
