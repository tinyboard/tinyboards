use porpl_utils::{
    settings::structs::Settings,
    version::VERSION,
};

pub fn build_user_agent(settings: &Settings) -> String {
    format!(
        "Porpl/{}; +{}",
        VERSION,
        settings.get_protocol_and_hostname()
    )
}