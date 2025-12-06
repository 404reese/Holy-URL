use std::sync::Arc;

use async_trait::async_trait;
use cassandra_cpp::{PreparedStatement, Session, Statement};
use tracing::instrument;

use crate::errors::{AppError, AppResult};

const INSERT_QUERY: &str = "INSERT INTO urls (short_id, long_url, created_at) VALUES (?, ?, toTimestamp(now()));";
const SELECT_QUERY: &str = "SELECT long_url FROM urls WHERE short_id = ?;";

#[async_trait]
pub trait UrlRepository: Send + Sync {
    async fn save(&self, short_id: &str, long_url: &str) -> AppResult<()>;
    async fn find(&self, short_id: &str) -> AppResult<Option<String>>;
}

pub struct CassandraRepo {
    session: Arc<Session>,
    insert_stmt: Arc<PreparedStatement>,
    select_stmt: Arc<PreparedStatement>,
}

impl CassandraRepo {
    pub async fn new(session: Arc<Session>) -> AppResult<Self> {
        let insert_stmt = Arc::new(prepare_statement(session.clone(), INSERT_QUERY).await?);
        let select_stmt = Arc::new(prepare_statement(session.clone(), SELECT_QUERY).await?);
        Ok(Self {
            session,
            insert_stmt,
            select_stmt,
        })
    }
}

#[async_trait]
impl UrlRepository for CassandraRepo {
    #[instrument(skip(self, short_id, long_url), fields(short_id = %short_id))]
    async fn save(&self, short_id: &str, long_url: &str) -> AppResult<()> {
        let session = self.session.clone();
        let insert = self.insert_stmt.clone();
        let sid = short_id.to_owned();
        let url = long_url.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut statement = bind_insert(&insert, &sid, &url)?;
            session
                .execute(&statement)
                .wait()
                .map_err(map_cassandra)?;
            Ok(())
        })
        .await
        .map_err(map_join)?
    }

    #[instrument(skip(self), fields(short_id = %short_id))]
    async fn find(&self, short_id: &str) -> AppResult<Option<String>> {
        let session = self.session.clone();
        let select = self.select_stmt.clone();
        let sid = short_id.to_owned();

        tokio::task::spawn_blocking(move || {
            let mut statement = select
                .bind()
                .map_err(map_cassandra)?;
            statement
                .bind_string(0, &sid)
                .map_err(map_cassandra)?;

            let result = session
                .execute(&statement)
                .wait()
                .map_err(map_cassandra)?;

            let row = result.first_row();
            if let Some(row) = row {
                let long_url: String = row
                    .get_column(0)
                    .and_then(|c| c.get_string())
                    .map_err(map_cassandra)?;
                Ok(Some(long_url))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(map_join)?
    }
}

fn bind_insert(
    insert: &PreparedStatement,
    short_id: &str,
    long_url: &str,
) -> AppResult<Statement> {
    let mut statement = insert.bind().map_err(map_cassandra)?;
    statement
        .bind_string(0, short_id)
        .map_err(map_cassandra)?;
    statement
        .bind_string(1, long_url)
        .map_err(map_cassandra)?;
    Ok(statement)
}

async fn prepare_statement(session: Arc<Session>, query: &str) -> AppResult<PreparedStatement> {
    let query_owned = query.to_owned();
    tokio::task::spawn_blocking(move || session.prepare(&query_owned).wait().map_err(map_cassandra))
        .await
        .map_err(map_join)?
}

fn map_cassandra<E: ToString>(err: E) -> AppError {
    AppError::Cassandra(err.to_string())
}

fn map_join(err: tokio::task::JoinError) -> AppError {
    AppError::Unexpected(format!("task join error: {err}"))
}

// TODO: wire Cassandra Session setup (Cluster configuration, auth, pooling) at composition root.
