pub type Date = chrono::DateTime<chrono::Utc>;

pub trait DateFormatting {
    fn format_yyyymmddthhmmssz(&self) -> String;
    fn format_yyyymmdd(&self) -> String;
}

impl DateFormatting for Date {
    fn format_yyyymmddthhmmssz(&self) -> String {
        const YYYYMMDDTHHMMSSZ: &str = "%Y%m%dT%H%M%SZ";
        self.format(YYYYMMDDTHHMMSSZ).to_string()
    }

    fn format_yyyymmdd(&self) -> String {
        const YYYYMMDD: &str = "%Y%m%d";
        self.format(YYYYMMDD).to_string()
    }
}
