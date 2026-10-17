use super::Fixture;

pub fn get(id: &str) -> Option<Fixture> {
    #[allow(clippy::match_single_binding)]
    Some(match id {
        _ => return None,
    })
}
