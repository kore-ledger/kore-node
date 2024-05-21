use crate::settings::KoreSettings;
use config::Config;

use super::params::Params;

pub fn build_config(env: bool, json: &str, yaml: &str, toml: &str) -> KoreSettings {
    let mut config = Config::builder();
    // ENV
    if env {
        config = config.add_source(config::Environment::with_prefix("KORE").separator("_"));
    }
    // JSON
    if !json.is_empty() {
        config = config.add_source(config::File::with_name(json));
    }

    // YAML
    if !yaml.is_empty() {
        config = config.add_source(config::File::with_name(yaml));
    }

    // TOML
	if !toml.is_empty() {
        config = config.add_source(config::File::with_name(toml));
    }

	let config = config.build().map_err(|e| {
		println!("Error building config: {}", e);
	}).unwrap();

	let params: Params = config.try_deserialize().map_err(|e| {
		println!("Error try deserialize config: {}", e);
	}).unwrap();

	KoreSettings::from(params)
}

#[cfg(test)]
mod tests {
    use super::build_config;
	fn create_file(name: &str, route: &str, data: &str) {

	}

    #[test]
	fn test_env() {
		build_config(true, "", "", "");
	} 

    #[test]
	fn test_json() {
		build_config(false, "./env/env.json", "", "");
	}

    #[test]
	fn yaml() {
		build_config(false, "", "./env/env.yaml", "");
	} 

    #[test]
	fn toml() {
		build_config(false, "", "", "./env/env.toml");
	} 
}