//! Axis scales — project [`PlottableValue`]s into the plot's 0..1 space
//! and generate their tick marks.
//!
//! Mirrors Swift Charts' scale surface ([linear], [log], [category],
//! [date]) so the chart's axis behaviour composes cleanly with the
//! `AxisConfig` builder. Each implementation is an owned struct so it
//! can be cloned into paint callbacks without interior mutability.
//!
//! [linear]: https://developer.apple.com/documentation/charts/linearaxisscale
//! [log]: https://developer.apple.com/documentation/charts/logscaleaxisscale
//! [category]: https://developer.apple.com/documentation/charts/categoryaxisscale
//! [date]: https://developer.apple.com/documentation/charts/dateaxisscale

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gpui::SharedString;

use super::types::{PlottableValue, nice_ticks};

/// A projection from a domain value to the plot's 0..1 normalised
/// coordinate, paired with a tick generator for the same domain.
///
/// `project` returns positions inside `[0.0, 1.0]` for values inside the
/// scale's domain; values outside the domain clamp to the closest edge
/// so paint code never has to special-case out-of-range inputs.
///
/// `ticks` returns up to `count` `(value, label)` pairs suitable for
/// rendering axis labels. Implementations round to "nice" values so the
/// labels read as 0, 20, 40, 60 rather than 0, 17.3, 34.6, ….
pub trait Scale: std::fmt::Debug + Send + Sync + 'static {
    /// Project a domain value into 0..1 (clamped).
    fn project(&self, value: &PlottableValue) -> f32;
    /// Generate approximately `count` axis ticks with display labels.
    fn ticks(&self, count: usize) -> Vec<(PlottableValue, SharedString)>;
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LinearScale
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Continuous numeric scale: `y = (v - domain_lo) / (domain_hi - domain_lo)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinearScale {
    /// `(domain_lo, domain_hi)` — the values that map to 0.0 and 1.0.
    pub domain: (f64, f64),
}

impl LinearScale {
    /// Create a linear scale from an explicit domain.
    pub fn new(domain_lo: f64, domain_hi: f64) -> Self {
        Self {
            domain: (domain_lo, domain_hi),
        }
    }
}

impl Scale for LinearScale {
    fn project(&self, value: &PlottableValue) -> f32 {
        let v = value.as_number().unwrap_or(self.domain.0);
        let (lo, hi) = self.domain;
        let range = hi - lo;
        if range.abs() < f64::EPSILON {
            return 0.0;
        }
        ((v - lo) / range).clamp(0.0, 1.0) as f32
    }

    fn ticks(&self, count: usize) -> Vec<(PlottableValue, SharedString)> {
        let (lo, hi) = self.domain;
        let ticks = nice_ticks(lo as f32, hi as f32, count);
        ticks
            .into_iter()
            .map(|t| {
                let label = format_linear(t);
                (PlottableValue::Number(t as f64), SharedString::from(label))
            })
            .collect()
    }
}

fn format_linear(v: f32) -> String {
    if v.abs() < 1e-6 {
        return "0".to_string();
    }
    if v.fract() == 0.0 && v.abs() < 1_000_000.0 {
        return format!("{}", v as i64);
    }
    format!("{v:.1}")
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// LogScale
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Logarithmic scale. Domain values must be positive; non-positive inputs
/// project to `0.0`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogScale {
    /// `(domain_lo, domain_hi)` — both must be positive.
    pub domain: (f64, f64),
    /// Logarithm base (typically 10 for scientific data, 2 for binary
    /// growth). Values ≤ 1 are rejected with `base = 10`.
    pub base: f64,
}

impl LogScale {
    /// Create a log-base-10 scale from an explicit domain.
    pub fn new(domain_lo: f64, domain_hi: f64) -> Self {
        Self::with_base(domain_lo, domain_hi, 10.0)
    }

    /// Create a log scale with a caller-supplied base.
    pub fn with_base(domain_lo: f64, domain_hi: f64, base: f64) -> Self {
        Self {
            domain: (domain_lo.max(f64::EPSILON), domain_hi.max(f64::EPSILON)),
            base: if base > 1.0 { base } else { 10.0 },
        }
    }
}

impl Scale for LogScale {
    fn project(&self, value: &PlottableValue) -> f32 {
        let v = value.as_number().unwrap_or(self.domain.0);
        if v <= 0.0 {
            return 0.0;
        }
        let (lo, hi) = self.domain;
        let log_v = v.log(self.base);
        let log_lo = lo.log(self.base);
        let log_hi = hi.log(self.base);
        let range = log_hi - log_lo;
        if range.abs() < f64::EPSILON {
            return 0.0;
        }
        ((log_v - log_lo) / range).clamp(0.0, 1.0) as f32
    }

    fn ticks(&self, count: usize) -> Vec<(PlottableValue, SharedString)> {
        let (lo, hi) = self.domain;
        if !(lo.is_finite() && hi.is_finite()) || lo <= 0.0 || hi <= 0.0 {
            return Vec::new();
        }
        // `f64::MAX.log2() ≈ 1024`, so decimal/natural logs stay well
        // inside `i32` range for any finite domain — but clamp anyway as
        // defense-in-depth against callers that feed pathological values.
        let log_lo_f = lo.log(self.base).floor().clamp(-1024.0, 1024.0);
        let log_hi_f = hi.log(self.base).ceil().clamp(-1024.0, 1024.0);
        let log_lo = log_lo_f as i32;
        let log_hi = log_hi_f as i32;
        let span = log_hi.saturating_sub(log_lo).max(1) as usize;
        let step = (span / count.max(1)).max(1) as i32;
        let mut out = Vec::new();
        let mut e = log_lo;
        // Hard cap on iterations so a huge span + step-of-1 still
        // terminates in bounded time; matches the `nice_ticks` defence.
        let max_iters = count.saturating_mul(4).max(32);
        for _ in 0..max_iters {
            if e > log_hi {
                break;
            }
            let v = self.base.powi(e);
            if v.is_finite() && v >= lo && v <= hi {
                out.push((
                    PlottableValue::Number(v),
                    SharedString::from(format_linear(v as f32)),
                ));
            }
            e = match e.checked_add(step) {
                Some(next) => next,
                None => break,
            };
        }
        out
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// CategoryScale
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

/// Discrete ordinal scale — one slot per named category.
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryScale {
    /// The category labels, in display order.
    pub values: Vec<SharedString>,
}

impl CategoryScale {
    /// Create a scale from a list of category labels.
    pub fn new(values: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Self {
            values: values.into_iter().map(Into::into).collect(),
        }
    }
}

impl Scale for CategoryScale {
    fn project(&self, value: &PlottableValue) -> f32 {
        let n = self.values.len();
        if n == 0 {
            return 0.0;
        }
        let idx = match value {
            PlottableValue::Category(s) => self.values.iter().position(|v| v == s),
            PlottableValue::Number(n) => {
                Some((*n as usize).min(self.values.len().saturating_sub(1)))
            }
            PlottableValue::Date(_) => None,
        };
        match idx {
            Some(i) => ((i as f32 + 0.5) / n as f32).clamp(0.0, 1.0),
            None => 0.0,
        }
    }

    fn ticks(&self, _count: usize) -> Vec<(PlottableValue, SharedString)> {
        // Category scales emit one tick per value regardless of requested
        // count — there's no meaningful notion of "every other category".
        self.values
            .iter()
            .map(|s| (PlottableValue::Category(s.clone()), s.clone()))
            .collect()
    }
}

// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
// DateScale
// ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
//
// DateScale picks a tick step from a fixed set of calendar-aware granularities
// (hour, day, week, month, quarter, year) — the largest that produces no more
// than `count` ticks. Swift Charts does the same thing. Labels format with
// `format_date_label`, which renders each granularity at the resolution a
// reader expects (`06:00` for hours, `Jan 6` for days, `Jan 2026` for months,
// `2026` for years).

const SECONDS_HOUR: u64 = 3600;
const SECONDS_DAY: u64 = 86_400;
const SECONDS_WEEK: u64 = 604_800;
const SECONDS_MONTH: u64 = 2_629_800; // ~30.44 days — calendar-month average
const SECONDS_QUARTER: u64 = SECONDS_MONTH * 3;
const SECONDS_YEAR: u64 = 31_557_600; // 365.25 days

/// Calendar-aware tick granularity.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DateGranularity {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl DateGranularity {
    fn step_secs(self) -> u64 {
        match self {
            Self::Hour => SECONDS_HOUR,
            Self::Day => SECONDS_DAY,
            Self::Week => SECONDS_WEEK,
            Self::Month => SECONDS_MONTH,
            Self::Quarter => SECONDS_QUARTER,
            Self::Year => SECONDS_YEAR,
        }
    }
}

/// Time-series scale over a `SystemTime` domain.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateScale {
    /// `(domain_lo, domain_hi)` — the instants that map to 0.0 and 1.0.
    pub domain: (SystemTime, SystemTime),
}

impl DateScale {
    /// Create a date scale from an explicit domain.
    pub fn new(domain_lo: SystemTime, domain_hi: SystemTime) -> Self {
        Self {
            domain: (domain_lo, domain_hi),
        }
    }

    /// Pick the coarsest calendar granularity whose tick count across the
    /// domain is at least `count`. Falls back to `Hour` for sub-hour ranges.
    ///
    /// Walking coarse → fine and picking the *first* granularity that yields
    /// ≥ count ticks matches Swift Charts' "reduce to monthly at 1-year
    /// zoom" behaviour: a 1-year domain asking for 5 ticks settles on Month
    /// (12 ticks) rather than Quarter (4 ticks, below the request) or Year
    /// (1 tick).
    fn pick_granularity(&self, count: usize) -> DateGranularity {
        let range_secs = duration_between(self.domain.0, self.domain.1).as_secs();
        let bucket = count.max(1) as u64;
        for g in [
            DateGranularity::Year,
            DateGranularity::Quarter,
            DateGranularity::Month,
            DateGranularity::Week,
            DateGranularity::Day,
            DateGranularity::Hour,
        ] {
            if range_secs / g.step_secs() >= bucket {
                return g;
            }
        }
        DateGranularity::Hour
    }
}

impl Scale for DateScale {
    fn project(&self, value: &PlottableValue) -> f32 {
        let t = match value {
            PlottableValue::Date(t) => *t,
            _ => return 0.0,
        };
        let lo = system_secs_since_epoch(self.domain.0);
        let hi = system_secs_since_epoch(self.domain.1);
        let v = system_secs_since_epoch(t);
        let range = hi - lo;
        if range.abs() < f64::EPSILON {
            return 0.0;
        }
        (((v - lo) / range) as f32).clamp(0.0, 1.0)
    }

    fn ticks(&self, count: usize) -> Vec<(PlottableValue, SharedString)> {
        let g = self.pick_granularity(count);
        let step = Duration::from_secs(g.step_secs());
        let range = duration_between(self.domain.0, self.domain.1);
        if range.is_zero() {
            return vec![(
                PlottableValue::Date(self.domain.0),
                SharedString::from(format_date_label(self.domain.0, g)),
            )];
        }

        let mut out = Vec::new();
        let mut t = self.domain.0;
        // Cap iterations so a pathological range_secs/step_secs can't loop
        // forever. `count * 4 + 32` matches the nice_ticks defence.
        let max_iters = count.saturating_mul(4).max(32);
        for _ in 0..max_iters {
            if duration_between(self.domain.0, t) > range {
                break;
            }
            out.push((
                PlottableValue::Date(t),
                SharedString::from(format_date_label(t, g)),
            ));
            t = match t.checked_add(step) {
                Some(next) => next,
                None => break,
            };
        }
        out
    }
}

fn system_secs_since_epoch(t: SystemTime) -> f64 {
    // `duration_since(UNIX_EPOCH)` returns `Err(_)` whenever `t` is
    // before 1970-01-01. Use the signed offset so pre-epoch timestamps
    // project distinctly — otherwise every date prior to the epoch
    // collapses onto the same tick.
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs_f64(),
        Err(e) => -e.duration().as_secs_f64(),
    }
}

fn duration_between(a: SystemTime, b: SystemTime) -> Duration {
    b.duration_since(a).unwrap_or(Duration::ZERO)
}

/// Format a `SystemTime` tick label at the given granularity.
///
/// Formats are ISO-ish and locale-independent: `2026`, `Jan 2026`,
/// `Jan 6 2026`, `06:00`. Phase 1 keeps this simple; a future pass can
/// plug in `chrono` for locale-aware strings without changing the
/// [`Scale`] API.
fn format_date_label(t: SystemTime, g: DateGranularity) -> String {
    // Use the civil calendar arithmetic from the chrono-free `days_since_epoch`
    // helper below — we only need year/month/day/hour to emit the coarse
    // labels each granularity wants.
    let secs = system_secs_since_epoch(t) as i64;
    let (year, month, day) = civil_date(secs);
    let hour = ((secs.rem_euclid(86_400)) / 3600) as u32;
    let month_name = MONTH_NAMES[(month.saturating_sub(1) as usize).min(11)];

    match g {
        DateGranularity::Year => format!("{year}"),
        DateGranularity::Quarter => {
            let q = (month - 1) / 3 + 1;
            format!("Q{q} {year}")
        }
        DateGranularity::Month => format!("{month_name} {year}"),
        DateGranularity::Week | DateGranularity::Day => {
            format!("{month_name} {day}")
        }
        DateGranularity::Hour => format!("{hour:02}:00"),
    }
}

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Convert a Unix-epoch second count to a `(year, month, day)` tuple
/// using the proleptic Gregorian calendar. Avoids pulling chrono in for
/// phase-1 scope — good enough for tick labels from 1970 to 2200.
fn civil_date(unix_secs: i64) -> (i32, u32, u32) {
    // Algorithm after Howard Hinnant's date library: split days into
    // full 400-year eras, then resolve year/month/day arithmetic on the
    // per-era day count.
    let days = unix_secs.div_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m, d)
}

#[cfg(test)]
mod tests {
    use core::prelude::v1::test;

    use super::*;

    #[test]
    fn linear_projects_lo_to_zero_and_hi_to_one() {
        let s = LinearScale::new(0.0, 100.0);
        assert!((s.project(&PlottableValue::Number(0.0)) - 0.0).abs() < 1e-6);
        assert!((s.project(&PlottableValue::Number(100.0)) - 1.0).abs() < 1e-6);
        assert!((s.project(&PlottableValue::Number(50.0)) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn linear_clamps_out_of_range() {
        let s = LinearScale::new(0.0, 100.0);
        assert!((s.project(&PlottableValue::Number(-10.0)) - 0.0).abs() < 1e-6);
        assert!((s.project(&PlottableValue::Number(200.0)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn linear_ticks_round_to_nice_values() {
        let s = LinearScale::new(0.0, 100.0);
        let ticks = s.ticks(5);
        assert!(!ticks.is_empty());
        for (v, _label) in &ticks {
            let n = v.as_number().unwrap();
            assert!((0.0..=100.0).contains(&n));
        }
    }

    #[test]
    fn log_projects_evenly_in_log_space() {
        let s = LogScale::new(1.0, 1000.0);
        // 1 → 0, 1000 → 1, 10 → 1/3, 100 → 2/3 in log10 space.
        assert!((s.project(&PlottableValue::Number(1.0)) - 0.0).abs() < 1e-4);
        assert!((s.project(&PlottableValue::Number(10.0)) - (1.0 / 3.0)).abs() < 1e-4);
        assert!((s.project(&PlottableValue::Number(100.0)) - (2.0 / 3.0)).abs() < 1e-4);
        assert!((s.project(&PlottableValue::Number(1000.0)) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn log_ticks_fall_on_powers_of_base() {
        let s = LogScale::new(1.0, 1e6);
        let ticks = s.ticks(6);
        // Each label should read as a power of 10 (1, 10, 100, ..., 1e6).
        // format_linear emits integer-free floats above 1e6, so cap domain there.
        for (v, _label) in &ticks {
            let n = v.as_number().unwrap();
            let log = n.log10();
            assert!((log.round() - log).abs() < 1e-6);
        }
    }

    #[test]
    fn log_rejects_non_positive_domain() {
        let s = LogScale::new(-10.0, 100.0);
        assert!(s.domain.0 > 0.0);
    }

    #[test]
    fn category_projects_to_slot_centers() {
        let s = CategoryScale::new(vec!["a", "b", "c", "d"]);
        let p_a = s.project(&PlottableValue::Category(SharedString::from("a")));
        let p_d = s.project(&PlottableValue::Category(SharedString::from("d")));
        assert!((p_a - 0.125).abs() < 1e-6);
        assert!((p_d - 0.875).abs() < 1e-6);
    }

    #[test]
    fn category_unknown_value_projects_to_zero() {
        let s = CategoryScale::new(vec!["a", "b"]);
        let p = s.project(&PlottableValue::Category(SharedString::from("z")));
        assert!((p - 0.0).abs() < 1e-6);
    }

    #[test]
    fn category_ticks_emit_one_per_value() {
        let s = CategoryScale::new(vec!["Mon", "Tue", "Wed"]);
        let ticks = s.ticks(2);
        assert_eq!(ticks.len(), 3);
        assert_eq!(ticks[0].1.as_ref(), "Mon");
        assert_eq!(ticks[2].1.as_ref(), "Wed");
    }

    #[test]
    fn date_projects_endpoints() {
        let lo = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
        let hi = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let s = DateScale::new(lo, hi);
        assert!((s.project(&PlottableValue::Date(lo)) - 0.0).abs() < 1e-6);
        assert!((s.project(&PlottableValue::Date(hi)) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn date_picks_finer_grain_for_short_span() {
        // 1-week domain → Day granularity, not Year.
        let lo = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let hi = lo + Duration::from_secs(SECONDS_WEEK);
        let s = DateScale::new(lo, hi);
        let g = s.pick_granularity(5);
        assert!(matches!(g, DateGranularity::Day));
    }

    #[test]
    fn date_picks_coarser_grain_for_long_span() {
        // 1-year domain with 5 ticks → Month granularity.
        let lo = UNIX_EPOCH + Duration::from_secs(1_600_000_000);
        let hi = lo + Duration::from_secs(SECONDS_YEAR);
        let s = DateScale::new(lo, hi);
        let g = s.pick_granularity(5);
        assert!(matches!(g, DateGranularity::Month));
    }

    #[test]
    fn date_label_formats_by_granularity() {
        // 2021-03-14 06:00 UTC.
        let t = UNIX_EPOCH + Duration::from_secs(1_615_702_800);
        assert_eq!(format_date_label(t, DateGranularity::Year), "2021");
        assert_eq!(format_date_label(t, DateGranularity::Month), "Mar 2021");
        assert_eq!(format_date_label(t, DateGranularity::Day), "Mar 14");
        assert_eq!(format_date_label(t, DateGranularity::Hour), "06:00");
    }

    #[test]
    fn civil_date_matches_known_dates() {
        // 1970-01-01 00:00:00 UTC.
        assert_eq!(civil_date(0), (1970, 1, 1));
        // 2000-01-01 00:00:00 UTC = 946684800.
        assert_eq!(civil_date(946_684_800), (2000, 1, 1));
        // 2021-03-14 00:00:00 UTC = 1615680000.
        assert_eq!(civil_date(1_615_680_000), (2021, 3, 14));
    }
}
