//! All words and marks a chart writes that neither the spec nor the data contain.
//!
//! Pattern: Value object, given by the caller (as `Labels` of the reader). A chart never decides
//! a language: number marks, month names, time patterns and the sentences of the notes and of the
//! text alternative come from here. `Default` is English, the language of the code (ADR 0011).

/// Patterns for time axis labels (a small `strftime`: `%Y %m %d %H %I %M %S %L %p %a %b %B %%`).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeFormats {
    /// Tick not on a whole second.
    pub millisecond: String,
    /// Tick not on a whole minute.
    pub second: String,
    /// Tick not on a whole hour.
    pub minute: String,
    /// Tick not at midnight.
    pub hour: String,
    /// A day that is not the start of a week or month.
    pub day: String,
    /// The start of a week (Sunday) that is not the start of a month.
    pub week: String,
    /// The start of a month.
    pub month: String,
    /// The start of a year.
    pub year: String,
}

impl Default for TimeFormats {
    fn default() -> Self {
        let s = String::from;
        Self {
            millisecond: s(".%L"),
            second: s(":%S"),
            minute: s("%I:%M"),
            hour: s("%I %p"),
            day: s("%a %d"),
            week: s("%b %d"),
            month: s("%B"),
            year: s("%Y"),
        }
    }
}

/// The caller's words and marks.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Labels {
    /// Decimal mark.
    pub decimal: String,
    /// Thousands separator (empty for none).
    pub group: String,
    /// Minus sign.
    pub minus: String,
    /// Name of the category `null`.
    pub null_category: String,
    /// Name of the category of rows that do not have the field at all.
    pub absent_category: String,
    /// Month names.
    pub months: [String; 12],
    /// Short month names.
    pub months_short: [String; 12],
    /// Short weekday names, Sunday first.
    pub weekdays_short: [String; 7],
    /// Before noon.
    pub am: String,
    /// After noon.
    pub pm: String,
    /// Patterns of time labels.
    pub time: TimeFormats,
    /// Title of a count without field.
    pub count_title: String,
    /// `{field}` summed.
    pub sum_title: String,
    /// `{field}` averaged.
    pub mean_title: String,
    /// Smallest `{field}`.
    pub min_title: String,
    /// Largest `{field}`.
    pub max_title: String,
    /// Note under a chart: `{dropped}` of `{rows}` rows are left out.
    pub note_left_out: String,
    /// Note under a chart: `{broken}` rows break a line or an area.
    pub note_broken: String,
    /// Note under a chart: `{count}` marks are not drawn, there are too many.
    pub note_omitted: String,
    /// Note under a chart: `{field}` has `{count}` values that are `{why}`.
    pub note_issue: String,
    /// Why: no value.
    pub why_null: String,
    /// Why: not a number.
    pub why_not_number: String,
    /// Why: not a date.
    pub why_not_date: String,
    /// Why: wrong kind.
    pub why_wrong_kind: String,
    /// Why: too large.
    pub why_overflow: String,
    /// Why: negative.
    pub why_negative: String,
    /// Shown instead of a chart without data.
    pub empty: String,
    /// Shown instead of a chart that cannot be drawn: `{reason}`.
    pub cannot_draw: String,
    /// Text alternative: kind of chart, `{kind}` with `{rows}` rows.
    pub alt_summary: String,
    /// Text alternative: the range of a field, `{field}` from `{min}` to `{max}`.
    pub alt_range: String,
    /// Names of the mark kinds: bar, line, area, point, arc.
    pub kinds: [String; 5],
    /// Text alternative: `{count}` rows exist that the table does not list.
    pub alt_more: String,
    /// Header of the count column of an aggregated table.
    pub alt_series: String,
}

impl Default for Labels {
    fn default() -> Self {
        let s = String::from;
        let months = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ]
        .map(s);
        Self {
            decimal: s("."),
            group: s(","),
            minus: s("\u{2212}"),
            null_category: s("null"),
            absent_category: s("undefined"),
            months_short: months.clone().map(|m| m.chars().take(3).collect()),
            months,
            weekdays_short: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"].map(s),
            am: s("AM"),
            pm: s("PM"),
            time: TimeFormats::default(),
            count_title: s("Count of Records"),
            sum_title: s("Sum of {field}"),
            mean_title: s("Mean of {field}"),
            min_title: s("Min of {field}"),
            max_title: s("Max of {field}"),
            note_left_out: s("Rows left out: {dropped} of {rows}."),
            note_broken: s("Rows that break a line: {broken}."),
            note_omitted: s("Marks not drawn, too many: {count}."),
            note_issue: s("{field}: {why} ({count})"),
            why_null: s("missing"),
            why_not_number: s("not a number"),
            why_not_date: s("not a date"),
            why_wrong_kind: s("of the wrong kind"),
            why_overflow: s("too large to add up"),
            why_negative: s("negative"),
            empty: s("No data"),
            cannot_draw: s("This chart cannot be drawn: {reason}"),
            alt_summary: s("{kind} chart with {rows} rows."),
            alt_range: s("{field} from {min} to {max}."),
            kinds: ["Bar", "Line", "Area", "Point", "Pie or donut"].map(s),
            alt_more: s("{count} more rows are not listed."),
            alt_series: s("Series"),
        }
    }
}

impl Labels {
    /// Replace each `{name}` in `template` by the given value; unknown names stay as they are.
    pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
        let mut out = template.to_string();
        for (name, value) in values {
            out = out.replace(&format!("{{{name}}}"), value);
        }
        out
    }
}
