#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Domain {
    pub vid: i64,
    pub block_range: sqlx::postgres::types::PgRange<i32>,
    pub id: String,
    pub name: Option<String>,
    pub label_name: Option<String>,
    pub labelhash: Option<Vec<u8>>,
    pub parent: Option<String>,
    pub subdomain_count: i32,
    pub resolved_address: Option<String>,
    pub resolver: Option<String>,
    pub ttl: Option<sqlx::types::BigDecimal>,
    pub is_migrated: bool,
    pub created_at: Option<sqlx::types::BigDecimal>,
    pub owner: String,
    pub registrant: Option<String>,
    pub wrapped_owner: Option<String>,
    pub expiry_date: Option<sqlx::types::BigDecimal>,
}
