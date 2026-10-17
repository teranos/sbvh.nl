//! When a call can be booked, and what a booking says.
//!
//! Pure: every function here takes the moment it is asked at, so the calendar
//! is pinned by tests rather than by waiting for a Tuesday. Times are unix
//! seconds on the wire and in the store; Amsterdam is only how they are said.

/// The context every meet attestation is written under.
pub const CONTEXT: &str = "meet";
/// A call booked: subject the meet id, actor the User who booked it.
pub const BOOKED: &str = "meet-book";
/// A call cancelled: subject the meet id, actor whoever cancelled it.
pub const CANCELLED: &str = "meet-cancel";

/// How much a prospect may write about what they want to talk about.
pub const ABOUT_MOST: usize = 2_000;
/// How much of a name or an address the browser may hand over.
const SAID_MOST: usize = 200;

/// The hours calls are offered in when the plugin's record names none.
pub const DEFAULT_HOURS: &str = "mon-fri 10:00-12:00,14:00-17:00";
pub const DEFAULT_MINUTES: i64 = 30;
pub const DEFAULT_NOTICE_HOURS: i64 = 24;
pub const DEFAULT_HORIZON_DAYS: i64 = 21;

const DAY_NAMES: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
const WEEKDAYS: [&str; 7] = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
const MONTHS: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September", "October",
    "November", "December",
];

/// One run of open time on some weekdays, in minutes past local midnight.
#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    /// Monday first.
    pub days: [bool; 7],
    pub from: i64,
    pub until: i64,
}

/// Everything that decides which slots are open.
#[derive(Debug, Clone, PartialEq)]
pub struct Calendar {
    pub rules: Vec<Rule>,
    /// How long one call is, and the step slots are offered at.
    pub minutes: i64,
    /// How far ahead of now the first slot is.
    pub notice_hours: i64,
    /// How many days ahead slots are offered.
    pub horizon_days: i64,
}

impl Default for Calendar {
    fn default() -> Self {
        Calendar {
            rules: hours(DEFAULT_HOURS).expect("the default hours parse"),
            minutes: DEFAULT_MINUTES,
            notice_hours: DEFAULT_NOTICE_HOURS,
            horizon_days: DEFAULT_HORIZON_DAYS,
        }
    }
}

/// A call that holds its slot: booked and not cancelled.
#[derive(Debug, Clone, PartialEq)]
pub struct Held {
    pub start: i64,
    pub minutes: i64,
}

impl Held {
    fn overlaps(&self, start: i64, minutes: i64) -> bool {
        start < self.start + self.minutes * 60 && self.start < start + minutes * 60
    }
}

/// Read the plugin's record into a calendar. Each key absent is its default;
/// each one present and wrong is refused, naming the key, rather than quietly
/// falling back — a calendar that silently offers the default hours is a
/// calendar offering hours nobody chose.
pub fn calendar(config: &std::collections::HashMap<String, String>) -> Result<Calendar, String> {
    let mut c = Calendar::default();
    if let Some(h) = config.get("hours").filter(|h| !h.trim().is_empty()) {
        c.rules = hours(h).map_err(|e| format!("hours: {e}"))?;
    }
    let number = |key: &str, least: i64, most: i64| -> Result<Option<i64>, String> {
        match config.get(key).map(|v| v.trim()).filter(|v| !v.is_empty()) {
            None => Ok(None),
            Some(v) => {
                let n: i64 = v.parse().map_err(|_| format!("{key}: {v:?} is not a whole number"))?;
                if n < least || n > most {
                    return Err(format!("{key}: {n} is outside {least}..={most}"));
                }
                Ok(Some(n))
            }
        }
    };
    if let Some(n) = number("slot_minutes", 10, 240)? {
        c.minutes = n;
    }
    if let Some(n) = number("notice_hours", 0, 24 * 14)? {
        c.notice_hours = n;
    }
    if let Some(n) = number("horizon_days", 1, 90)? {
        c.horizon_days = n;
    }
    Ok(c)
}

