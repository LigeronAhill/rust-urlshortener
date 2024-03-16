use base64::Engine;
use base64::engine::general_purpose;
use metrics::counter;
use rand::Rng;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Error, PgPool};
use sqlx::error::ErrorKind;
use crate::error::internal_error;
use crate::models::{CountedLinkStatistic, Link};

type Result<T> = std::result::Result<T, sqlx::Error>;
#[derive(Clone)]
pub struct LinkRepository {
    pool: PgPool,
}
impl LinkRepository {
    pub async fn new(db_url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new().max_connections(20).connect(db_url).await?;
        Ok(Self { pool })
    }
    pub async fn get_link(&self, requested_link: &str) -> Result<Option<Link>> {
        sqlx::query_as!(
            Link,
            "SELECT * FROM links WHERE id = $1",
            requested_link
        )
        .fetch_optional(&self.pool).await
    }
    pub async fn create_link(&self, target_url: &str) -> Result<Link> {
        for _ in 1..=3 {
            let new_link_id = generate_id();
            let link = sqlx::query_as!(
            Link,
            r"
            WITH inserted_link as (
                INSERT INTO links (id, target_url) 
                VALUES ($1, $2) 
                RETURNING *
            )
            SELECT * FROM inserted_link",
            &new_link_id,
            target_url
        )
                .fetch_one(&self.pool).await;
            match link {
                Ok(link) => return Ok(link),
                Err(e) => match e {
                    Error::Database(db_err) if db_err.kind() == ErrorKind::UniqueViolation => {}
                    _ => return Err(e),
                },
            }
        }
        tracing::error!("Failed to create link");
        counter!("create_link_failed", "reason" => "too_many_attempts");
        Err(Error::RowNotFound)}
    pub async fn update_link(&self, id: &str, update: &str) -> Result<Link> {
        let link = sqlx::query_as!(
            Link,
            r"
            WITH updated_link as (
                UPDATE links SET target_url = $1 WHERE id = $2
                RETURNING *
            )
            SELECT * FROM updated_link",
            update,
            id
        )
        .fetch_one(&self.pool).await?;
        Ok(link)
    }
    pub async fn add_statistic(&self, link_id: &str, referer: &Option<String>, user_agent: &Option<String>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO link_statistics (link_id, referer, user_agent)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(link_id)
        .bind(referer)
        .bind(user_agent)
        .execute(&self.pool).await?;
        Ok(())
    }
    pub async fn get_link_statistic(&self, link_id: &str) -> Result<Vec<CountedLinkStatistic>> {
        sqlx::query_as!(
            CountedLinkStatistic,
            r#"
            SELECT count(*) as amount, referer, user_agent
            FROM link_statistics
            GROUP BY link_id, referer, user_agent 
            HAVING link_id = $1
            "#,
            link_id
        )
        .fetch_all(&self.pool).await
    }
    pub async fn get_settings(&self) -> Result<crate::auth::Settings> {
        sqlx::query_as!(
            crate::auth::Settings,
            "SELECT id, encrypted_global_api_key FROM settings WHERE id = $1",
            "DEFAULT_SETTINGS"
        )
            .fetch_one(&self.pool)
            .await
    }
}

fn generate_id() -> String {
    let random_number = rand::thread_rng().gen_range(0..u32::MAX);
    general_purpose::URL_SAFE_NO_PAD.encode(random_number.to_string())
}