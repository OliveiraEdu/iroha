use eyre::WrapErr;
use iroha_client::client::Client;
use iroha_config::client::Configuration as ClientConfiguration;
use iroha_crypto::prelude::*;
use iroha_data_model::prelude::*;
use iroha_data_model::transaction::Executable;
use iroha_data_model::isi::RegisterBox;
use iroha_primitives::small::SmallStr;
use std::error::Error;
use std::fs::File;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    // Load configuration from config.json
    let config_loc = "/root/Git/iroha/configs/peer/config.json";
    let file = File::open(config_loc).expect("Unable to load the configuration file");
    let config: Result<ClientConfiguration, serde_json::Error> =
        serde_json::from_reader(file);

    match config {
        Ok(config) => {
            // Successfully parsed the configuration
            let iroha_client: Client = Client::new(&config)?;

            // Set up key pairs and account information
            let kp = KeyPair::new(
                PublicKey::from_str(
                    r#"ed01207233bfc89dcbd68c19fde6ce6158225298ec1131b6a130d1aeb454c1ab5183c0"#,
                )?,
                PrivateKey::from_hex(
                    Algorithm::Ed25519,
                    "9ac47abf59b356e0bd7dcbbbb4dec080e302156a48ca907e47cb6aea1d32719e7233bfc89dcbd68c19fde6ce6158225298ec1131b6a130d1aeb454c1ab5183c0"
                        .into(),
                )?,
            )?;
            let (public_key, private_key) = kp.clone().into();
            let account_id: AccountId = "alice@wonderland".parse()?;

            // Create ClientConfiguration
            let config = ClientConfiguration {
                public_key,
                private_key,
                account_id,
                torii_api_url: SmallStr::from_string(iroha_config::torii::uri::DEFAULT_API_URL.to_owned()),
                // Add other fields as needed
                // Use appropriate values for the missing fields or provide default values
                add_transaction_nonce: Default::default(), // Add appropriate value
                basic_auth: Default::default(), // Add appropriate value
                torii_telemetry_url: Default::default(), // Add appropriate value
                transaction_limits: Default::default(),
                transaction_status_timeout_ms: Default::default(),
                transaction_time_to_live_ms: : Default::default()
                // Add other fields as needed
            };

            // Create a domain ID
            let looking_glass: DomainId = "looking_glass".parse()?;

            // Create an ISI
            let create_looking_glass = RegisterBox::new(Domain::new(looking_glass));

            // Prepare a transaction
            let metadata = UnlimitedMetadata::default();
            let instructions: Vec<Instruction> = vec![create_looking_glass.into()];
            let tx = iroha_client
                .build_transaction(
                    Executable::Instructions(instructions),
                    metadata,
                )
                .wrap_err("Error building a domain registration transaction")?;

            // Submit a prepared domain registration transaction
            iroha_client.submit_transaction(tx)
                .wrap_err("Failed to submit transaction")?;

            Ok(())
        }
        Err(err) => {
            // Print the error message and stop the program
            eprintln!("Failed to parse the configuration file: {}", err);
            std::process::exit(1);
        }
    }
}
