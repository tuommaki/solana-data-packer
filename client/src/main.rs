use {
    clap::{crate_description, crate_name, crate_version, App, Arg},
    solana_clap_utils::input_parsers::keypair_of,
    solana_client::rpc_client::RpcClient,
    solana_sdk::pubkey::Pubkey,
    std::fs,
    std::str::FromStr,
};

mod uploader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg({
            let arg = Arg::with_name("config_file")
                .short("C")
                .long("config")
                .value_name("PATH")
                .takes_value(true)
                .global(true)
                .help("Configuration file to use");
            if let Some(ref config_file) = *solana_cli_config::CONFIG_FILE {
                arg.default_value(config_file)
            } else {
                arg
            }
        })
        .arg(
            Arg::with_name("program_id")
            .long("program_id")
            .value_name("program_id")
            .takes_value(true)
            .required(true)
            .help("Solana Data Bucket on-chain program ID")
        )
        .arg(
            Arg::with_name("author")
                .long("author")
                .takes_value(true)
                .required(true)
                .help("Solana author keypair path"),
        )
        .arg(
            Arg::with_name("payer")
                .long("payer")
                .takes_value(true)
                .required(true)
                .help("Solana payer keypair path"),
        )
        .arg(
            Arg::with_name("json_rpc_url")
                .long("url")
                .takes_value(true)
                .default_value("http://127.0.0.1:8899")
                .help("JSON RPC URL for the Solana cluster.  Default from the configuration file."),
        )
        .arg(
            Arg::with_name("files")
            .required(true)
            .takes_value(true)
            .multiple(true)
            .help("files to store")
        )
        .get_matches();


    let url = matches.value_of("json_rpc_url").unwrap();
    let client = RpcClient::new(url.to_string());
    let author = keypair_of(&matches, "author").expect("invalid solana author keypair");
    let payer = keypair_of(&matches, "payer").expect("invalid solana payer keypair");
    let program_id = Pubkey::from_str(matches.value_of("program_id").unwrap()).unwrap();
    let files: Vec<&str> = matches.values_of("files").unwrap().collect();

    for f in files {
        println!("saving '{}' into solana account", f);
        let f_data = fs::read(f)?;
        uploader::upload(&client, &program_id, &author, &payer, f_data.as_ref()).await?;
    }

    Ok(())
}
