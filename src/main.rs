use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, SecondsFormat, Utc};
use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use dialoguer::{theme::ColorfulTheme, Input, Password, Select};
use ethers_core::types::transaction::eip712::TypedData;
use ethers_core::utils::keccak256;
use ethers_signers::{LocalWallet, Signer};
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use solana_sdk::signature::{Keypair, Signer as SolanaSigner};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MAINNET_API_URL: &str = "https://api.tokenlayer.network/functions/v1";
const TESTNET_API_URL: &str = "https://api-testnet.tokenlayer.network/functions/v1";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

#[derive(Debug, Clone, ValueEnum)]
enum Source {
    Mainnet,
    Testnet,
}

impl Source {
    fn as_str(&self) -> &'static str {
        match self {
            Source::Mainnet => "Mainnet",
            Source::Testnet => "Testnet",
        }
    }

    fn default_base_url(&self) -> &'static str {
        match self {
            Source::Mainnet => MAINNET_API_URL,
            Source::Testnet => TESTNET_API_URL,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum WalletChainArg {
    Ethereum,
    Solana,
}

#[derive(Debug, Clone, ValueEnum)]
enum TradeDirection {
    Buy,
    Sell,
}

impl TradeDirection {
    fn as_str(&self) -> &'static str {
        match self {
            TradeDirection::Buy => "buy",
            TradeDirection::Sell => "sell",
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CandleIntervalArg {
    #[value(name = "1m")]
    OneMinute,
    #[value(name = "5m")]
    FiveMinutes,
    #[value(name = "15m")]
    FifteenMinutes,
    #[value(name = "1h")]
    OneHour,
    #[value(name = "4h")]
    FourHours,
    #[value(name = "1d")]
    OneDay,
}

impl CandleIntervalArg {
    fn as_str(&self) -> &'static str {
        match self {
            CandleIntervalArg::OneMinute => "1m",
            CandleIntervalArg::FiveMinutes => "5m",
            CandleIntervalArg::FifteenMinutes => "15m",
            CandleIntervalArg::OneHour => "1h",
            CandleIntervalArg::FourHours => "4h",
            CandleIntervalArg::OneDay => "1d",
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum TokenLaunchStageArg {
    New,
    Graduating,
    Graduated,
}

impl TokenLaunchStageArg {
    fn as_str(&self) -> &'static str {
        match self {
            TokenLaunchStageArg::New => "new",
            TokenLaunchStageArg::Graduating => "graduating",
            TokenLaunchStageArg::Graduated => "graduated",
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "token-layer")]
#[command(about = "Token Layer Rust CLI")]
struct Cli {
    #[arg(long, env = "TL_API_BASE_URL")]
    base_url: Option<String>,

    #[arg(long, env = "TL_SOURCE", value_enum, default_value_t = Source::Mainnet)]
    source: Source,

    #[arg(long, env = "TL_EXPIRES_AFTER_MS", default_value_t = 300_000)]
    expires_after: u64,

    #[arg(long, env = "TL_TOKEN")]
    token: Option<String>,

    #[arg(long, env = "TL_JWT")]
    jwt: Option<String>,

    #[arg(long, env = "TL_API_KEY")]
    api_key: Option<String>,

    #[arg(long, env = "TL_PROFILE")]
    profile: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {
        #[arg(long, default_value = "default")]
        name: String,

        #[arg(long)]
        force: bool,

        #[arg(long)]
        quick_wallet: bool,
    },
    Wallet {
        #[command(subcommand)]
        command: WalletCommands,
    },
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },
    Action {
        #[command(subcommand)]
        command: ActionCommands,
    },
    Info {
        #[command(subcommand)]
        command: InfoCommands,
    },
}

#[derive(Debug, Subcommand)]
enum ProfileCommands {
    List,
    Use {
        #[arg(long)]
        name: String,
    },
}

#[derive(Debug, Subcommand)]
enum WalletCommands {
    Add {
        #[arg(long, value_enum)]
        chain: WalletChainArg,

        #[arg(long)]
        name: String,
    },
    List,
}

#[derive(Debug, Subcommand)]
enum ActionCommands {
    Register {
        #[arg(long)]
        wallet_name: Option<String>,
        #[arg(long)]
        wallet_address: Option<String>,
        #[arg(long, default_value = "0x1")]
        signature_chain_id: String,
        #[arg(long)]
        message: Option<String>,
    },
    CreateToken {
        #[arg(long)]
        name: String,
        #[arg(long)]
        symbol: String,
        #[arg(long)]
        description: String,
        #[arg(long)]
        image: String,
        #[arg(long)]
        chain_slug: String,

        #[arg(long)]
        banner: Option<String>,
        #[arg(long)]
        video: Option<String>,
        #[arg(long, action = ArgAction::Append)]
        destination_chain: Vec<String>,
        #[arg(long)]
        pool_type: Option<String>,
        #[arg(long)]
        user_address: Option<String>,
        #[arg(long)]
        builder_code: Option<String>,
        #[arg(long)]
        builder_fee: Option<u64>,
        #[arg(long)]
        token_referral: Option<String>,
        #[arg(long, action = ArgAction::Append)]
        tag: Vec<String>,
        #[arg(long)]
        amount_in: Option<f64>,
        #[arg(long)]
        tokens_out: Option<f64>,
        #[arg(long)]
        max_amount_in: Option<f64>,

        #[arg(long)]
        website: Option<String>,
        #[arg(long)]
        twitter: Option<String>,
        #[arg(long)]
        youtube: Option<String>,
        #[arg(long)]
        discord: Option<String>,
        #[arg(long)]
        telegram: Option<String>,

        #[arg(long)]
        wallet_name: Option<String>,
        #[arg(long)]
        wallet_address: Option<String>,
        #[arg(long, default_value = "0x1")]
        signature_chain_id: String,
    },
    TradeToken {
        #[arg(long)]
        token_id: String,
        #[arg(long)]
        chain_slug: String,
        #[arg(long, value_enum)]
        direction: TradeDirection,

        #[arg(long)]
        buy_amount_usd: Option<f64>,
        #[arg(long)]
        buy_amount_token: Option<String>,
        #[arg(long)]
        sell_amount_token: Option<String>,
        #[arg(long)]
        sell_amount_usd: Option<f64>,
        #[arg(long)]
        user_address: Option<String>,
        #[arg(long)]
        builder_code: Option<String>,
        #[arg(long)]
        builder_fee: Option<u64>,
        #[arg(long)]
        token_referral: Option<String>,
    },
    TransferToken {
        #[arg(long)]
        token_id: String,
        #[arg(long)]
        recipient_address: String,
        #[arg(long)]
        amount: String,
        #[arg(long)]
        from_chain_slug: String,
        #[arg(long)]
        to_chain_slug: String,
        #[arg(long)]
        wallet_address: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
enum InfoCommands {
    GetTokensV2 {
        #[arg(long, action = ArgAction::Append)]
        hashtag: Vec<String>,
        #[arg(long)]
        keyword: Option<String>,
        #[arg(long, action = ArgAction::Append)]
        chain: Vec<String>,
        #[arg(long)]
        builder_code: Option<String>,
        #[arg(long, value_enum)]
        stage: Option<TokenLaunchStageArg>,
        #[arg(long)]
        order_by: Option<String>,
        #[arg(long)]
        order_direction: Option<String>,
        #[arg(long)]
        offset: Option<u64>,
        #[arg(long)]
        limit: Option<u64>,
        #[arg(long)]
        verified_only: Option<bool>,
    },
    GetPoolData {
        #[arg(long)]
        token_id: String,
    },
    Me {
        #[arg(long)]
        include_testnets: bool,
    },
    GetTokenTrades {
        #[arg(long)]
        token_id: String,
        #[arg(long)]
        limit: Option<u64>,
        #[arg(long)]
        offset: Option<u64>,
    },
    GetTokenTransfers {
        #[arg(long)]
        token_id: String,
        #[arg(long)]
        limit: Option<u64>,
        #[arg(long)]
        offset: Option<u64>,
    },
    GetTokenActivity {
        #[arg(long)]
        token_id: String,
        #[arg(long)]
        limit: Option<u64>,
        #[arg(long)]
        offset: Option<u64>,
        #[arg(long, action = ArgAction::Append)]
        include_activity_type: Vec<String>,
        #[arg(long, action = ArgAction::Append)]
        ignore_activity_type: Vec<String>,
        #[arg(long, action = ArgAction::Append)]
        include_activity_subtype: Vec<String>,
        #[arg(long, action = ArgAction::Append)]
        ignore_activity_subtype: Vec<String>,
    },
    GetTokenCandles {
        #[arg(long)]
        token_id: String,
        #[arg(long, value_enum)]
        candle_interval: Option<CandleIntervalArg>,
        #[arg(long)]
        venue: Option<String>,
        #[arg(long)]
        from_timestamp: Option<String>,
        #[arg(long)]
        to_timestamp: Option<String>,
        #[arg(long)]
        limit: Option<u64>,
        #[arg(long)]
        offset: Option<u64>,
        #[arg(long)]
        ascending: Option<bool>,
    },
    GetTokenStats {
        #[arg(long)]
        token_id: String,
    },
    GetTokenAbout {
        #[arg(long)]
        token_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum WalletChain {
    Ethereum,
    Solana,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredWallet {
    id: String,
    name: String,
    chain: WalletChain,
    address: String,
    private_key: String,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct WalletStore {
    wallets: Vec<StoredWallet>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "camelCase")]
enum StoredAuth {
    Wallet {
        wallet_name: String,
        signature_chain_id: String,
    },
    Jwt {
        token: String,
    },
    ApiKey {
        token: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct StoredProfile {
    name: String,
    auth: StoredAuth,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct ProfileStore {
    active_profile: Option<String>,
    profiles: Vec<StoredProfile>,
}

#[derive(Debug, Clone)]
struct AppContext {
    base_url: Option<String>,
    source: Source,
    expires_after: u64,
    token: Option<String>,
    jwt: Option<String>,
    api_key: Option<String>,
    profile: Option<StoredProfile>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let selected_profile = resolve_selected_profile(cli.profile.as_deref())?;
    let context = AppContext {
        base_url: cli.base_url.clone(),
        source: cli.source.clone(),
        expires_after: cli.expires_after,
        token: cli.token.clone(),
        jwt: cli.jwt.clone(),
        api_key: cli.api_key.clone(),
        profile: selected_profile,
    };

    match cli.command {
        Commands::Init {
            name,
            force,
            quick_wallet,
        } => {
            if quick_wallet {
                init_wallets(&name, force)?
            } else {
                init_wizard(&name, force)?
            }
        }
        Commands::Wallet { command } => handle_wallet(command)?,
        Commands::Profile { command } => handle_profile(command)?,
        Commands::Action { command } => handle_action(&context, command).await?,
        Commands::Info { command } => handle_info(&context, command).await?,
    }

    Ok(())
}

fn init_wallets(name: &str, force: bool) -> Result<()> {
    let mut store = load_wallet_store()?;

    if !force {
        let eth_name = format!("{name}-eth");
        let sol_name = format!("{name}-sol");
        let exists_eth = store.wallets.iter().any(|w| w.name == eth_name);
        let exists_sol = store.wallets.iter().any(|w| w.name == sol_name);
        if exists_eth || exists_sol {
            return Err(anyhow!(
                "Wallet names already exist. Use --force or choose a different --name."
            ));
        }
    }

    let eth = generate_ethereum_wallet(format!("{name}-eth"));
    let sol = generate_solana_wallet(format!("{name}-sol"));

    store.wallets.push(eth.clone());
    store.wallets.push(sol.clone());
    save_wallet_store(&store)?;

    println!("Created wallets:");
    println!("- {} [{}] {}", eth.name, "ethereum", eth.address);
    println!("- {} [{}] {}", sol.name, "solana", sol.address);
    println!("Stored at {}", wallet_store_path()?.display());
    Ok(())
}

fn init_wizard(default_name: &str, force: bool) -> Result<()> {
    let theme = ColorfulTheme::default();
    let mut profile_store = load_profile_store()?;
    let mut wallet_store = load_wallet_store()?;

    let profile_name: String = Input::with_theme(&theme)
        .with_prompt("Profile name")
        .default(default_name.to_string())
        .interact_text()
        .context("Failed to read profile name")?;

    if profile_exists(&profile_store, &profile_name) && !force {
        return Err(anyhow!(
            "Profile '{profile_name}' already exists. Re-run with `init --force` to overwrite."
        ));
    }

    let auth_options = vec!["Wallet (local keys)", "JWT", "API Key"];
    let auth_index = Select::with_theme(&theme)
        .with_prompt("Select auth method")
        .items(&auth_options)
        .default(0)
        .interact()
        .context("Failed to select auth method")?;

    let auth = match auth_index {
        0 => {
            let wallet_options = vec!["Create new wallet pair", "Use existing ethereum wallet"];
            let wallet_choice = Select::with_theme(&theme)
                .with_prompt("Wallet setup")
                .items(&wallet_options)
                .default(0)
                .interact()
                .context("Failed to select wallet setup")?;

            let wallet_name = if wallet_choice == 0 {
                let prefix: String = Input::with_theme(&theme)
                    .with_prompt("Wallet prefix")
                    .default(profile_name.clone())
                    .interact_text()
                    .context("Failed to read wallet prefix")?;
                let eth_name = format!("{prefix}-eth");
                let sol_name = format!("{prefix}-sol");
                if wallet_store.wallets.iter().any(|w| w.name == eth_name)
                    || wallet_store.wallets.iter().any(|w| w.name == sol_name)
                {
                    return Err(anyhow!(
                        "Wallet names '{eth_name}' or '{sol_name}' already exist. Use a different prefix."
                    ));
                }
                let eth = generate_ethereum_wallet(eth_name.clone());
                let sol = generate_solana_wallet(sol_name.clone());
                wallet_store.wallets.push(eth.clone());
                wallet_store.wallets.push(sol.clone());
                save_wallet_store(&wallet_store)?;
                println!(
                    "Created wallets: {} ({}) and {} ({})",
                    eth.name, eth.address, sol.name, sol.address
                );
                eth_name
            } else {
                let eth_wallets = wallet_store
                    .wallets
                    .iter()
                    .filter(|w| matches!(w.chain, WalletChain::Ethereum))
                    .collect::<Vec<_>>();
                if eth_wallets.is_empty() {
                    return Err(anyhow!(
                        "No ethereum wallets found. Run `tokenlayer init --quick-wallet` first."
                    ));
                }
                let labels = eth_wallets
                    .iter()
                    .map(|w| format!("{} ({})", w.name, w.address))
                    .collect::<Vec<_>>();
                let idx = Select::with_theme(&theme)
                    .with_prompt("Select ethereum wallet")
                    .items(&labels)
                    .default(0)
                    .interact()
                    .context("Failed to select ethereum wallet")?;
                eth_wallets[idx].name.clone()
            };

            let signature_chain_id: String = Input::with_theme(&theme)
                .with_prompt("Signature chain id (hex)")
                .default("0x1".to_string())
                .interact_text()
                .context("Failed to read signature chain id")?;

            StoredAuth::Wallet {
                wallet_name,
                signature_chain_id,
            }
        }
        1 => {
            let token = Password::with_theme(&theme)
                .with_prompt("JWT token")
                .allow_empty_password(false)
                .interact()
                .context("Failed to read JWT token")?;
            StoredAuth::Jwt { token }
        }
        _ => {
            let token = Password::with_theme(&theme)
                .with_prompt("API key")
                .allow_empty_password(false)
                .interact()
                .context("Failed to read API key")?;
            StoredAuth::ApiKey { token }
        }
    };

    upsert_profile(
        &mut profile_store,
        StoredProfile {
            name: profile_name.clone(),
            auth,
            created_at: Utc::now(),
        },
        true,
    );
    profile_store.active_profile = Some(profile_name.clone());
    save_profile_store(&profile_store)?;
    println!("Profile '{profile_name}' saved and set as active.");
    Ok(())
}

fn handle_profile(command: ProfileCommands) -> Result<()> {
    match command {
        ProfileCommands::List => {
            let store = load_profile_store()?;
            if store.profiles.is_empty() {
                println!("No profiles found. Run: tokenlayer init");
                return Ok(());
            }
            let active = store.active_profile.clone();
            for profile in store.profiles {
                let marker = if active.as_deref() == Some(profile.name.as_str()) {
                    "*"
                } else {
                    " "
                };
                let auth_label = match profile.auth {
                    StoredAuth::Wallet { .. } => "wallet",
                    StoredAuth::Jwt { .. } => "jwt",
                    StoredAuth::ApiKey { .. } => "apiKey",
                };
                println!(
                    "{marker}\t{}\t{}\t{}",
                    profile.name,
                    auth_label,
                    profile.created_at.to_rfc3339()
                );
            }
        }
        ProfileCommands::Use { name } => {
            let mut store = load_profile_store()?;
            if !profile_exists(&store, &name) {
                return Err(anyhow!("Profile '{name}' not found."));
            }
            store.active_profile = Some(name.clone());
            save_profile_store(&store)?;
            println!("Active profile set to '{name}'.");
        }
    }
    Ok(())
}

fn handle_wallet(command: WalletCommands) -> Result<()> {
    match command {
        WalletCommands::Add { chain, name } => {
            let mut store = load_wallet_store()?;
            if store.wallets.iter().any(|w| w.name == name) {
                return Err(anyhow!("Wallet with name '{name}' already exists."));
            }

            let wallet = match chain {
                WalletChainArg::Ethereum => generate_ethereum_wallet(name),
                WalletChainArg::Solana => generate_solana_wallet(name),
            };

            store.wallets.push(wallet.clone());
            save_wallet_store(&store)?;
            println!(
                "Added wallet: {} [{}] {}",
                wallet.name,
                wallet_chain_label(&wallet.chain),
                wallet.address
            );
        }
        WalletCommands::List => {
            let store = load_wallet_store()?;
            if store.wallets.is_empty() {
                println!("No wallets found. Run: token-layer init");
                return Ok(());
            }

            for wallet in store.wallets {
                println!(
                    "{}\t{}\t{}\t{}",
                    wallet.name,
                    wallet_chain_label(&wallet.chain),
                    wallet.address,
                    wallet.created_at.to_rfc3339()
                );
            }
        }
    }

    Ok(())
}

async fn handle_action(context: &AppContext, command: ActionCommands) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/token-layer",
        normalize_api_base_url(context.base_url.as_deref(), &context.source)
    );

    match command {
        ActionCommands::Register {
            wallet_name,
            wallet_address,
            signature_chain_id,
            message,
        } => {
            let profile_wallet = profile_wallet_defaults(context);
            let resolved_wallet_name = wallet_name.or(profile_wallet.wallet_name);
            let resolved_signature_chain_id =
                resolve_signature_chain_id(signature_chain_id, profile_wallet.signature_chain_id);
            let stored = resolve_ethereum_wallet(
                resolved_wallet_name.as_deref(),
                wallet_address.as_deref(),
            )?;
            let wallet = parse_local_wallet(&stored)?;
            let nonce = now_nonce_ms()?;
            let (body, bearer) = build_register_signed_request(
                &wallet,
                context.source.as_str(),
                context.expires_after,
                nonce,
                &resolved_signature_chain_id,
                message,
            )
            .await?;
            let response = post_json(&client, &url, body, Some(bearer)).await?;
            print_pretty_json(&response)?;
            return Ok(());
        }
        ActionCommands::CreateToken {
            name,
            symbol,
            description,
            image,
            chain_slug,
            banner,
            video,
            destination_chain,
            pool_type,
            user_address,
            builder_code,
            builder_fee,
            token_referral,
            tag,
            amount_in,
            tokens_out,
            max_amount_in,
            website,
            twitter,
            youtube,
            discord,
            telegram,
            wallet_name,
            wallet_address,
            signature_chain_id,
        } => {
            let profile_wallet = profile_wallet_defaults(context);
            let resolved_wallet_name = wallet_name.or(profile_wallet.wallet_name);
            let resolved_signature_chain_id =
                resolve_signature_chain_id(signature_chain_id, profile_wallet.signature_chain_id);
            let mut action = Map::new();
            action.insert("type".to_string(), json!("createToken"));
            action.insert("name".to_string(), json!(name));
            action.insert("symbol".to_string(), json!(symbol));
            action.insert("description".to_string(), json!(description));
            action.insert("image".to_string(), json!(image));
            action.insert("chainSlug".to_string(), json!(chain_slug));

            insert_opt(&mut action, "banner", banner.map(Value::String));
            insert_opt(&mut action, "video", video.map(Value::String));

            if !destination_chain.is_empty() {
                action.insert("destinationChains".to_string(), json!(destination_chain));
            }
            insert_opt(&mut action, "poolType", pool_type.map(Value::String));
            insert_opt(&mut action, "userAddress", user_address.map(Value::String));
            insert_opt(
                &mut action,
                "token_referral",
                token_referral.map(Value::String),
            );
            if !tag.is_empty() {
                action.insert("tags".to_string(), json!(tag));
            }
            insert_opt(&mut action, "amountIn", amount_in.map(|v| json!(v)));
            insert_opt(&mut action, "tokensOut", tokens_out.map(|v| json!(v)));
            insert_opt(&mut action, "maxAmountIn", max_amount_in.map(|v| json!(v)));

            let links = build_links(website, twitter, youtube, discord, telegram);
            insert_opt(&mut action, "links", links);

            let builder = build_builder(builder_code, builder_fee)?;
            insert_opt(&mut action, "builder", builder);

            let nonce = now_nonce_ms()?;
            if resolved_wallet_name.is_some() || wallet_address.is_some() {
                let stored = resolve_ethereum_wallet(
                    resolved_wallet_name.as_deref(),
                    wallet_address.as_deref(),
                )?;
                let wallet = parse_local_wallet(&stored)?;
                let (body, bearer) = build_create_token_signed_request(
                    &wallet,
                    context.source.as_str(),
                    context.expires_after,
                    nonce,
                    &resolved_signature_chain_id,
                    action,
                )
                .await?;
                let response = post_json(&client, &url, body, Some(bearer)).await?;
                print_pretty_json(&response)?;
                return Ok(());
            }

            let token = resolve_bearer_token(context)?;
            let body = json!({
                "source": context.source.as_str(),
                "expiresAfter": context.expires_after,
                "action": Value::Object(action),
            });
            let response = post_json(&client, &url, body, Some(token)).await?;
            print_pretty_json(&response)?;
            return Ok(());
        }
        ActionCommands::TradeToken {
            token_id,
            chain_slug,
            direction,
            buy_amount_usd,
            buy_amount_token,
            sell_amount_token,
            sell_amount_usd,
            user_address,
            builder_code,
            builder_fee,
            token_referral,
        } => {
            let mut action = Map::new();
            action.insert("type".to_string(), json!("tradeToken"));
            action.insert("tokenId".to_string(), json!(token_id));
            action.insert("chainSlug".to_string(), json!(chain_slug));
            action.insert("direction".to_string(), json!(direction.as_str()));

            insert_opt(
                &mut action,
                "buyAmountUSD",
                buy_amount_usd.map(|v| json!(v)),
            );
            insert_opt(
                &mut action,
                "buyAmountToken",
                buy_amount_token.map(Value::String),
            );
            insert_opt(
                &mut action,
                "sellAmountToken",
                sell_amount_token.map(Value::String),
            );
            insert_opt(
                &mut action,
                "sellAmountUSD",
                sell_amount_usd.map(|v| json!(v)),
            );
            insert_opt(&mut action, "userAddress", user_address.map(Value::String));
            insert_opt(
                &mut action,
                "token_referral",
                token_referral.map(Value::String),
            );

            let builder = build_builder(builder_code, builder_fee)?;
            insert_opt(&mut action, "builder", builder);

            let token = resolve_bearer_token(context)?;
            let body = json!({
                "source": context.source.as_str(),
                "expiresAfter": context.expires_after,
                "action": Value::Object(action),
            });
            let response = post_json(&client, &url, body, Some(token)).await?;
            print_pretty_json(&response)?;
            return Ok(());
        }
        ActionCommands::TransferToken {
            token_id,
            recipient_address,
            amount,
            from_chain_slug,
            to_chain_slug,
            wallet_address,
        } => {
            let token = resolve_bearer_token(context)?;
            let body = json!({
                "source": context.source.as_str(),
                "expiresAfter": context.expires_after,
                "action": {
                    "type": "transferToken",
                    "token_id": token_id,
                    "recipient_address": recipient_address,
                    "amount": amount,
                    "from_chain_slug": from_chain_slug,
                    "to_chain_slug": to_chain_slug,
                    "wallet_address": wallet_address
                },
            });
            let response = post_json(&client, &url, body, Some(token)).await?;
            print_pretty_json(&response)?;
            return Ok(());
        }
    }
}

async fn handle_info(context: &AppContext, command: InfoCommands) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/info",
        normalize_api_base_url(context.base_url.as_deref(), &context.source)
    );

    let (body, needs_auth) = match command {
        InfoCommands::GetTokensV2 {
            hashtag,
            keyword,
            chain,
            builder_code,
            stage,
            order_by,
            order_direction,
            offset,
            limit,
            verified_only,
        } => {
            let mut body = Map::new();
            body.insert("type".to_string(), json!("getTokensV2"));
            if !hashtag.is_empty() {
                body.insert("hashtags".to_string(), json!(hashtag));
            }
            insert_opt(&mut body, "keyword", keyword.map(Value::String));
            if !chain.is_empty() {
                body.insert("chains".to_string(), json!(chain));
            }
            insert_opt(&mut body, "builder_code", builder_code.map(Value::String));
            insert_opt(
                &mut body,
                "stage",
                stage.map(|value| Value::String(value.as_str().to_string())),
            );
            insert_opt(&mut body, "order_by", order_by.map(Value::String));
            insert_opt(
                &mut body,
                "order_direction",
                order_direction.map(Value::String),
            );
            insert_opt(&mut body, "offset", offset.map(|v| json!(v)));
            insert_opt(&mut body, "limit", limit.map(|v| json!(v)));
            insert_opt(&mut body, "verified_only", verified_only.map(|v| json!(v)));
            (Value::Object(body), false)
        }
        InfoCommands::GetPoolData { token_id } => (
            json!({
                "type": "getPoolData",
                "tokenId": token_id
            }),
            false,
        ),
        InfoCommands::Me { include_testnets } => (
            json!({
                "type": "me",
                "include_testnets": include_testnets,
            }),
            true,
        ),
        InfoCommands::GetTokenTrades {
            token_id,
            limit,
            offset,
        } => (
            json!({
                "type": "getTokenTrades",
                "token_id": token_id,
                "limit": limit,
                "offset": offset,
            }),
            false,
        ),
        InfoCommands::GetTokenTransfers {
            token_id,
            limit,
            offset,
        } => (
            json!({
                "type": "getTokenTransfers",
                "token_id": token_id,
                "limit": limit,
                "offset": offset,
            }),
            false,
        ),
        InfoCommands::GetTokenActivity {
            token_id,
            limit,
            offset,
            include_activity_type,
            ignore_activity_type,
            include_activity_subtype,
            ignore_activity_subtype,
        } => {
            let mut body = Map::new();
            body.insert("type".to_string(), json!("getTokenActivity"));
            body.insert("token_id".to_string(), json!(token_id));
            insert_opt(&mut body, "limit", limit.map(|v| json!(v)));
            insert_opt(&mut body, "offset", offset.map(|v| json!(v)));
            if !include_activity_type.is_empty() {
                body.insert(
                    "include_activity_types".to_string(),
                    json!(include_activity_type),
                );
            }
            if !ignore_activity_type.is_empty() {
                body.insert(
                    "ignore_activity_types".to_string(),
                    json!(ignore_activity_type),
                );
            }
            if !include_activity_subtype.is_empty() {
                body.insert(
                    "include_activity_subtypes".to_string(),
                    json!(include_activity_subtype),
                );
            }
            if !ignore_activity_subtype.is_empty() {
                body.insert(
                    "ignore_activity_subtypes".to_string(),
                    json!(ignore_activity_subtype),
                );
            }
            (Value::Object(body), false)
        }
        InfoCommands::GetTokenCandles {
            token_id,
            candle_interval,
            venue,
            from_timestamp,
            to_timestamp,
            limit,
            offset,
            ascending,
        } => {
            let mut body = Map::new();
            body.insert("type".to_string(), json!("getTokenCandles"));
            body.insert("token_id".to_string(), json!(token_id));
            insert_opt(
                &mut body,
                "candle_interval",
                candle_interval.map(|v| json!(v.as_str())),
            );
            insert_opt(&mut body, "venue", venue.map(Value::String));
            insert_opt(
                &mut body,
                "from_timestamp",
                from_timestamp.map(Value::String),
            );
            insert_opt(&mut body, "to_timestamp", to_timestamp.map(Value::String));
            insert_opt(&mut body, "limit", limit.map(|v| json!(v)));
            insert_opt(&mut body, "offset", offset.map(|v| json!(v)));
            insert_opt(&mut body, "ascending", ascending.map(|v| json!(v)));
            (Value::Object(body), false)
        }
        InfoCommands::GetTokenStats { token_id } => (
            json!({
                "type": "getTokenStats",
                "token_id": token_id,
            }),
            false,
        ),
        InfoCommands::GetTokenAbout { token_id } => (
            json!({
                "type": "getTokenAbout",
                "token_id": token_id,
            }),
            false,
        ),
    };

    let bearer = if needs_auth {
        Some(resolve_bearer_token(context)?)
    } else {
        resolve_optional_bearer_token(context)
    };

    let response = post_json(&client, &url, body, bearer).await?;
    print_pretty_json(&response)?;
    Ok(())
}

async fn post_json(
    client: &reqwest::Client,
    url: &str,
    body: Value,
    bearer: Option<String>,
) -> Result<Value> {
    let mut request = client.post(url).json(&body);
    if let Some(token) = bearer {
        request = request.bearer_auth(token);
    }

    let response = request.send().await.context("Failed to send request")?;
    let status = response.status();
    let text = response
        .text()
        .await
        .context("Failed to read response body")?;
    let json_payload = serde_json::from_str::<Value>(&text).unwrap_or_else(|_| {
        json!({
            "error": "non_json_response",
            "status": status.as_u16(),
            "body": text,
        })
    });

    if status.is_success() {
        Ok(json_payload)
    } else {
        Err(anyhow!("Request failed ({status}): {json_payload}"))
    }
}

fn now_nonce_ms() -> Result<u64> {
    let ts = Utc::now().timestamp_millis();
    if ts < 0 {
        return Err(anyhow!("System clock produced negative timestamp"));
    }
    Ok(ts as u64)
}

fn parse_chain_id_hex(signature_chain_id: &str) -> Result<u64> {
    let trimmed = signature_chain_id.trim();
    let without_prefix = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
        .ok_or_else(|| anyhow!("signature chain id must be hex (example: 0x1)"))?;
    u64::from_str_radix(without_prefix, 16)
        .with_context(|| format!("Invalid signature chain id: {signature_chain_id}"))
}

fn normalize_hex_address(address: &str) -> String {
    address.trim().to_lowercase()
}

fn hash_string_array(values: &[String]) -> String {
    if values.is_empty() {
        return format!("0x{}", hex::encode(keccak256("".as_bytes())));
    }
    let joined = values.join("|");
    format!("0x{}", hex::encode(keccak256(joined.as_bytes())))
}

fn hash_links(links: Option<&Value>) -> String {
    let links_obj = links.and_then(Value::as_object);
    let website = links_obj
        .and_then(|o| o.get("website"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let twitter = links_obj
        .and_then(|o| o.get("twitter"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let youtube = links_obj
        .and_then(|o| o.get("youtube"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let discord = links_obj
        .and_then(|o| o.get("discord"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let telegram = links_obj
        .and_then(|o| o.get("telegram"))
        .and_then(Value::as_str)
        .unwrap_or("");

    let canonical = format!("{website}|{twitter}|{youtube}|{discord}|{telegram}");
    format!("0x{}", hex::encode(keccak256(canonical.as_bytes())))
}

fn value_to_string(value: Option<&Value>) -> String {
    match value {
        None => "0".to_string(),
        Some(v) if v.is_null() => "0".to_string(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        Some(other) => other.to_string(),
    }
}

fn resolve_ethereum_wallet(
    wallet_name: Option<&str>,
    wallet_address: Option<&str>,
) -> Result<StoredWallet> {
    let store = load_wallet_store()?;
    let mut wallets = store
        .wallets
        .into_iter()
        .filter(|w| matches!(w.chain, WalletChain::Ethereum))
        .collect::<Vec<_>>();

    if wallets.is_empty() {
        return Err(anyhow!(
            "No Ethereum wallets found. Run `token-layer init` or `token-layer wallet add --chain ethereum`."
        ));
    }

    if let Some(name) = wallet_name {
        return wallets
            .into_iter()
            .find(|w| w.name == name)
            .ok_or_else(|| anyhow!("Ethereum wallet named '{name}' not found"));
    }

    if let Some(address) = wallet_address {
        let target = normalize_hex_address(address);
        return wallets
            .into_iter()
            .find(|w| normalize_hex_address(&w.address) == target)
            .ok_or_else(|| anyhow!("Ethereum wallet with address '{address}' not found"));
    }

    wallets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    wallets
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("No Ethereum wallets available"))
}

fn parse_local_wallet(stored: &StoredWallet) -> Result<LocalWallet> {
    stored
        .private_key
        .parse::<LocalWallet>()
        .with_context(|| format!("Failed to parse private key for wallet '{}'", stored.name))
}

fn build_register_message(
    address: &str,
    nonce_ms: u64,
    chain_id: u64,
    expires_after: u64,
) -> String {
    let issued_at = DateTime::from_timestamp_millis(nonce_ms as i64)
        .map(|dt| dt.to_rfc3339_opts(SecondsFormat::Millis, true))
        .unwrap_or_else(|| Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));
    let expiration = DateTime::from_timestamp_millis((nonce_ms + expires_after) as i64)
        .map(|dt| dt.to_rfc3339_opts(SecondsFormat::Millis, true))
        .unwrap_or_else(|| Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true));

    format!(
        "app.tokenlayer.network wants you to sign in with your Ethereum account:\n{address}\n\nTokenLayer register timestamp: {nonce_ms}\n\nURI: https://app.tokenlayer.network\nVersion: 1\nChain ID: {chain_id}\nNonce: {nonce_ms}\nIssued At: {issued_at}\nExpiration Time: {expiration}"
    )
}

async fn build_register_signed_request(
    wallet: &LocalWallet,
    source: &str,
    expires_after: u64,
    nonce: u64,
    signature_chain_id: &str,
    message_override: Option<String>,
) -> Result<(Value, String)> {
    let chain_id_num = parse_chain_id_hex(signature_chain_id)?;
    let wallet_address = format!("{:#x}", wallet.address());
    let message = message_override.unwrap_or_else(|| {
        build_register_message(&wallet_address, nonce, chain_id_num, expires_after)
    });
    let action_signature = wallet
        .sign_message(message.clone())
        .await
        .context("Failed to sign register message")?;

    let typed_data_value = json!({
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "RegisterAction": [
                {"name": "type", "type": "string"},
                {"name": "method", "type": "string"},
                {"name": "source", "type": "string"},
                {"name": "nonce", "type": "uint64"},
                {"name": "expiresAfter", "type": "uint64"}
            ]
        },
        "primaryType": "RegisterAction",
        "domain": {
            "name": "TokenLayerSignTransaction",
            "version": "1",
            "chainId": chain_id_num.to_string(),
            "verifyingContract": ZERO_ADDRESS
        },
        "message": {
            "type": "register",
            "method": "web3",
            "source": source,
            "nonce": nonce.to_string(),
            "expiresAfter": expires_after.to_string()
        }
    });

    let typed_data: TypedData = serde_json::from_value(typed_data_value)
        .context("Failed to construct register typed data")?;
    let typed_signature = wallet
        .sign_typed_data(&typed_data)
        .await
        .context("Failed to sign register typed data")?;

    let body = json!({
        "source": source,
        "nonce": nonce,
        "expiresAfter": expires_after,
        "signature": typed_signature.to_string(),
        "signatureChainId": signature_chain_id,
        "action": {
            "type": "register",
            "method": "web3",
            "message": message,
            "signature": action_signature.to_string()
        }
    });

    Ok((body, wallet_address))
}

async fn build_create_token_signed_request(
    wallet: &LocalWallet,
    source: &str,
    expires_after: u64,
    nonce: u64,
    signature_chain_id: &str,
    mut action: Map<String, Value>,
) -> Result<(Value, String)> {
    let chain_id_num = parse_chain_id_hex(signature_chain_id)?;
    let wallet_address = format!("{:#x}", wallet.address());
    if !action.contains_key("userAddress") {
        action.insert(
            "userAddress".to_string(),
            Value::String(wallet_address.clone()),
        );
    }

    let destination_chains = action
        .get("destinationChains")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let tags = action
        .get("tags")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let links_hash = hash_links(action.get("links"));
    let destination_hash = hash_string_array(&destination_chains);
    let tags_hash = hash_string_array(&tags);
    let builder = action.get("builder").and_then(Value::as_object);
    let builder_code = builder
        .and_then(|b| b.get("code"))
        .and_then(Value::as_str)
        .unwrap_or(ZERO_ADDRESS);
    let builder_fee = builder
        .and_then(|b| b.get("fee"))
        .map(|v| value_to_string(Some(v)))
        .unwrap_or_else(|| "0".to_string());
    let token_referral = action
        .get("token_referral")
        .and_then(Value::as_str)
        .unwrap_or(ZERO_ADDRESS);
    let pool_type = action
        .get("poolType")
        .and_then(Value::as_str)
        .unwrap_or("meme");
    let user_address = action
        .get("userAddress")
        .and_then(Value::as_str)
        .unwrap_or(&wallet_address);

    let typed_data_value = json!({
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "CreateTokenAction": [
                {"name": "type", "type": "string"},
                {"name": "source", "type": "string"},
                {"name": "name", "type": "string"},
                {"name": "symbol", "type": "string"},
                {"name": "description", "type": "string"},
                {"name": "image", "type": "string"},
                {"name": "banner", "type": "string"},
                {"name": "video", "type": "string"},
                {"name": "chainSlug", "type": "string"},
                {"name": "destinationChainsHash", "type": "bytes32"},
                {"name": "poolType", "type": "string"},
                {"name": "userAddress", "type": "address"},
                {"name": "builderCode", "type": "address"},
                {"name": "builderFee", "type": "uint256"},
                {"name": "tokenReferral", "type": "address"},
                {"name": "tagsHash", "type": "bytes32"},
                {"name": "linksHash", "type": "bytes32"},
                {"name": "tokenType", "type": "string"},
                {"name": "amountIn", "type": "string"},
                {"name": "tokensOut", "type": "string"},
                {"name": "maxAmountIn", "type": "string"},
                {"name": "nonce", "type": "uint64"},
                {"name": "expiresAfter", "type": "uint64"}
            ]
        },
        "primaryType": "CreateTokenAction",
        "domain": {
            "name": "TokenLayerSignTransaction",
            "version": "1",
            "chainId": chain_id_num.to_string(),
            "verifyingContract": ZERO_ADDRESS
        },
        "message": {
            "type": "createToken",
            "source": source,
            "name": action.get("name").and_then(Value::as_str).unwrap_or(""),
            "symbol": action.get("symbol").and_then(Value::as_str).unwrap_or(""),
            "description": action.get("description").and_then(Value::as_str).unwrap_or(""),
            "image": action.get("image").and_then(Value::as_str).unwrap_or(""),
            "banner": action.get("banner").and_then(Value::as_str).unwrap_or(""),
            "video": action.get("video").and_then(Value::as_str).unwrap_or(""),
            "chainSlug": action.get("chainSlug").and_then(Value::as_str).unwrap_or(""),
            "destinationChainsHash": destination_hash,
            "poolType": pool_type,
            "userAddress": user_address,
            "builderCode": builder_code,
            "builderFee": builder_fee,
            "tokenReferral": token_referral,
            "tagsHash": tags_hash,
            "linksHash": links_hash,
            "tokenType": "coin",
            "amountIn": value_to_string(action.get("amountIn")),
            "tokensOut": value_to_string(action.get("tokensOut")),
            "maxAmountIn": value_to_string(action.get("maxAmountIn")),
            "nonce": nonce.to_string(),
            "expiresAfter": expires_after.to_string()
        }
    });
    let typed_data: TypedData = serde_json::from_value(typed_data_value)
        .context("Failed to construct createToken typed data")?;
    let signature = wallet
        .sign_typed_data(&typed_data)
        .await
        .context("Failed to sign createToken typed data")?;

    let body = json!({
        "source": source,
        "nonce": nonce,
        "expiresAfter": expires_after,
        "signature": signature.to_string(),
        "signatureChainId": signature_chain_id,
        "action": Value::Object(action),
    });

    Ok((body, wallet_address))
}

fn build_links(
    website: Option<String>,
    twitter: Option<String>,
    youtube: Option<String>,
    discord: Option<String>,
    telegram: Option<String>,
) -> Option<Value> {
    let mut links = Map::new();
    insert_opt(&mut links, "website", website.map(Value::String));
    insert_opt(&mut links, "twitter", twitter.map(Value::String));
    insert_opt(&mut links, "youtube", youtube.map(Value::String));
    insert_opt(&mut links, "discord", discord.map(Value::String));
    insert_opt(&mut links, "telegram", telegram.map(Value::String));

    if links.is_empty() {
        None
    } else {
        Some(Value::Object(links))
    }
}

fn build_builder(builder_code: Option<String>, builder_fee: Option<u64>) -> Result<Option<Value>> {
    match (builder_code, builder_fee) {
        (None, None) => Ok(None),
        (Some(code), fee) => {
            let mut builder = Map::new();
            builder.insert("code".to_string(), Value::String(code));
            if let Some(fee_value) = fee {
                builder.insert("fee".to_string(), json!(fee_value));
            }
            Ok(Some(Value::Object(builder)))
        }
        (None, Some(_)) => Err(anyhow!("--builder-fee requires --builder-code")),
    }
}

fn insert_opt(target: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(v) = value {
        if !v.is_null() {
            target.insert(key.to_string(), v);
        }
    }
}

fn normalize_api_base_url(raw: Option<&str>, source: &Source) -> String {
    let mut base = raw
        .unwrap_or(source.default_base_url())
        .trim_end_matches('/')
        .to_string();

    if let Some(stripped) = base.strip_suffix("/token-layer") {
        base = stripped.to_string();
    }
    if let Some(stripped) = base.strip_suffix("/info") {
        base = stripped.to_string();
    }

    base
}

struct ProfileWalletDefaults {
    wallet_name: Option<String>,
    signature_chain_id: Option<String>,
}

fn profile_wallet_defaults(context: &AppContext) -> ProfileWalletDefaults {
    match &context.profile {
        Some(StoredProfile {
            auth:
                StoredAuth::Wallet {
                    wallet_name,
                    signature_chain_id,
                },
            ..
        }) => ProfileWalletDefaults {
            wallet_name: Some(wallet_name.clone()),
            signature_chain_id: Some(signature_chain_id.clone()),
        },
        _ => ProfileWalletDefaults {
            wallet_name: None,
            signature_chain_id: None,
        },
    }
}

fn resolve_signature_chain_id(
    requested_signature_chain_id: String,
    profile_signature_chain_id: Option<String>,
) -> String {
    if requested_signature_chain_id == "0x1" {
        return profile_signature_chain_id.unwrap_or(requested_signature_chain_id);
    }
    requested_signature_chain_id
}

fn resolve_optional_bearer_token(context: &AppContext) -> Option<String> {
    context
        .token
        .clone()
        .or_else(|| context.jwt.clone())
        .or_else(|| context.api_key.clone())
        .or_else(|| env::var("TL_JWT").ok())
        .or_else(|| env::var("TL_API_KEY").ok())
        .or_else(|| match &context.profile {
            Some(StoredProfile {
                auth: StoredAuth::Jwt { token },
                ..
            }) => Some(token.clone()),
            Some(StoredProfile {
                auth: StoredAuth::ApiKey { token },
                ..
            }) => Some(token.clone()),
            _ => None,
        })
        .filter(|t| !t.trim().is_empty())
}

fn resolve_bearer_token(context: &AppContext) -> Result<String> {
    resolve_optional_bearer_token(context).ok_or_else(|| {
        anyhow!("Missing auth token. Provide --token, --jwt, --api-key or set TL_JWT/TL_API_KEY.")
    })
}

fn print_pretty_json(value: &Value) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).context("Failed to format JSON output")?
    );
    Ok(())
}

fn generate_ethereum_wallet(name: String) -> StoredWallet {
    let wallet = LocalWallet::new(&mut thread_rng());
    let private_key = format!("0x{}", hex::encode(wallet.signer().to_bytes()));

    StoredWallet {
        id: format!("eth-{}", Utc::now().timestamp_millis()),
        name,
        chain: WalletChain::Ethereum,
        address: format!("{:?}", wallet.address()),
        private_key,
        created_at: Utc::now(),
    }
}

fn generate_solana_wallet(name: String) -> StoredWallet {
    let keypair = Keypair::new();
    let private_key = bs58::encode(keypair.to_bytes()).into_string();

    StoredWallet {
        id: format!("sol-{}", Utc::now().timestamp_millis()),
        name,
        chain: WalletChain::Solana,
        address: keypair.pubkey().to_string(),
        private_key,
        created_at: Utc::now(),
    }
}

fn wallet_chain_label(chain: &WalletChain) -> &'static str {
    match chain {
        WalletChain::Ethereum => "ethereum",
        WalletChain::Solana => "solana",
    }
}

fn resolve_selected_profile(requested_profile: Option<&str>) -> Result<Option<StoredProfile>> {
    let store = load_profile_store()?;
    if store.profiles.is_empty() {
        return Ok(None);
    }

    if let Some(name) = requested_profile {
        return store
            .profiles
            .iter()
            .find(|p| p.name == name)
            .cloned()
            .map(Some)
            .ok_or_else(|| anyhow!("Profile '{name}' not found."));
    }

    if let Some(active) = store.active_profile {
        if let Some(profile) = store.profiles.iter().find(|p| p.name == active) {
            return Ok(Some(profile.clone()));
        }
    }

    Ok(store.profiles.first().cloned())
}

fn profile_exists(store: &ProfileStore, name: &str) -> bool {
    store.profiles.iter().any(|p| p.name == name)
}

fn upsert_profile(store: &mut ProfileStore, profile: StoredProfile, overwrite: bool) {
    if let Some(existing) = store.profiles.iter_mut().find(|p| p.name == profile.name) {
        if overwrite {
            *existing = profile;
        }
        return;
    }
    store.profiles.push(profile);
}

fn profile_store_path() -> Result<PathBuf> {
    Ok(config_root_dir()?.join("profiles.json"))
}

fn wallet_store_path() -> Result<PathBuf> {
    Ok(config_root_dir()?.join("wallets.json"))
}

fn config_root_dir() -> Result<PathBuf> {
    if let Ok(path) = env::var("TL_CLI_HOME") {
        return Ok(PathBuf::from(path));
    }
    if let Some(home) = dirs::home_dir() {
        return Ok(home.join(".token-layer-cli"));
    }
    Ok(env::current_dir()?.join(".token-layer-cli"))
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }
    Ok(())
}

fn load_wallet_store() -> Result<WalletStore> {
    let path = wallet_store_path()?;
    if !path.exists() {
        return Ok(WalletStore::default());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let store = serde_json::from_str::<WalletStore>(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(store)
}

fn save_wallet_store(store: &WalletStore) -> Result<()> {
    let path = wallet_store_path()?;
    ensure_parent_dir(&path)?;
    let content =
        serde_json::to_string_pretty(store).context("Failed to serialize wallet store")?;
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

fn load_profile_store() -> Result<ProfileStore> {
    let path = profile_store_path()?;
    if !path.exists() {
        return Ok(ProfileStore::default());
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let store = serde_json::from_str::<ProfileStore>(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(store)
}

fn save_profile_store(store: &ProfileStore) -> Result<()> {
    let path = profile_store_path()?;
    ensure_parent_dir(&path)?;
    let content =
        serde_json::to_string_pretty(store).context("Failed to serialize profile store")?;
    fs::write(&path, content).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}