/// Parse open hours: rules separated by `;`, each weekdays then ranges, as in
/// `mon-fri 10:00-12:00,14:00-17:00; sat 10:00-12:00`.
pub fn hours(said: &str) -> Result<Vec<Rule>, String> {
    let mut rules = Vec::new();
    for part in said.split(';').map(str::trim).filter(|p| !p.is_empty()) {
        let (days_said, ranges_said) = part
            .split_once(char::is_whitespace)
            .ok_or_else(|| format!("{part:?} names days but no hours"))?;
        let days = weekdays(days_said)?;
        for range in ranges_said.split(',').map(str::trim).filter(|r| !r.is_empty()) {
            let (from, until) = range
                .split_once('-')
                .ok_or_else(|| format!("{range:?} is not HH:MM-HH:MM"))?;
            let (from, until) = (clock_of(from)?, clock_of(until)?);
            if until <= from {
                return Err(format!("{range:?} ends before it starts"));
            }
            rules.push(Rule { days, from, until });
        }
    }
    if rules.is_empty() {
        return Err(format!("{said:?} names no hours"));
    }
    Ok(rules)
}

fn weekdays(said: &str) -> Result<[bool; 7], String> {
    let index = |d: &str| {
        DAY_NAMES
            .iter()
            .position(|n| n.eq_ignore_ascii_case(d.trim()))
            .ok_or_else(|| format!("{d:?} is not a weekday: mon tue wed thu fri sat sun"))
    };
    let mut days = [false; 7];
    for piece in said.split(',') {
        match piece.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (index(a)?, index(b)?);
                if b < a {
                    return Err(format!("{piece:?} runs backwards"));
                }
                for d in &mut days[a..=b] {
                    *d = true;
                }
            }
            None => days[index(piece)?] = true,
        }
    }
    Ok(days)
}

fn clock_of(said: &str) -> Result<i64, String> {
    let (h, m) = said
        .trim()
        .split_once(':')
        .ok_or_else(|| format!("{said:?} is not HH:MM"))?;
    let (h, m): (i64, i64) = (
        h.parse().map_err(|_| format!("{said:?} is not HH:MM"))?,
        m.parse().map_err(|_| format!("{said:?} is not HH:MM"))?,
    );
    if !(0..=24).contains(&h) || !(0..60).contains(&m) || (h == 24 && m != 0) {
        return Err(format!("{said:?} is not a time of day"));
    }
    Ok(h * 60 + m)
}

/// Every slot open between `now` and the calendar's horizon, earliest first.
pub fn open(now: i64, cal: &Calendar, held: &[Held]) -> Vec<i64> {
    let first = now + cal.notice_hours * 3_600;
    let today = (now + amsterdam_offset(now)).div_euclid(86_400);
    let mut slots = Vec::new();
    for day in today..=today + cal.horizon_days {
        let weekday = (day + 3).rem_euclid(7) as usize;
        for rule in cal.rules.iter().filter(|r| r.days[weekday]) {
            let mut minute = rule.from;
            while minute + cal.minutes <= rule.until {
                let start = unix_of(day, minute);
                if start >= first && !held.iter().any(|h| h.overlaps(start, cal.minutes)) {
                    slots.push(start);
                }
                minute += cal.minutes;
            }
        }
    }
    slots.sort_unstable();
    slots.dedup();
    slots
}

/// Whether one slot is open now. The same question `open` answers, asked of
/// one slot, so the slot a prospect saw and the slot the plugin books are
/// decided by one rule.
pub fn is_open(start: i64, now: i64, cal: &Calendar, held: &[Held]) -> bool {
    open(now, cal, held).binary_search(&start).is_ok()
}

/// What a prospect wrote about the call: trimmed, and refused when too long
/// rather than cut, because a cut message says something they did not.
pub fn about(said: &serde_json::Value) -> Result<String, String> {
    let text = said.as_str().unwrap_or_default().trim().to_string();
    if text.chars().count() > ABOUT_MOST {
        return Err(format!("what you wrote is longer than {ABOUT_MOST} characters"));
    }
    Ok(text)
}

/// A name or an address the browser handed over: what the provider showed
/// it. Trimmed, and bounded so a stranger cannot store a novel under it.
pub fn said(v: &serde_json::Value) -> String {
    v.as_str().unwrap_or_default().trim().chars().take(SAID_MOST).collect()
}

