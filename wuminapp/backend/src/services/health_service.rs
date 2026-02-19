use crate::{app_state::AppState, models::HealthData};

pub fn get_health(state: &AppState) -> HealthData {
    HealthData {
        service: state.service,
        version: state.version,
        status: "UP",
    }
}
