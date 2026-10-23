use foundry_core::error::{AppError, AppResult};
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions, MySqlSslMode};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Parses MySQL connection options from URL and applies global environment overrides if present.
pub fn parse_mysql_options(connection_url: &str) -> AppResult<MySqlConnectOptions> {
    let mut options = MySqlConnectOptions::from_str(connection_url).map_err(|e| {
        AppError::Database(format!(
            "Failed to parse MySQL connection URL '{}': {}",
            connection_url, e
        ))
    })?;

    // Check if SSL mode is explicitly configured via environment variable
    // Only applies if the connection string didn't explicitly specify ssl-mode or sslmode
    let url_has_ssl_mode =
        connection_url.contains("ssl-mode") || connection_url.contains("sslmode");
    if !url_has_ssl_mode
        && let Ok(env_ssl) =
            std::env::var("DATABASE_SSL_MODE").or_else(|_| std::env::var("MYSQL_SSL_MODE"))
    {
        let parsed_mode = match env_ssl.trim().to_ascii_lowercase().as_str() {
            "disabled" | "disable" | "false" | "0" | "off" => Some(MySqlSslMode::Disabled),
            "preferred" | "prefer" => Some(MySqlSslMode::Preferred),
            "required" | "require" | "true" | "1" | "on" => Some(MySqlSslMode::Required),
            "verify_ca" | "verify-ca" => Some(MySqlSslMode::VerifyCa),
            "verify_identity" | "verify-identity" | "verify-full" => {
                Some(MySqlSslMode::VerifyIdentity)
            }
            other => {
                warn!(
                    "Unrecognized DATABASE_SSL_MODE '{}'. Valid values: disabled, preferred, required, verify_ca, verify_identity.",
                    other
                );
                None
            }
        };

        if let Some(mode) = parsed_mode {
            debug!(
                "Applying environment DATABASE_SSL_MODE={:?} to MySQL connection",
                mode
            );
            options = options.ssl_mode(mode);
        }
    }

    Ok(options)
}

/// Connects to MySQL with automatic resilience:
/// - Lazy connection support for "dummy" testing environments
/// - Intelligent fallback to unencrypted connection when legacy X.509 v1 self-signed certificates are encountered in Preferred mode
/// - Rich diagnostic error reporting
pub async fn create_mysql_pool(
    connection_url: &str,
    max_connections: u32,
    acquire_timeout: Duration,
    context: &str,
) -> AppResult<MySqlPool> {
    let options = parse_mysql_options(connection_url)?;

    let pool_options = MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(acquire_timeout);

    if connection_url.contains("dummy") {
        return Ok(pool_options.connect_lazy_with(options));
    }

    match pool_options.clone().connect_with(options.clone()).await {
        Ok(pool) => Ok(pool),
        Err(err) => {
            let err_msg = err.to_string();

            // Check if failure is due to legacy X.509 certificate (e.g. UnsupportedCertVersion from webpki)
            let is_legacy_cert = err_msg.contains("UnsupportedCertVersion")
                || err_msg.contains("invalid peer certificate");
            let is_preferred = matches!(options.get_ssl_mode(), MySqlSslMode::Preferred);

            if is_legacy_cert && is_preferred {
                warn!(
                    "MySQL connection attempt [{context}] encountered legacy TLS peer certificate ({}). \
                     Automatically falling back to unencrypted connection (ssl-mode=disabled). \
                     To suppress this warning, explicitly set '?ssl-mode=disabled' in DATABASE_URL or set DATABASE_SSL_MODE=disabled.",
                    err_msg
                );

                let fallback_options = options.ssl_mode(MySqlSslMode::Disabled);
                match pool_options.connect_with(fallback_options).await {
                    Ok(fallback_pool) => {
                        info!(
                            "MySQL connection successfully established using unencrypted fallback [{context}]."
                        );
                        return Ok(fallback_pool);
                    }
                    Err(fallback_err) => {
                        return Err(AppError::Database(format!(
                            "Failed to connect to MySQL [{context}]: TLS handshake failed with legacy certificate ({err_msg}), and unencrypted fallback also failed: {fallback_err}"
                        )));
                    }
                }
            }

            // Provide user-friendly diagnostic guidance
            let diagnostic = if err_msg.contains("Access denied for user") {
                "Authentication failed. Please verify MySQL username, password, and database host access permissions."
            } else if err_msg.contains("RSA auth backend disabled") {
                "MySQL 8.0+ caching_sha2_password requires RSA encryption when TLS is disabled. Ensure the 'mysql' feature is active in foundry."
            } else if err_msg.contains("Unknown database") {
                "Target database does not exist. Ensure AUTO_MIGRATE=true or create the database before starting."
            } else {
                "Please verify database network connectivity, port, and credentials."
            };

            Err(AppError::Database(format!(
                "Failed to connect to MySQL [{context}]: {err_msg}. Remediation: {diagnostic}"
            )))
        }
    }
}

/// Splits a database connection URL into a server-level URL (without database name but preserving query parameters)
/// and the extracted database name.
///
/// Example: `mysql://user:pass@host:3306/dbname?ssl-mode=disabled`
/// Returns: `("mysql://user:pass@host:3306/?ssl-mode=disabled", "dbname")`
pub fn extract_server_url(connection_url: &str) -> Option<(String, String)> {
    let slash_idx = connection_url.rfind('/')?;
    let (base, db_part) = connection_url.split_at(slash_idx);

    let db_without_leading_slash = db_part.trim_start_matches('/');
    let db_name = db_without_leading_slash
        .split('?')
        .next()
        .unwrap_or("")
        .trim();
    if db_name.is_empty() {
        return None;
    }

    let query_suffix = if let Some(q_idx) = db_without_leading_slash.find('?') {
        &db_without_leading_slash[q_idx..]
    } else {
        ""
    };

    let server_url = format!("{}/{}", base, query_suffix);
    Some((server_url, db_name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_server_url() {
        let (server_url, db) =
            extract_server_url("mysql://root:pass@localhost:3306/foundry_test").unwrap();
        assert_eq!(server_url, "mysql://root:pass@localhost:3306/");
        assert_eq!(db, "foundry_test");

        let (server_url2, db2) = extract_server_url(
            "mysql://root:pass@localhost:3306/foundry_test?ssl-mode=disabled&charset=utf8mb4",
        )
        .unwrap();
        assert_eq!(
            server_url2,
            "mysql://root:pass@localhost:3306/?ssl-mode=disabled&charset=utf8mb4"
        );
        assert_eq!(db2, "foundry_test");
    }

    #[test]
    fn test_parse_mysql_options_default() {
        let opts = parse_mysql_options("mysql://root:pass@localhost:3306/foundry").unwrap();
        assert!(matches!(opts.get_ssl_mode(), MySqlSslMode::Preferred));
    }
}