/// The civil date of a day count since 1970-01-01, after Howard Hinnant.
fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// The day count since 1970-01-01 of a civil date, the inverse of `civil`.
fn days(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn last_sunday(y: i64, m: i64) -> i64 {
    let first_of_next = if m == 12 { days(y + 1, 1, 1) } else { days(y, m + 1, 1) };
    let last = first_of_next - 1;
    // 1970-01-01 was a Thursday: day 3 counts Sunday as 0.
    last - (last + 4).rem_euclid(7)
}

/// Seconds Amsterdam is ahead of UTC at a moment: two in summer time, which
/// runs from the last Sunday of March at 01:00 UTC to the last Sunday of
/// October at 01:00 UTC, and one otherwise.
fn amsterdam_offset(unix: i64) -> i64 {
    let (y, _, _) = civil(unix.div_euclid(86_400));
    let summer_from = last_sunday(y, 3) * 86_400 + 3_600;
    let summer_until = last_sunday(y, 10) * 86_400 + 3_600;
    if unix >= summer_from && unix < summer_until {
        7_200
    } else {
        3_600
    }
}

/// The moment a local Amsterdam day and minute is. Settled twice, because the
/// offset depends on the moment it is the offset of.
fn unix_of(day: i64, minute: i64) -> i64 {
    let local = day * 86_400 + minute * 60;
    let guess = local - amsterdam_offset(local - 3_600);
    local - amsterdam_offset(guess)
}

/// A moment as Amsterdam says it: `Tuesday 21 October 2026, 14:00`.
pub fn said_at(unix: i64) -> String {
    let local = unix + amsterdam_offset(unix);
    let day = local.div_euclid(86_400);
    let (y, m, d) = civil(day);
    let minute = local.rem_euclid(86_400) / 60;
    format!(
        "{} {d} {} {y}, {:02}:{:02}",
        WEEKDAYS[(day + 3).rem_euclid(7) as usize],
        MONTHS[(m - 1) as usize],
        minute / 60,
        minute % 60
    )
}

/// The clock a call ends at, in Amsterdam.
pub fn said_until(unix: i64) -> String {
    let local = unix + amsterdam_offset(unix);
    let minute = local.rem_euclid(86_400) / 60;
    format!("{:02}:{:02}", minute / 60, minute % 60)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 2026-10-19 is a Monday. 08:00 UTC is 10:00 in Amsterdam summer time.
    const MONDAY_10_SUMMER: i64 = 1_792_396_800;

    #[test]
    fn the_civil_date_round_trips() {
        for (y, m, d) in [(1970, 1, 1), (2026, 3, 29), (2026, 10, 25), (2026, 12, 31), (2028, 2, 29)] {
            assert_eq!(civil(days(y, m, d)), (y, m, d));
        }
    }

    #[test]
    fn ten_in_the_morning_lands_on_ten_on_both_sides_of_the_clock_change() {
        // Monday 19 October 2026 is summer time, Monday 26 October is winter.
        assert_eq!(unix_of(days(2026, 10, 19), 600), MONDAY_10_SUMMER);
        assert_eq!(said_at(MONDAY_10_SUMMER), "Monday 19 October 2026, 10:00");
        let winter = unix_of(days(2026, 10, 26), 600);
        assert_eq!(winter - MONDAY_10_SUMMER, 7 * 86_400 + 3_600);
        assert_eq!(said_at(winter), "Monday 26 October 2026, 10:00");
    }

    #[test]
    fn hours_are_read_as_days_then_ranges() {
        let rules = hours("mon-wed,fri 09:30-12:00,13:00-17:00; sat 10:00-11:00").expect("parses");
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[0].days, [true, true, true, false, true, false, false]);
        assert_eq!((rules[0].from, rules[0].until), (570, 720));
        assert_eq!(rules[2].days, [false, false, false, false, false, true, false]);
    }

    #[test]
    fn hours_that_cannot_be_read_say_what_is_wrong() {
        assert!(hours("").unwrap_err().contains("names no hours"));
        assert!(hours("mon-fri").unwrap_err().contains("no hours"));
        assert!(hours("funday 10:00-11:00").unwrap_err().contains("not a weekday"));
        assert!(hours("mon 12:00-10:00").unwrap_err().contains("ends before"));
        assert!(hours("mon 25:00-26:00").unwrap_err().contains("time of day"));
        assert!(hours("fri-mon 10:00-11:00").unwrap_err().contains("backwards"));
    }

    #[test]
    fn a_record_with_a_wrong_key_is_refused_naming_the_key() {
        let config = HashMap::from([("slot_minutes".to_string(), "five".to_string())]);
        assert!(calendar(&config).unwrap_err().starts_with("slot_minutes"));
        let config = HashMap::from([("hours".to_string(), "someday".to_string())]);
        assert!(calendar(&config).unwrap_err().starts_with("hours"));
        assert_eq!(calendar(&HashMap::new()).unwrap(), Calendar::default());
    }

    fn one_morning() -> Calendar {
        Calendar {
            rules: hours("mon 10:00-11:00").unwrap(),
            minutes: 30,
            notice_hours: 0,
            horizon_days: 6,
        }
    }

    #[test]
    fn the_open_hours_are_cut_into_slots_of_one_call_each() {
        let now = MONDAY_10_SUMMER - 86_400;
        assert_eq!(open(now, &one_morning(), &[]), vec![MONDAY_10_SUMMER, MONDAY_10_SUMMER + 1_800]);
    }

    #[test]
    fn a_held_slot_is_not_offered_and_neither_is_one_it_overlaps() {
        let now = MONDAY_10_SUMMER - 86_400;
        let held = [Held { start: MONDAY_10_SUMMER + 900, minutes: 30 }];
        assert!(open(now, &one_morning(), &held).is_empty());
        let held = [Held { start: MONDAY_10_SUMMER, minutes: 30 }];
        assert_eq!(open(now, &one_morning(), &held), vec![MONDAY_10_SUMMER + 1_800]);
    }

    #[test]
    fn nothing_inside_the_notice_is_offered() {
        let mut cal = one_morning();
        cal.notice_hours = 24;
        // Sunday 10:30 Amsterdam: Monday 10:00 is 23.5 hours away, 10:30 is 24.
        let now = MONDAY_10_SUMMER - 86_400 + 1_800;
        assert_eq!(open(now, &cal, &[]), vec![MONDAY_10_SUMMER + 1_800]);
    }

    #[test]
    fn nothing_past_the_horizon_is_offered() {
        let mut cal = one_morning();
        cal.horizon_days = 1;
        // From the Tuesday before, Monday is six days away.
        assert!(open(MONDAY_10_SUMMER - 6 * 86_400, &cal, &[]).is_empty());
    }

    #[test]
    fn a_slot_is_open_exactly_when_it_is_offered() {
        let now = MONDAY_10_SUMMER - 86_400;
        assert!(is_open(MONDAY_10_SUMMER, now, &one_morning(), &[]));
        assert!(!is_open(MONDAY_10_SUMMER + 60, now, &one_morning(), &[]));
        assert!(!is_open(MONDAY_10_SUMMER, MONDAY_10_SUMMER + 1, &one_morning(), &[]));
    }

    #[test]
    fn a_call_that_does_not_fit_in_its_range_is_not_offered() {
        let mut cal = one_morning();
        cal.minutes = 45;
        assert_eq!(open(MONDAY_10_SUMMER - 86_400, &cal, &[]), vec![MONDAY_10_SUMMER]);
    }

    #[test]
    fn what_a_prospect_wrote_is_refused_when_too_long_rather_than_cut() {
        assert_eq!(about(&serde_json::json!("  DORA audit  ")).unwrap(), "DORA audit");
        assert_eq!(about(&serde_json::Value::Null).unwrap(), "");
        let long = "é".repeat(ABOUT_MOST + 1);
        assert!(about(&serde_json::json!(long)).is_err());
        assert!(about(&serde_json::json!("é".repeat(ABOUT_MOST))).is_ok());
    }

    #[test]
    fn a_name_the_browser_hands_over_is_bounded() {
        assert_eq!(said(&serde_json::json!(" Ada ")), "Ada");
        assert_eq!(said(&serde_json::json!("x".repeat(500))).len(), 200);
        assert_eq!(said(&serde_json::json!(5)), "");
    }

    #[test]
    fn a_call_is_said_in_amsterdam_time() {
        assert_eq!(said_until(MONDAY_10_SUMMER + 1_800), "10:30");
    }
}
