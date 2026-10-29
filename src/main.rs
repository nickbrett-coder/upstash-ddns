mod config;
mod error;
mod keyring;
mod upstash;

use std::io::{self, Write};
use std::net::Ipv6Addr;
use std::str::FromStr;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use config::Config;
use upstash::Upstash;

#[derive(Parser, Debug)]
#[command(
    name = "upstash-ddns",
    version,
    about = "Manage IPv6 records stored in Upstash Redis"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Get the IPv6 address registered for a hostname.
    Get {
        /// Hostname to look up.
        hostname: String,
    },

    /// Set the IPv6 address registered for a hostname.
    Set {
        /// Hostname to update.
        hostname: String,

        /// IPv6 address to register.
        address: String,
    },

    /// Delete a hostname record.
    Delete {
        /// Hostname to delete.
        hostname: String,
    },

    /// Configure the application.
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Manage the Upstash access token.
    Credentials {
        #[command(subcommand)]
        command: CredentialsCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ConfigCommand {
    /// Create or replace the configuration file.
    Set {
        /// Upstash Redis REST URL.
        url: String,
    },

    /// Display the configuration file location.
    Path,

    /// Display the configured Upstash URL.
    Show,
}

#[derive(Subcommand, Debug)]
enum CredentialsCommand {
    /// Store an Upstash token in systemd credentials.
    Set,

    /// Test access to the token in systemd credentials.
    Get,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Config { command } => handle_config(command).await,

        Command::Credentials { command } => handle_credentials(command).await,

        Command::Get { hostname } => {
            let config = Config::load()?;
            let token = keyring::get_token()
                .await
                .context("could not retrieve Upstash token")?;

            let upstash = Upstash::new(config.url, token);

            let key = record_key(&hostname)?;

            match upstash.get(&key).await? {
                Some(address) => {
                    validate_ipv6(&address)?;
                    println!("{address}");
                }

                None => {
                    eprintln!("No IPv6 address registered for {hostname}");
                    std::process::exit(1);
                }
            }

            Ok(())
        }

        Command::Set { hostname, address } => {
            let config = Config::load()?;
            let token = keyring::get_token()
                .await
                .context("could not retrieve Upstash token")?;

            validate_hostname(&hostname)?;
            validate_ipv6(&address)?;

            let upstash = Upstash::new(config.url, token);

            let key = record_key(&hostname)?;

            upstash.set(&key, &address).await?;

            println!("{hostname} -> {address}");

            Ok(())
        }

        Command::Delete { hostname } => {
            let config = Config::load()?;
            let token = keyring::get_token()
                .await
                .context("could not retrieve Upstash token")?;

            validate_hostname(&hostname)?;

            let _upstash = Upstash::new(config.url, token);

            // We don't expose DEL yet because the minimal client only
            // implements GET and SET.
            //
            // This is intentionally left explicit rather than silently
            // pretending that deletion happened.
            anyhow::bail!(
                "delete is not implemented yet for hostname '{hostname}'"
            );
        }
    }
}

async fn handle_config(command: ConfigCommand) -> Result<()> {
    match command {
        ConfigCommand::Set { url } => {
            let config = Config { url };

            config.save()?;

            println!(
                "Configuration written to {}",
                config::config_path()?.display()
            );
        }

        ConfigCommand::Path => {
            println!("{}", config::config_path()?.display());
        }

        ConfigCommand::Show => {
            let config = Config::load()?;

            println!("{}", config.url);
        }
    }

    Ok(())
}

async fn handle_credentials(command: CredentialsCommand) -> Result<()> {
    match command {
        CredentialsCommand::Set => {
            print!("Upstash access token: ");
            io::stdout().flush()?;

            let token = rpassword::read_password()?;

            if token.trim().is_empty() {
                anyhow::bail!("token cannot be empty");
            }

            keyring::set_token(token.trim()).await?;

            println!("Upstash token stored in systemd credentials.");
        }

        CredentialsCommand::Get => {
            let token = keyring::get_token().await?;

            println!(
                "Upstash token found in systemd credentials ({} characters).",
                token.len()
            );
        }
    }

    Ok(())
}

fn record_key(hostname: &str) -> Result<String> {
    validate_hostname(hostname)?;

    Ok(format!("ddns:{hostname}"))
}

fn validate_hostname(hostname: &str) -> Result<()> {
    if hostname.is_empty() {
        anyhow::bail!("hostname cannot be empty");
    }

    if hostname.len() > 253 {
        anyhow::bail!("hostname is too long");
    }

    if hostname.contains('/') {
        anyhow::bail!("hostname cannot contain '/'");
    }

    Ok(())
}

fn validate_ipv6(address: &str) -> Result<Ipv6Addr> {
    let address = Ipv6Addr::from_str(address)
        .map_err(|_| anyhow::anyhow!("invalid IPv6 address: {address}"))?;

    Ok(address)
}

