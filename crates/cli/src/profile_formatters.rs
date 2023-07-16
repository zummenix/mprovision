use colored::Colorize;
use mprovision::Profile;
use time::error::Format;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::OffsetDateTime;

/// Formats a profile in one line.
pub fn format_oneline(profile: &Profile) -> Result<String, Format> {
    const FMT: &[FormatItem] = format_description!("[year]-[month]-[day]");
    Ok(format!(
        "{} {} {} {}",
        profile.info.uuid.yellow(),
        OffsetDateTime::from(profile.info.expiration_date)
            .format(FMT)?
            .blue(),
        profile.info.app_identifier.green(),
        profile.info.name
    ))
}

/// Formats a profile multilined.
pub fn format_multiline(profile: &Profile) -> Result<String, Format> {
    const FMT: &[FormatItem] =
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second] UTC");
    let dates = format!(
        "{} - {}",
        OffsetDateTime::from(profile.info.creation_date).format(FMT)?,
        OffsetDateTime::from(profile.info.expiration_date).format(FMT)?,
    )
    .blue();
    Ok(format!(
        "{}\n{}\n{}\n{}",
        profile.info.uuid.yellow(),
        profile.info.app_identifier.green(),
        profile.info.name,
        dates
    ))
}
