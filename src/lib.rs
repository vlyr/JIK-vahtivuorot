pub mod event;
pub mod utils;

pub mod parser;
pub mod wilma_client;

#[cfg(test)]
mod tests {
    use super::utils::*;

    #[test]
    fn start_to_number() {
        assert_eq!(start_str_to_number("8:45"), 525);
    }

    #[test]
    fn number_to_start() {
        assert_eq!(format_time(525), "8:45");
        assert_eq!(format_time(540), "9:00");
    }
}
