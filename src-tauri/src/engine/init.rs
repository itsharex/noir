use std::path::PathBuf;

use deadpool_postgres::{
    Config as PsqlConfig, ManagerConfig as PsqlManagerConfig, RecyclingMethod, SslMode,
};
use deadpool_sqlite::Config as SqliteConfig;
use mysql::{Opts, OptsBuilder, Pool as MysqlPool};
use openssl::ssl::{SslConnector, SslFiletype, SslMethod, SslVerifyMode};
use postgres::NoTls;
use postgres_openssl::MakeTlsConnector;

use crate::{
    engine::types::{
        config::{ConnectionConfig, ConnectionOpts, ConnectionPool, Dialect, Mode},
        connection::InitiatedConnection,
    },
    utils::error::Error,
};

pub async fn init_conn(cfg: ConnectionConfig) -> Result<InitiatedConnection, Error> {
    match &cfg.dialect {
        Dialect::Mysql => {
            if cfg.mode == Mode::File {
                return Err(anyhow::anyhow!("File mode is not supported for Mysql").into());
            }
            let builder = OptsBuilder::new();
            let builder = builder
                .from_hash_map(&cfg.credentials)?
                .tcp_connect_timeout(Some(std::time::Duration::from_secs(15)))
                .prefer_socket(cfg.mode == Mode::Socket);
            let opts = Opts::from(builder);
            let cloned = opts.clone();
            match MysqlPool::new(opts.clone()) {
                Ok(pool) => {
                    let schema = cloned.get_db_name().unwrap_or("");
                    Ok(InitiatedConnection {
                        config: cfg.clone(),
                        pool: ConnectionPool::Mysql(pool),
                        opts: ConnectionOpts::Mysql(opts),
                        schema: schema.to_string(),
                    })
                }
                Err(e) => Err(Error::Mysql(e)),
            }
        }
        Dialect::Postgresql => {
            if cfg.mode == Mode::File {
                return Err(anyhow::anyhow!("File mode is not supported for Postgresql").into());
            }
            let mut config = PsqlConfig::new();
            config.user = cfg.credentials.get("user").cloned();
            config.password = cfg.credentials.get("password").cloned();
            config.dbname = cfg.credentials.get("db_name").cloned();
            config.host = cfg.credentials.get("host").cloned();
            config.port = cfg
                .credentials
                .get("port")
                .cloned()
                .map(|p| p.parse::<u16>().expect("Port should be a valid number"));
            config.connect_timeout = Some(std::time::Duration::from_secs(15));
            config.manager = Some(PsqlManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            });
            let ssl_mode = cfg.credentials.get("ssl_mode").unwrap();
            config.ssl_mode = match ssl_mode.as_str() {
                "prefer" => Some(SslMode::Prefer),
                "require" => Some(SslMode::Require),
                _ => Some(SslMode::Disable),
            };
            let rt = Some(deadpool_postgres::Runtime::Tokio1);
            let ca_cert = cfg
                .credentials
                .get("ca_cert")
                .cloned()
                .unwrap_or("".to_string());
            let client_cert = cfg
                .credentials
                .get("client_cert")
                .cloned()
                .unwrap_or("".to_string());
            let client_key = cfg
                .credentials
                .get("client_key")
                .cloned()
                .unwrap_or("".to_string());

            if (!client_cert.is_empty() && client_key.is_empty())
                || (client_cert.is_empty() && !client_key.is_empty())
            {
                return Err(
                    anyhow::anyhow!("client_cert and client_key must be set together").into(),
                );
            }

            let pool = match config.ssl_mode {
                Some(mode) => match mode {
                    SslMode::Prefer | SslMode::Require => {
                        if !ca_cert.is_empty() && !client_cert.is_empty() && !client_key.is_empty()
                        {
                            let mut builder = SslConnector::builder(SslMethod::tls_client())?;
                            builder.set_verify(SslVerifyMode::PEER); // peer - veirfy ca - must add ca file, none - allow self signed or without ca
                            builder.set_ca_file(ca_cert)?;
                            builder.set_certificate_chain_file(client_cert)?;
                            builder.set_private_key_file(client_key, SslFiletype::PEM)?;
                            let connector = MakeTlsConnector::new(builder.build());
                            Some(config.create_pool(rt, connector)?)
                        } else if !ca_cert.is_empty() {
                            let mut builder = SslConnector::builder(SslMethod::tls_client())?;
                            builder.set_verify(SslVerifyMode::PEER); // peer - veirfy ca - must add ca file, none - allow self signed or without ca
                            builder.set_ca_file(cfg.credentials.get("ca_cert").unwrap())?;
                            let connector = MakeTlsConnector::new(builder.build());
                            Some(config.create_pool(rt, connector)?)
                        } else {
                            let mut builder = SslConnector::builder(SslMethod::tls())?;
                            builder.set_verify(SslVerifyMode::NONE); // peer - veirfy ca - must add ca file, none - allow self signed or without ca
                            let connector = MakeTlsConnector::new(builder.build());
                            Some(config.create_pool(rt, connector)?)
                        }
                    }
                    SslMode::Disable => Some(config.create_pool(rt, NoTls)?),
                    _ => None,
                },
                None => None,
            };

            match pool {
                Some(pool) => {
                    let _cfg = config.clone();
                    Ok(InitiatedConnection {
                        config: cfg.clone(),
                        pool: ConnectionPool::Postgresql(pool),
                        opts: ConnectionOpts::Postgresql(_cfg),
                        schema: "public".to_string(),
                    })
                }
                None => Err(anyhow::anyhow!("Cannot create pool").into()),
            }
        }
        Dialect::Sqlite => {
            if cfg.mode != Mode::File {
                return Err(anyhow::anyhow!("Only file mode is supported for Sqlite").into());
            }
            let path = cfg
                .credentials
                .get("path")
                .cloned()
                .unwrap_or("".to_string());
            let config = SqliteConfig::new(PathBuf::from(path.clone()));
            match config.create_pool(deadpool_sqlite::Runtime::Tokio1) {
                Ok(pool) => match pool.get().await {
                    Ok(_) => Ok(InitiatedConnection {
                        config: cfg.clone(),
                        pool: ConnectionPool::Sqlite(pool),
                        opts: ConnectionOpts::Sqlite(config),
                        schema: path.to_string(),
                    }),
                    Err(e) => Err(Error::DeadpoolSqlitePool(e)),
                },
                Err(e) => Err(Error::DeadpoolSqliteCreatePool(e)),
            }
        }
    }
}
